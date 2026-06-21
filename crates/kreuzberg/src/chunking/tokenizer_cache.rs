//! In-memory cache for HuggingFace tokenizers.
//!
//! Tokenizers are downloaded from HuggingFace Hub on first use and cached in-memory
//! for subsequent calls. File-level caching is handled by the `hf-hub` crate
//! (defaults to `~/.cache/huggingface/`, configurable via `HF_HOME` env var).

use ahash::AHashMap;
use std::sync::{Arc, RwLock};

use std::sync::LazyLock;

use crate::KreuzbergError;

/// Default model used by [`count_tokens`] when no model is specified.
///
/// `Xenova/gpt-4o` encodes with the o200k_base vocabulary, making it the closest
/// widely-available HuggingFace proxy for GPT-4o and Claude token counts.
pub const DEFAULT_COUNT_TOKENS_MODEL: &str = "Xenova/gpt-4o";

/// Global in-memory cache for loaded tokenizers.
///
/// Keyed by model ID string. Once a tokenizer is loaded and parsed,
/// it's stored here to avoid re-downloading and re-parsing on subsequent calls.
static TOKENIZER_CACHE: LazyLock<RwLock<AHashMap<String, Arc<tokenizers::Tokenizer>>>> =
    LazyLock::new(|| RwLock::new(AHashMap::new()));

/// Get a cached tokenizer or initialize one from HuggingFace Hub.
///
/// Uses a two-phase locking strategy (read lock first, write lock on miss)
/// following the same pattern as the embeddings model cache in `embeddings.rs`.
///
/// # Arguments
///
/// * `model` - HuggingFace model ID (e.g., "Xenova/gpt-4o", "bert-base-uncased")
///
/// # Errors
///
/// Returns an error if the tokenizer cannot be downloaded or parsed.
pub(crate) fn get_or_init_tokenizer(model: &str) -> crate::Result<Arc<tokenizers::Tokenizer>> {
    // Phase 1: try read lock (fast path for cache hits)
    {
        let cache = TOKENIZER_CACHE
            .read()
            .map_err(|e| KreuzbergError::Other(format!("Tokenizer cache read lock poisoned: {}", e)))?;
        if let Some(tok) = cache.get(model) {
            return Ok(Arc::clone(tok));
        }
    }

    // Phase 2: write lock, double-check, then initialize
    let mut cache = TOKENIZER_CACHE
        .write()
        .map_err(|e| KreuzbergError::Other(format!("Tokenizer cache write lock poisoned: {}", e)))?;

    // Double-check after acquiring write lock (another thread may have initialized)
    if let Some(tok) = cache.get(model) {
        return Ok(Arc::clone(tok));
    }

    let tokenizer = tokenizers::Tokenizer::from_pretrained(model, None)
        .map_err(|e| KreuzbergError::validation(format!("Failed to load tokenizer '{}': {}", model, e)))?;

    let arc = Arc::new(tokenizer);
    cache.insert(model.to_string(), Arc::clone(&arc));
    Ok(arc)
}

