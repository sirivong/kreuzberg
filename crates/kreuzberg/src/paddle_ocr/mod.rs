//! PaddleOCR backend using ONNX Runtime.
//!
//! This module provides a PaddleOCR implementation that uses ONNX Runtime
//! for inference, enabling high-quality OCR without Python dependencies.
//!
//! # Features
//!
//! - PP-OCRv4/v5 model support
//! - Excellent CJK (Chinese, Japanese, Korean) recognition
//! - Pure Rust implementation via `paddle-ocr-rs`
//! - Shared ONNX Runtime with embeddings feature
//!
//! # Model Files
//!
//! PaddleOCR requires three model files:
//! - Detection model (`*_det_*.onnx`)
//! - Classification model (`*_cls_*.onnx`)
//! - Recognition model (`*_rec_*.onnx`)
//!
//! Models are auto-downloaded on first use to `~/.cache/kreuzberg/paddle-ocr/`.
//!
//! # Example
//!
//! ```rust,ignore
//! use kreuzberg::ocr::paddle::PaddleOcrBackend;
//! use kreuzberg::plugins::OcrBackend;
//! use kreuzberg::OcrConfig;
//!
//! let backend = PaddleOcrBackend::new()?;
//! let config = OcrConfig {
//!     language: "ch".to_string(),
//!     ..Default::default()
//! };
//!
//! let result = backend.process_image(&image_bytes, &config).await?;
//! println!("Extracted: {}", result.content);
//! ```

mod backend;
mod config;
mod model_manager;

pub use backend::PaddleOcrBackend;
pub use config::{PaddleLanguage, PaddleOcrConfig};
pub use model_manager::{CacheStats, ModelManager, ModelPaths, RecModelPaths, SharedModelPaths};

/// Supported languages for PaddleOCR.
///
/// PaddleOCR supports 16 optimized language models covering 106+ languages
/// via 12 script-family recognition models (PP-OCRv5 and PP-OCRv3).
pub const SUPPORTED_LANGUAGES: &[&str] = &[
    "ch",          // Chinese (Simplified)
    "en",          // English
    "french",      // French
    "german",      // German
    "korean",      // Korean
    "japan",       // Japanese
    "chinese_cht", // Chinese (Traditional)
    "ta",          // Tamil
    "te",          // Telugu
    "ka",          // Kannada
    "latin",       // Latin script languages
    "arabic",      // Arabic
    "cyrillic",    // Cyrillic script languages
    "devanagari",  // Devanagari script languages
    "thai",        // Thai
    "greek",       // Greek
];

/// Check if a language code is supported by PaddleOCR.
pub fn is_language_supported(lang: &str) -> bool {
    SUPPORTED_LANGUAGES.contains(&lang)
}

/// Map a PaddleOCR language code to its script family.
///
/// Script families group languages that share a single recognition model.
/// For example, French, German, and Spanish all use the `latin` rec model.
/// Chinese simplified, traditional, and Japanese share the `chinese` rec model.
///
/// # Script Families (12)
///
/// | Family | PP-OCR Version | Languages |
/// |---|---|---|
/// | `english` | v5 | English |
/// | `chinese` | v5 server | Chinese (simplified+traditional), Japanese |
/// | `latin` | v5 | French, German, Spanish, Italian, 40+ more |
/// | `korean` | v5 | Korean |
/// | `eslav` | v5 | Russian, Ukrainian, Belarusian |
/// | `thai` | v5 | Thai |
/// | `greek` | v5 | Greek |
/// | `arabic` | v3 | Arabic, Persian, Urdu |
/// | `devanagari` | v3 | Hindi, Marathi, Nepali, 10+ more |
/// | `tamil` | v3 | Tamil |
/// | `telugu` | v3 | Telugu |
/// | `kannada` | v3 | Kannada |
pub fn language_to_script_family(paddle_lang: &str) -> &'static str {
    match paddle_lang {
        "en" => "english",
        "ch" | "japan" | "chinese_cht" => "chinese",
        "korean" => "korean",
        "french" | "german" | "latin" => "latin",
        "arabic" => "arabic",
        "cyrillic" => "eslav",
        "devanagari" => "devanagari",
        "ta" => "tamil",
        "te" => "telugu",
        "ka" => "kannada",
        "thai" => "thai",
        "greek" => "greek",
        _ => "english",
    }
}

/// Map Kreuzberg language codes to PaddleOCR language codes.
pub fn map_language_code(kreuzberg_code: &str) -> Option<&'static str> {
    match kreuzberg_code {
        // Direct mappings
        "ch" | "chi_sim" | "zho" | "zh" | "chinese" => Some("ch"),
        "en" | "eng" | "english" => Some("en"),
        "fr" | "fra" | "french" => Some("french"),
        "de" | "deu" | "german" => Some("german"),
        "ko" | "kor" | "korean" => Some("korean"),
        "ja" | "jpn" | "japanese" | "japan" => Some("japan"),
        "chi_tra" | "zh_tw" | "zh_hant" | "chinese_cht" => Some("chinese_cht"),
        "ta" | "tam" | "tamil" => Some("ta"),
        "te" | "tel" | "telugu" => Some("te"),
        // Kannada: "kn" is ISO 639-1 (correct), "ka" is PaddleOCR-native code for Kannada
        // Note: "ka" is ISO 639-1 for Georgian, but PaddleOCR uses "ka" for Kannada
        "ka" | "kn" | "kan" | "kannada" => Some("ka"),
        "ar" | "ara" | "arabic" => Some("arabic"),
        "ru" | "rus" | "russian" | "uk" | "ukr" | "ukrainian" | "be" | "bel" | "belarusian" | "cyrillic" => {
            Some("cyrillic")
        }
        "hi" | "hin" | "hindi" | "devanagari" => Some("devanagari"),
        "th" | "tha" | "thai" => Some("thai"),
        "el" | "ell" | "greek" => Some("greek"),
        // Latin script fallback for European languages
        "latin" | "es" | "spa" | "spanish" | "it" | "ita" | "italian" | "pt" | "por" | "portuguese" | "nl" | "nld"
        | "dutch" | "pl" | "pol" | "polish" | "sv" | "swe" | "swedish" | "da" | "dan" | "danish" | "no" | "nor"
        | "norwegian" | "fi" | "fin" | "finnish" | "cs" | "ces" | "czech" | "sk" | "slk" | "slovak" | "hr" | "hrv"
        | "croatian" | "hu" | "hun" | "hungarian" | "ro" | "ron" | "romanian" | "tr" | "tur" | "turkish" | "id"
        | "ind" | "indonesian" | "ms" | "msa" | "malay" | "vi" | "vie" | "vietnamese" => Some("latin"),
        _ => None,
    }
}
