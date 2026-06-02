//! Named-entity recognition (NER).
//!
//! Shared by:
//! - the NER post-processor at `crate::plugins::processor::builtin::ner` (populates
//!   [`ExtractionResult::entities`](crate::types::ExtractionResult::entities))
//! - the redaction engine at `crate::text::redaction::engine` (consumes the same
//!   `Entity` stream to redact PERSON / ORGANIZATION / LOCATION mentions that the
//!   pure-Rust pattern engine cannot detect).
//!
//! Backends implement the [`NerBackend`] trait. Two are bundled:
//!
//! - [`gline::GlineBackend`] under `#[cfg(feature = "ner-onnx")]` — local ONNX
//!   inference via the upstream `gline-rs` crate. Models download lazily from
//!   HuggingFace via [`crate::model_download`].
//! - [`llm::LlmBackend`] under `#[cfg(feature = "ner-llm")]` — liter-llm with a
//!   structured-output schema. Used when categories outstrip the ONNX taxonomy.

#![cfg(feature = "ner")]

pub mod backend;
#[cfg(feature = "ner-onnx")]
pub mod gline;
#[cfg(feature = "ner-llm")]
pub mod llm;

pub use backend::NerBackend;
