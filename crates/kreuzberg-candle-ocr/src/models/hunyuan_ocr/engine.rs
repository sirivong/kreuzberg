// Vendored from jhqxxx/aha (Apache-2.0). See repo-root ATTRIBUTIONS.md § jhqxxx/aha.

//! Inference engine for Hunyuan-OCR: orchestrates model loading and inference.

use candle_core::{DType, Device};
use candle_nn::VarBuilder;

use crate::error::{Result, CandleOcrError};
use crate::models::hunyuan_ocr::config::{HunYuanVLConfig, HunyuanOCRGenerationConfig};
use crate::models::hunyuan_ocr::model::HunyuanVLModel;
use crate::models::hunyuan_ocr::processor::HunyuanVLProcessor;

/// Hunyuan-OCR inference engine: manages model, processor, and generation config.
pub struct HunyuanOCREngine {
    processor: HunyuanVLProcessor,
    model: HunyuanVLModel,
    device: Device,
    generation_config: HunyuanOCRGenerationConfig,
    model_name: String,
}

impl HunyuanOCREngine {
    /// Initialize the Hunyuan-OCR engine from a model directory.
    ///
    /// # Arguments
    /// * `path` - Path to model directory containing config.json, generation_config.json, etc.
    /// * `device` - Optional device; defaults to CPU if None
    /// * `dtype` - Optional data type; defaults to based on config.dtype
    pub fn init(path: &str, device: Option<&Device>, dtype: Option<DType>) -> Result<Self> {
        let config_path = format!("{}/config.json", path);
        let config_bytes = std::fs::read(&config_path)?;
        let cfg: HunYuanVLConfig = serde_json::from_slice(&config_bytes)
            .map_err(|e| CandleOcrError::ModelLoadFailed(format!("Parse config: {}", e)))?;

        let device = device.cloned().unwrap_or(Device::Cpu);

        let cfg_dtype = cfg.dtype.as_str();
        let dtype = dtype.unwrap_or(match cfg_dtype {
            "bfloat16" => DType::BF16,
            "float16" => DType::F16,
            _ => DType::F32,
        });

        let processor = HunyuanVLProcessor::new(path, &device, dtype)?;

        // Find and mmap safetensors files.
        let model_files: Vec<String> = std::fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "safetensors")
            })
            .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
            .collect();

        if model_files.is_empty() {
            return Err(CandleOcrError::ModelLoadFailed(
                "No safetensors files found in model path".to_string(),
            ));
        }

        #[allow(unsafe_code)]
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&model_files, dtype, &device)
                .map_err(|e| CandleOcrError::ModelLoadFailed(format!("Load weights: {}", e)))?
        };

        let generation_config_path = format!("{}/generation_config.json", path);
        let gen_config_bytes = std::fs::read(&generation_config_path)?;
        let generation_config: HunyuanOCRGenerationConfig = serde_json::from_slice(&gen_config_bytes)
            .map_err(|e| CandleOcrError::ModelLoadFailed(format!("Parse generation config: {}", e)))?;

        let model = HunyuanVLModel::new(vb, cfg, generation_config.eos_token_id.clone())?;

        let model_name = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("hunyuan_ocr")
            .to_string();

        Ok(Self {
            processor,
            model,
            device,
            generation_config,
            model_name,
        })
    }

    /// Get the model name.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Get a reference to the generation config.
    pub fn generation_config(&self) -> &HunyuanOCRGenerationConfig {
        &self.generation_config
    }

    /// Get a mutable reference to the model.
    pub fn model_mut(&mut self) -> &mut HunyuanVLModel {
        &mut self.model
    }

    /// Get a reference to the processor.
    pub fn processor(&self) -> &HunyuanVLProcessor {
        &self.processor
    }

    /// Get the device used for inference.
    pub fn device(&self) -> &Device {
        &self.device
    }
}
