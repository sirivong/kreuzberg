//! PaddleOCR-VL backend plugin for the Xberg OCR pipeline.
//!
//! This module wraps the candle-based PaddleOCR-VL engine in the `OcrBackend`
//! trait, making it available to the extraction pipeline. The engine itself
//! (`xberg_candle_ocr::models::PaddleOcrVlEngine`) implements the PaddleOCR-VL 1.5
//! architecture (SigLIP vision encoder + ERNIE 4.5 text decoder); PaddleOCR-VL 1.6 is
//! a weights-only upgrade that keeps this architecture unchanged, so the same engine
//! loads either checkpoint — only the default `model_id` changed.
//!
//! # Engine pool design
//!
//! The pool key is `(task, DevicePreference)`. Engines are expensive to initialise
//! (~900 MB – 2 GB of safetensors weights depending on the checkpoint). The pool
//! ensures each `(task, device)` combination is loaded at most once per process.
//!
//! `PaddleOcrVlEngine::process_image` takes `&mut self` (the model maintains KV
//! cache state), so the pool stores engines wrapped in `parking_lot::Mutex` for
//! interior mutability.
//!
//! # Weight resolution
//!
//! `backend_options.model_path`, when present, always wins (offline / custom weights).
//! Otherwise the backend auto-downloads `backend_options.model_id` (default
//! `xberg-io/paddleocr-vl-1.6`, a checksum-pinned mirror of
//! `PaddlePaddle/PaddleOCR-VL-1.6`) through [`super::model_stager`].

use async_trait::async_trait;
use std::borrow::Cow;
use std::path::Path;
use std::sync::{Arc, LazyLock};

use ahash::AHashMap;
use parking_lot::{Mutex, RwLock};

use crate::Result;
use crate::core::config::OcrConfig;
use crate::plugins::{OcrBackend, OcrBackendType, Plugin};
use crate::types::ExtractedDocument;
use xberg_candle_ocr::DType;
use xberg_candle_ocr::DevicePreference;
use xberg_candle_ocr::models::PaddleOcrVlEngine;
use xberg_candle_ocr::models::PaddleOcrVlTask;

/// Engine pool key: `(task, device_preference)`.
type PoolKey = (PaddleOcrVlTask, DevicePreference);
/// Pooled engine value: mutex-wrapped engine for interior mutability.
type PooledEngine = Arc<Mutex<PaddleOcrVlEngine>>;

/// Process-wide engine pool keyed by `(task, DevicePreference)`.
///
/// `DevicePreference::Auto` keeps its own slot because it resolves to whatever
/// is available at runtime — collapsing it onto a concrete device would be wrong.
///
/// Engines are wrapped in `Mutex` because `PaddleOcrVlEngine::process_image`
/// takes `&mut self` (it manages an internal KV cache).
static ENGINE_POOL: LazyLock<RwLock<AHashMap<PoolKey, PooledEngine>>> = LazyLock::new(|| RwLock::new(AHashMap::new()));

/// Return a cached engine for `(task, preference)`, initialising one on first use.
///
/// Uses a read → miss → write → double-check pattern so that two racing callers
/// do not both pay the initialisation cost.
///
/// # Errors
///
/// Returns [`crate::XbergError::Ocr`] if device selection fails or the
/// engine cannot be initialised from the model directory.
fn get_or_init_engine(
    model_path: &str,
    task: PaddleOcrVlTask,
    preference: DevicePreference,
) -> crate::Result<PooledEngine> {
    let key: PoolKey = (task, preference);

    {
        let pool = ENGINE_POOL.read();
        if let Some(engine) = pool.get(&key) {
            return Ok(Arc::clone(engine));
        }
    }

    let candle_device = preference.select().map_err(|e| crate::XbergError::Ocr {
        message: format!("Failed to select compute device: {e}"),
        source: Some(Box::new(e)),
    })?;

    tracing::info!(
        task = ?task,
        preference = ?preference,
        "Initialising PaddleOCR-VL engine (cold start)"
    );
    let new_engine =
        PaddleOcrVlEngine::new(model_path, task, candle_device, DType::F32).map_err(|e| crate::XbergError::Ocr {
            message: format!("PaddleOCR-VL engine initialisation failed: {e}"),
            source: Some(Box::new(e)),
        })?;
    let new_engine = Arc::new(Mutex::new(new_engine));

    let mut pool = ENGINE_POOL.write();
    if let Some(existing) = pool.get(&key) {
        return Ok(Arc::clone(existing));
    }
    pool.insert(key, Arc::clone(&new_engine));
    Ok(new_engine)
}