/// Count the number of tokens in `text` for the given HuggingFace tokenizer model.
///
/// Reuses the global in-memory tokenizer cache — the tokenizer is downloaded and
/// parsed only on the first call for each model, then served from memory for all
/// subsequent calls. File-level caching is handled by the `hf-hub` crate (defaults
/// to `~/.cache/huggingface/`).
///
/// # Arguments
///
/// * `text`  - The text to tokenize.
/// * `model` - HuggingFace model ID to use for tokenization.  Pass `None` to use
///   the default model ([`DEFAULT_COUNT_TOKENS_MODEL`], `"Xenova/gpt-4o"`), which
///   encodes with the o200k_base vocabulary — the closest widely-available proxy
///   for GPT-4o and Claude token counts.
///
/// # Returns
///
/// The number of tokens produced by the tokenizer.  If the tokenizer cannot be
/// loaded (e.g. network unavailable, model not found), falls back to a
/// whitespace-split heuristic so the function never panics or propagates an error.
///
/// # Example
///
/// ```rust,no_run
/// use kreuzberg::chunking::count_tokens;
///
/// let n = count_tokens("Hello, world!", None);
/// assert!(n > 0);
///
/// let n_gpt4 = count_tokens("Hello, world!", Some("Xenova/gpt-4o"));
/// assert!(n_gpt4 > 0);
/// ```
///
/// # Note
///
/// This function is intentionally excluded from language bindings (alef-skipped) —
/// it is a Rust-only utility that relies on the cached tokenizer infrastructure.
#[cfg_attr(alef, alef(skip))]
pub fn count_tokens(text: &str, model: Option<&str>) -> usize {
    let model = model.unwrap_or(DEFAULT_COUNT_TOKENS_MODEL);
    match get_or_init_tokenizer(model) {
        Ok(tokenizer) => match tokenizer.encode(text, false) {
            Ok(encoding) => encoding.len(),
            Err(_) => whitespace_token_estimate(text),
        },
        Err(_) => whitespace_token_estimate(text),
    }
}

/// Heuristic fallback: split on whitespace and count non-empty tokens.
///
/// Used when the HuggingFace tokenizer cannot be loaded.
fn whitespace_token_estimate(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_returns_same_instance() {
        // This test requires network access to download a tokenizer.
        // Skip in CI by checking for a specific env var.
        if std::env::var("CI").is_ok() {
            return;
        }

        let model = "bert-base-uncased";
        let tok1 = get_or_init_tokenizer(model).unwrap();
        let tok2 = get_or_init_tokenizer(model).unwrap();

        // Same Arc instance (pointer equality)
        assert!(Arc::ptr_eq(&tok1, &tok2));
    }

    /// Verify that `count_tokens` returns a meaningful non-zero count and that the
    /// `None`-model path resolves to [`DEFAULT_COUNT_TOKENS_MODEL`].
    ///
    /// This test requires network access on the first run to download the tokenizer.
    /// Skipped in CI (no network) via the `CI` environment variable.
    #[test]
    fn test_count_tokens_none_defaults_to_gpt4o_and_returns_nonzero() {
        if std::env::var("CI").is_ok() {
            return;
        }

        let text = "Hello, world! This is a test sentence for token counting.";

        // None resolves to DEFAULT_COUNT_TOKENS_MODEL ("Xenova/gpt-4o")
        let count_via_none = count_tokens(text, None);
        assert!(
            count_via_none > 0,
            "count_tokens(text, None) must return a non-zero count"
        );

        // Explicit model must return the same value (same model, same cache entry)
        let count_via_explicit = count_tokens(text, Some(DEFAULT_COUNT_TOKENS_MODEL));
        assert_eq!(
            count_via_none, count_via_explicit,
            "None and Some(DEFAULT_COUNT_TOKENS_MODEL) must produce the same count"
        );
    }

    /// Verify that the fallback heuristic (whitespace split) is used when the
    /// tokenizer cannot be loaded (bogus model id), and that the function never
    /// panics or returns zero for non-empty text.
    #[test]
    fn test_count_tokens_falls_back_gracefully_on_invalid_model() {
        let text = "six distinct whitespace separated words here";
        // An obviously invalid model ID — `get_or_init_tokenizer` will fail.
        let count = count_tokens(text, Some("__invalid_model_that_does_not_exist__"));
        // Fallback: split_whitespace gives 6 tokens for the sentence above.
        assert_eq!(count, 6, "fallback whitespace estimator should count 6 words");
    }

    /// Ensure the whitespace heuristic itself handles edge cases.
    #[test]
    fn test_whitespace_token_estimate_edge_cases() {
        assert_eq!(whitespace_token_estimate(""), 0);
        assert_eq!(whitespace_token_estimate("   "), 0);
        assert_eq!(whitespace_token_estimate("one"), 1);
        assert_eq!(whitespace_token_estimate("one two three"), 3);
    }
}
