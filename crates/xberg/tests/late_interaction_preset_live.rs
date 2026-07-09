//! Live validation for the self-hosted ColBERT late-interaction preset.
//!
//! Downloads the preset's ONNX model from `xberg-io/late-interaction-models`
//! and runs real inference, asserting a non-empty, well-formed multi-vector
//! embedding with finite values. Opt out on offline dev with `XBERG_SKIP_LIVE_HF=1`.

#![cfg(feature = "late-interaction")]

use xberg::core::config::{LateInteractionConfig, LateInteractionModelType};

fn should_skip() -> bool {
    std::env::var("XBERG_SKIP_LIVE_HF").is_ok()
}

#[test]
fn colbert_small_v1_multi_vector_embeds() {
    if should_skip() {
        eprintln!("XBERG_SKIP_LIVE_HF=1, skipping");
        return;
    }
    let preset = xberg::get_late_interaction_preset("colbert").expect("preset must exist");
    assert_eq!(preset.model_repo, "xberg-io/late-interaction-models");
    assert_eq!(preset.model_file, "colbert-small-v1/model.onnx");
    assert_eq!(preset.dim, 96);

    let config = LateInteractionConfig {
        model: LateInteractionModelType::Preset {
            name: "colbert".to_string(),
        },
        ..Default::default()
    };
    let out = xberg::embed_multi_vector(
        vec!["the quick brown fox jumps over the lazy dog".to_string()],
        &config,
        false,
    )
    .expect("late-interaction embed must succeed");

    assert_eq!(out.len(), 1, "one multi-vector embedding per input");
    let mv = &out[0];
    assert!(mv.num_tokens > 0, "multi-vector embedding must have at least one token");
    assert_eq!(mv.dim, 96, "colbert-small-v1 embeddings are 96-dimensional");
    assert!(mv.is_well_formed(), "data length must equal num_tokens * dim");
    assert_eq!(
        mv.data.len() % (mv.dim as usize),
        0,
        "flat buffer length must be a multiple of dim"
    );
    assert!(!mv.data.is_empty(), "multi-vector data must be non-empty");
    assert!(
        mv.data.iter().all(|v| v.is_finite()),
        "all per-token embedding values must be finite"
    );
}

#[test]
fn gte_moderncolbert_multi_vector_embeds() {
    if should_skip() {
        eprintln!("XBERG_SKIP_LIVE_HF=1, skipping");
        return;
    }
    let preset = xberg::get_late_interaction_preset("gte-moderncolbert").expect("preset must exist");
    assert_eq!(preset.model_repo, "xberg-io/late-interaction-models");
    assert_eq!(preset.model_file, "gte-moderncolbert-v1/model.onnx");
    assert_eq!(preset.dim, 128);

    let config = LateInteractionConfig {
        model: LateInteractionModelType::Preset {
            name: "gte-moderncolbert".to_string(),
        },
        ..Default::default()
    };
    let out = xberg::embed_multi_vector(
        vec!["the quick brown fox jumps over the lazy dog".to_string()],
        &config,
        false,
    )
    .expect("late-interaction embed must succeed");

    assert_eq!(out.len(), 1, "one multi-vector embedding per input");
    let mv = &out[0];
    assert!(mv.num_tokens > 0, "multi-vector embedding must have at least one token");
    assert_eq!(mv.dim, 128, "gte-moderncolbert embeddings are 128-dimensional");
    assert!(mv.is_well_formed(), "data length must equal num_tokens * dim");
    assert_eq!(
        mv.data.len() % (mv.dim as usize),
        0,
        "flat buffer length must be a multiple of dim"
    );
    assert!(!mv.data.is_empty(), "multi-vector data must be non-empty");
    assert!(
        mv.data.iter().all(|v| v.is_finite()),
        "all per-token embedding values must be finite"
    );
}
