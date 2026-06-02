//! Document-summarisation configuration.
//!
//! When `ExtractionConfig::summarization` is `Some`, the summarisation post-processor
//! runs at the Middle stage and populates
//! [`ExtractionResult::summary`](crate::types::ExtractionResult::summary).

use crate::types::summary::SummaryStrategy;
use serde::{Deserialize, Serialize};

/// Configuration for the summarisation post-processor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct SummarizationConfig {
    /// Summarisation strategy.
    #[serde(default)]
    pub strategy: SummaryStrategy,
    /// Maximum summary length in tokens. `None` lets the backend pick a default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// LLM configuration for the abstractive backend. Ignored when
    /// `strategy = Extractive`. Required when `strategy = Abstractive`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm: Option<super::llm::LlmConfig>,
}

impl Default for SummarizationConfig {
    fn default() -> Self {
        Self {
            strategy: SummaryStrategy::default(),
            max_tokens: None,
            llm: None,
        }
    }
}

impl Default for SummaryStrategy {
    fn default() -> Self {
        Self::Extractive
    }
}
