//! Redaction & anonymisation configuration.
//!
//! When `ExtractionConfig::redaction` is `Some`, the redaction post-processor runs
//! as the Late stage of the pipeline and rewrites `content`, `formatted_content`,
//! every chunk's text, and the textual fields of `entities` / `summary` /
//! `translation` / `page_classifications` using the configured strategy. The
//! original text never appears in the returned `ExtractionResult`.

use crate::types::redaction::{PiiCategory, RedactionStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for the redaction post-processor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct RedactionConfig {
    /// Categories to redact. Empty means "every category supported by the engine."
    #[serde(default)]
    #[cfg_attr(feature = "api", schema(value_type = Vec<PiiCategory>))]
    pub categories: HashSet<PiiCategory>,
    /// Strategy applied to every match.
    #[serde(default)]
    pub strategy: RedactionStrategy,
    /// Optional NER backend — required to redact PERSON / ORGANIZATION / LOCATION
    /// categories (the pure-Rust pattern engine only covers regex-detectable PII).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ner: Option<super::ner::NerConfig>,
    /// When `true`, chunk byte ranges are kept consistent with the rewritten content by
    /// adjusting `byte_start` / `byte_end` after replacement. When `false`, chunk byte
    /// ranges still refer to the *original* content offsets — useful when downstream
    /// consumers want to map findings back to the original document.
    #[serde(default = "default_preserve_offsets")]
    pub preserve_offsets: bool,
}

fn default_preserve_offsets() -> bool {
    true
}

impl Default for RedactionConfig {
    fn default() -> Self {
        Self {
            categories: HashSet::new(),
            strategy: RedactionStrategy::default(),
            ner: None,
            preserve_offsets: true,
        }
    }
}

impl Default for RedactionStrategy {
    fn default() -> Self {
        Self::Mask
    }
}
