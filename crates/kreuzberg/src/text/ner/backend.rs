//! Backend trait shared by every NER implementation.

use crate::Result;
use crate::types::entity::{Entity, EntityCategory};
use async_trait::async_trait;

/// One-method trait that every NER backend implements.
///
/// The redaction engine and the NER post-processor both consume backends through
/// this trait so they can be swapped without rewriting consumer code.
#[async_trait]
#[cfg_attr(alef, alef(skip))]
pub trait NerBackend: Send + Sync {
    /// Identify entities in `text` belonging to any of the `categories`.
    ///
    /// Implementations must return entities in source byte-offset order. Byte offsets
    /// are 0-indexed and refer to UTF-8 byte positions in `text`. When `categories`
    /// is empty the backend returns every entity it can identify.
    async fn detect(&self, text: &str, categories: &[EntityCategory]) -> Result<Vec<Entity>>;
}
