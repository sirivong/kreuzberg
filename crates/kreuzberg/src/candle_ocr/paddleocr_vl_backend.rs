//! PaddleOCR-VL backend plugin for the Kreuzberg OCR pipeline.
//!
//! This module wraps the candle-based PaddleOCR-VL engine in the `OcrBackend` trait,
//! making it available to the extraction pipeline.

use async_trait::async_trait;
use std::borrow::Cow;
use std::path::Path;

use crate::Result;
use crate::core::config::OcrConfig;
use crate::plugins::{OcrBackend, OcrBackendType, Plugin};
use crate::types::ExtractionResult;
use kreuzberg_candle_ocr::DevicePreference;
use kreuzberg_candle_ocr::models::PaddleOcrVlTask;
#[cfg(not(target_arch = "wasm32"))]
use kreuzberg_candle_ocr::DType;

/// PaddleOCR-VL backend using candle transformers.
///
/// A vision-language model for comprehensive document parsing. Supports text recognition,
/// tables, formulas, and charts through a unified interface with markdown output.
///
/// Supports 109+ languages through the PaddlePaddle pretrained models.
///
/// # Configuration
///
/// PaddleOCR-VL accepts backend options for task selection:
/// ```json
/// {
///   "task": "ocr",
///   "device": "auto"
/// }
/// ```
///
/// - `task` (string): `"ocr"` (default), `"table"`, `"formula"`, `"chart"`
/// - `device` (string): `"auto"`, `"cpu"`, `"cuda"`, `"metal"`
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
    fn parse_options(config: &OcrConfig) -> (PaddleOcrVlTask, DevicePreference) {
        let mut task = PaddleOcrVlTask::default();
        let mut device = DevicePreference::default();

        if let Some(opts) = &config.backend_options {
            // Parse task preference
            if let Some(t) = opts.get("task").and_then(|v| v.as_str()) {
                task = match t {
                    "table" => PaddleOcrVlTask::Table,
                    "formula" => PaddleOcrVlTask::Formula,
                    "chart" => PaddleOcrVlTask::Chart,
                    _ => PaddleOcrVlTask::Ocr, // default on unknown
                };
            }

            // Parse device preference
            if let Some(d) = opts.get("device").and_then(|v| v.as_str()) {
                device = match d {
                    "cpu" => DevicePreference::Cpu,
                    "cuda" => DevicePreference::Cuda,
                    "metal" => DevicePreference::Metal,
                    _ => DevicePreference::Auto,
                };
            }
        }

        (task, device)
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
    async fn process_image(&self, image_bytes: &[u8], config: &OcrConfig) -> Result<ExtractionResult> {
        // Parse configuration
        let (task, device) = Self::parse_options(config);

        // Validate image data
        if image_bytes.is_empty() {
            return Err(crate::KreuzbergError::Validation {
                message: "Empty image data provided to PaddleOCR-VL".to_string(),
                source: None,
            });
        }

        // Clone image bytes for async block
        let image_bytes = image_bytes.to_vec();
        let task = task;
        let device = device;

        // Run inference in a blocking task to avoid blocking the async runtime
        let content = tokio::task::spawn_blocking(move || {
            // Select compute device
            let candle_device = device.select().map_err(|e| crate::KreuzbergError::Ocr {
                message: format!("Failed to select compute device: {}", e),
                source: None,
            })?;

            // Load or retrieve cached engine
            // Use f32 dtype for compatibility and stability
            let engine = kreuzberg_candle_ocr::models::PaddleOcrVlEngine::new(task, candle_device, DType::F32)
                .map_err(|e| crate::KreuzbergError::Ocr {
                    message: format!("PaddleOCR-VL engine initialization failed: {}", e),
                    source: None,
                })?;

            // Process image through encoder-decoder pipeline
            let output = engine
                .process_image(&image_bytes)
                .map_err(|e| crate::KreuzbergError::Ocr {
                    message: format!("PaddleOCR-VL inference failed: {}", e),
                    source: None,
                })?;

            Ok::<String, crate::KreuzbergError>(output.content)
        })
        .await
        .map_err(|e| crate::KreuzbergError::Ocr {
            message: format!("PaddleOCR-VL task execution failed: {}", e),
            source: None,
        })??;

        Ok(ExtractionResult {
            content,
            mime_type: Cow::Borrowed("text/markdown"),
            ..Default::default()
        })
    }

    async fn process_image_file(&self, path: &Path, config: &OcrConfig) -> Result<ExtractionResult> {
        let bytes = crate::core::io::read_file_async(path).await?;
        self.process_image(&bytes, config).await
    }

    fn supports_language(&self, _lang: &str) -> bool {
        // PaddleOCR-VL supports 109+ languages as per the official model documentation.
        // For simplicity, accept all language codes.
        true
    }

    fn supported_languages(&self) -> Vec<String> {
        // Major language codes supported by PaddleOCR-VL
        vec![
            "eng", "en", // English
            "zho", "zh", // Chinese (simplified and traditional)
            "jpn", "ja", // Japanese
            "kor", "ko", // Korean
            "fra", "fr", // French
            "deu", "de", // German
            "spa", "es", // Spanish
            "ita", "it", // Italian
            "por", "pt", // Portuguese
            "rus", "ru", // Russian
            "ara", "ar", // Arabic
            "hin", "hi", // Hindi
            "tha", "th", // Thai
            "vie", "vi", // Vietnamese
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn backend_type(&self) -> OcrBackendType {
        OcrBackendType::Candle
    }

    fn emits_structured_markdown(&self) -> bool {
        // PaddleOCR-VL emits markdown output directly from the VLM,
        // so the extraction pipeline should skip layout reconstruction stages.
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
        // Should support common language codes
        assert!(backend.supports_language("eng"));
        assert!(backend.supports_language("zho"));
        assert!(backend.supports_language("jpn"));
        assert!(backend.supports_language("fra"));
        // Should also support unknown codes (accept all)
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
        let (task, device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(task, PaddleOcrVlTask::Ocr);
        assert_eq!(device, DevicePreference::Auto);
    }

    #[test]
    fn test_parse_options_custom_task() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "task": "table"
        }));
        let (task, _device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(task, PaddleOcrVlTask::Table);
    }

    #[test]
    fn test_parse_options_custom_device() {
        let mut config = OcrConfig::default();
        config.backend_options = Some(serde_json::json!({
            "device": "cpu"
        }));
        let (_task, device) = PaddleOcrVlBackend::parse_options(&config);
        assert_eq!(device, DevicePreference::Cpu);
    }

    #[test]
    fn test_initialize_and_shutdown() {
        let backend = PaddleOcrVlBackend::default_task();
        assert!(backend.initialize().is_ok());
        assert!(backend.shutdown().is_ok());
    }
}
