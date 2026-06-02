//! # kreuzberg-candle-ocr
//!
//! Candle-based VLM OCR engines for Kreuzberg. Pure-Rust transformer OCR.
//!
//! ## Phase 1 status
//!
//! This crate currently exposes only the skeleton: error types, device selection,
//! and per-model module placeholders. Real model code is wired in Phase 3 per the
//! plan at the end of `feat/candle-backends`.
//!
//! ## Per-model sub-features
//!
//! - `trocr` — Microsoft TrOCR
//! - `paddleocr-vl` — PaddleOCR-VL 0.9B
//! - `got-ocr` — GOT-OCR 2.0 0.7B
//! - `glm-ocr` — GLM-OCR 0.9B
//!
//! ## Device acceleration
//!
//! Pass-through features to candle: `cuda`, `metal`, `mkl`, `accelerate`.

#![allow(clippy::too_many_arguments)]

pub mod device;
pub mod error;
pub mod models;

pub use device::DevicePreference;
pub use error::{CandleOcrError, Result};

#[cfg(not(target_arch = "wasm32"))]
pub use candle_core::DType;

/// Identifier for the model emitted by a [`CandleEngine`]. Used by the
/// backend layer to record telemetry and pick decoding hyperparameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelKind {
    Trocr,
    PaddleOcrVl,
    GotOcr,
    GlmOcr,
}

/// Output produced by a candle OCR engine for a single image.
#[derive(Debug, Clone)]
pub struct CandleOcrOutput {
    /// Recognised content. For VLM backends this is markdown; for TrOCR it is plain text.
    pub content: String,
    /// True if `content` is markdown (and the extraction pipeline should skip
    /// layout-reconstruction stages).
    pub is_structured_markdown: bool,
    /// Optional model-emitted confidence in `[0.0, 1.0]`. `None` if the model
    /// does not expose token-level confidences.
    pub confidence: Option<f32>,
}
