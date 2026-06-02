//! NER (named-entity recognition) configuration.
//!
//! When `ExtractionConfig::ner` is `Some`, the NER post-processor runs after
//! extraction and populates [`ExtractionResult::entities`](crate::types::ExtractionResult::entities).

use crate::types::entity::EntityCategory;
use serde::{Deserialize, Serialize};

/// Configuration for the NER post-processor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct NerConfig {
    /// Backend that runs the entity detection.
    #[serde(default)]
    pub backend: NerBackendKind,
    /// Entity categories to detect. Defaults to a sensible PERSON/ORG/LOCATION/EMAIL set
    /// when empty.
    #[serde(default)]
    pub categories: Vec<EntityCategory>,
    /// Override the default model — only used by [`NerBackendKind::Onnx`].
    /// `None` lets the backend pick its pinned default
    /// (`urchade/gliner_multi-v2.1` for gline-rs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional LLM configuration — only used by [`NerBackendKind::Llm`]. Token usage
    /// for LLM backends is recorded in `ExtractionResult::llm_usage`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<super::llm::LlmConfig>,
}

impl Default for NerConfig {
    fn default() -> Self {
        Self {
            backend: NerBackendKind::default(),
            categories: Vec::new(),
            model: None,
            llm: None,
        }
    }
}

/// NER backend selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum NerBackendKind {
    /// gline-rs ONNX inference. Requires `ner-onnx` feature. Models download lazily from
    /// HuggingFace via `model_download::hf_download`.
    #[default]
    Onnx,
    /// liter-llm zero-shot NER via structured-output prompts. Requires `ner-llm`
    /// feature. Useful when domain-specific categories outstrip the ONNX taxonomy.
    Llm,
}