/// Default HuggingFace repo id for PaddleOCR-VL weights: a checksum-pinned mirror of
/// `PaddlePaddle/PaddleOCR-VL-1.6`. Used when `backend_options` provides neither
/// `model_path` nor a custom `model_id`.
const DEFAULT_MODEL_ID: &str = "xberg-io/paddleocr-vl-1.6";

/// PaddleOCR-VL backend using candle transformers.
///
/// A vision-language model for comprehensive document parsing. Supports text
/// recognition, tables, formulas, and charts through a unified interface with
/// markdown output. The engine implements the PaddleOCR-VL 1.5 architecture (SigLIP
/// vision encoder + ERNIE 4.5 text decoder); PaddleOCR-VL 1.6 keeps that architecture
/// unchanged, so it loads through the same engine — only the default weights changed.
///
/// Supports 109+ languages through the PaddlePaddle pretrained models.
///
/// # Configuration
///
/// PaddleOCR-VL accepts backend options for task selection, device, and weight source:
/// ```json
/// {
///   "task": "ocr",
///   "device": "auto",
///   "model_id": "xberg-io/paddleocr-vl-1.6",
///   "model_path": "/path/to/paddleocr-vl-model"
/// }
/// ```
///
/// - `task` (string): `"ocr"` (default), `"table"`, `"formula"`, `"chart"`
/// - `device` (string): `"auto"`, `"cpu"`, `"cuda"`, `"metal"`
/// - `model_id` (string): HuggingFace repo id to auto-download weights from. Defaults to
///   `xberg-io/paddleocr-vl-1.6`, a checksum-pinned mirror of
///   `PaddlePaddle/PaddleOCR-VL-1.6`. Ignored when `model_path` is set.
/// - `model_path` (string, optional): path to a local model directory. When omitted,
///   the weights named by `model_id` are downloaded on first use and cached under the
///   xberg cache directory (`hf-hub`'s shared blob cache) — no manual staging required.
#[cfg_attr(alef, alef(skip))]
pub struct PaddleOcrVlBackend {
    task: PaddleOcrVlTask,
}

impl PaddleOcrVlBackend {
    /// Create a new PaddleOCR-VL backend with the specified task.
    pub fn new(task: PaddleOcrVlTask) -> Self {
        Self { task }
    }

    /// Create a PaddleOCR-VL backend with the default task (OCR).
    pub fn default_task() -> Self {
        Self::new(PaddleOcrVlTask::default())
    }

    /// Parse backend options to extract PaddleOCR-VL-specific configuration.
    ///
    /// Device selection is delegated to [`crate::candle_ocr::resolve_device_preference`]
    /// so the central `AccelerationConfig` is honoured.
    ///
    /// Returns `(task, model_path, model_id, device_preference)`. `model_id` defaults to
    /// [`DEFAULT_MODEL_ID`] and is only consulted when `model_path` is absent.
    fn parse_options(config: &OcrConfig) -> (PaddleOcrVlTask, Option<String>, String, DevicePreference) {
        let mut task = PaddleOcrVlTask::default();
        let mut model_path: Option<String> = None;
        let mut model_id = DEFAULT_MODEL_ID.to_string();

        if let Some(opts) = &config.backend_options {
            if let Some(t) = opts.get("task").and_then(|v| v.as_str()) {
                task = match t {
                    "table" => PaddleOcrVlTask::Table,
                    "formula" => PaddleOcrVlTask::Formula,
                    "chart" => PaddleOcrVlTask::Chart,
                    _ => PaddleOcrVlTask::Ocr,
                };
            }
            if let Some(p) = opts.get("model_path").and_then(|v| v.as_str()) {
                model_path = Some(p.to_string());
            }
            if let Some(id) = opts.get("model_id").and_then(|v| v.as_str()) {
                model_id = id.to_string();
            }
        }

        let device = super::resolve_device_preference(config);
        (task, model_path, model_id, device)
    }
}

impl Plugin for PaddleOcrVlBackend {
    fn name(&self) -> &str {
        "candle-paddleocr-vl"
    }

    fn version(&self) -> String {
        "0.1.0".to_string()
    }

    fn initialize(&self) -> Result<()> {
        tracing::debug!("Initializing PaddleOCR-VL backend: {} task", self.task);
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl OcrBackend for PaddleOcrVlBackend {
    /// Process an image using the PaddleOCR-VL engine.
    ///
    /// # Errors
    ///
    /// Returns [`crate::XbergError::Validation`] if `image_bytes` is empty.
    /// Returns [`crate::XbergError::Ocr`] if weight download, device selection,
    /// engine initialisation, or inference fails.
    async fn process_image(&self, image_bytes: &[u8], config: &OcrConfig) -> Result<ExtractedDocument> {
        let (task, model_path, model_id, device) = Self::parse_options(config);

        if image_bytes.is_empty() {
            return Err(crate::XbergError::Validation {
                message: "Empty image data provided to PaddleOCR-VL".to_string(),
                source: None,
            });
        }

        let image_bytes = image_bytes.to_vec();

        let content = tokio::task::spawn_blocking(move || {
            let model_path = match model_path {
                Some(p) => p,
                None => super::model_stager::ensure_paddleocr_vl_16(&model_id)
                    .map(|dir| dir.to_string_lossy().into_owned())
                    .map_err(|e| crate::XbergError::Ocr {
                        message: format!("PaddleOCR-VL weight download failed: {e}"),
                        source: None,
                    })?,
            };
            let engine = get_or_init_engine(&model_path, task, device)?;

            let mut engine_guard = engine.lock();
            let output = engine_guard
                .process_image(&image_bytes)
                .map_err(|e| crate::XbergError::Ocr {
                    message: format!("PaddleOCR-VL inference failed: {e}"),
                    source: Some(Box::new(e)),
                })?;

            Ok::<String, crate::XbergError>(output.content)
        })
        .await
        .map_err(|e| crate::XbergError::Ocr {
            message: format!("PaddleOCR-VL task execution failed: {e}"),
            source: None,
        })??;

        Ok(ExtractedDocument {
            content,
            mime_type: Cow::Borrowed("text/markdown"),
            ..Default::default()
        })
    }

    /// Process an image file using the PaddleOCR-VL engine.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if inference fails.
    async fn process_image_file(&self, path: &Path, config: &OcrConfig) -> Result<ExtractedDocument> {
        let bytes = crate::core::io::read_file_async(path).await?;
        self.process_image(&bytes, config).await
    }

    fn supports_language(&self, _lang: &str) -> bool {
        true
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "eng", "en", "zho", "zh", "jpn", "ja", "kor", "ko", "fra", "fr", "deu", "de", "spa", "es", "ita", "it",
            "por", "pt", "rus", "ru", "ara", "ar", "hin", "hi", "tha", "th", "vie", "vi",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn backend_type(&self) -> OcrBackendType {
        OcrBackendType::Candle
    }

    fn emits_structured_markdown(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paddleocr_vl_backend_creation() {
        let backend = PaddleOcrVlBackend::default_task();
        assert_eq!(backend.name(), "candle-paddleocr-vl");
        assert_eq!(backend.backend_type(), OcrBackendType::Candle);
    }

    #[test]
    fn test_paddleocr_vl_emits_structured_markdown() {
        let backend = PaddleOcrVlBackend::default_task();
        assert!(backend.emits_structured_markdown());
    }

    #[test]
    fn test_paddleocr_vl_language_support() {
        let backend = PaddleOcrVlBackend::default_task();
        assert!(backend.supports_language("eng"));
        assert!(backend.supports_language("zho"));
        assert!(backend.supports_language("jpn"));
        assert!(backend.supports_language("fra"));
        assert!(backend.supports_language("unknown"));
    }

    #[test]
    fn test_paddleocr_vl_supported_languages() {
        let backend = PaddleOcrVlBackend::default_task();
        let langs = backend.supported_languages();
        assert!(langs.contains(&"eng".to_string()));
        assert!(langs.contains(&"zho".to_string()));
        assert!(langs.contains(&"jpn".to_string()));
    }

    #[test]
    fn test_parse_options_defaults() {
        let config = OcrConfig::default();
        let (task, model_path, model_id, device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(task, PaddleOcrVlTask::Ocr);
        assert!(model_path.is_none());
        assert_eq!(model_id, DEFAULT_MODEL_ID);
        assert_eq!(device, DevicePreference::Auto);
    }

    #[test]
    fn test_parse_options_custom_task() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "task": "table"
        }));
        let (task, _model_path, _model_id, _device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(task, PaddleOcrVlTask::Table);
    }

    #[test]
    fn test_parse_options_custom_device() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "device": "cpu"
        }));
        let (_task, _model_path, _model_id, device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(device, DevicePreference::Cpu);
    }

    #[test]
    fn test_parse_options_model_path() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "model_path": "/models/paddleocr-vl"
        }));
        let (_task, model_path, _model_id, _device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(model_path.as_deref(), Some("/models/paddleocr-vl"));
    }

    #[test]
    fn test_parse_options_custom_model_id() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "model_id": "some-org/custom-paddleocr-vl"
        }));
        let (_task, model_path, model_id, _device) = PaddleOcrVlBackend::parse_options(&config);
        assert!(model_path.is_none());
        assert_eq!(model_id, "some-org/custom-paddleocr-vl");
    }

    #[test]
    fn test_initialize_and_shutdown() {
        let backend = PaddleOcrVlBackend::default_task();
        assert!(backend.initialize().is_ok());
        assert!(backend.shutdown().is_ok());
    }
}
