fn main() {
    println!("cargo::rustc-check-cfg=cfg(coverage)");

    // `inference_ort` marks builds where the ONNX Runtime engine is linked, so the
    // inference seam ([`crate::inference`]) can compile-time select ONNX Runtime
    // over tract. Every ORT-backed capability enables `ort-bundled` (the default)
    // or opts into `ort-dynamic`; either implies the `ort` crate is present. On
    // no-ORT targets (WASM, Android x86_64) neither is set, so the seam falls back
    // to the pure-Rust tract backend.
    println!("cargo::rustc-check-cfg=cfg(inference_ort)");
    if std::env::var_os("CARGO_FEATURE_ORT_BUNDLED").is_some()
        || std::env::var_os("CARGO_FEATURE_ORT_DYNAMIC").is_some()
    {
        println!("cargo::rustc-cfg=inference_ort");
    }

    // `auto_rotate` marks builds where the document-orientation capability is
    // present regardless of engine: the ORT-backed `auto-rotate` feature or the
    // pure-Rust `auto-rotate-tract` feature (no-ORT targets). Consumer sites gate
    // on this cfg so they need not enumerate every engine variant; `default_backend`
    // (via `inference_ort`) still picks the concrete engine.
    println!("cargo::rustc-check-cfg=cfg(auto_rotate)");
    if std::env::var_os("CARGO_FEATURE_AUTO_ROTATE").is_some()
        || std::env::var_os("CARGO_FEATURE_AUTO_ROTATE_TRACT").is_some()
    {
        println!("cargo::rustc-cfg=auto_rotate");
    }

    if std::env::var_os("CARGO_FEATURE_ORT_BUNDLED").is_some()
        && std::env::var_os("CARGO_FEATURE_ORT_DYNAMIC").is_some()
    {
        println!(
            "cargo::warning=features 'ort-bundled' and 'ort-dynamic' are both enabled; bundled ORT remains the default unless dynamic ORT is explicitly selected at runtime"
        );
    }

    // `layout_detection` marks builds where the layout-detection capability is
    // present regardless of engine: the ORT-backed `layout-detection` feature (RT-DETR,
    // YOLO, TATR, SLANeXT, PP-DocLayout-V3) or the pure-Rust `layout-tract` feature
    // (RT-DETR + table classifier only; no-ORT targets). Consumer sites gate on this
    // cfg so they need not enumerate every engine variant; `default_backend` (via
    // `inference_ort`) still picks the concrete engine, and ORT-only model files
    // (session.rs, tatr.rs, slanet.rs, yolo.rs, pp_doclayout_v3.rs) stay gated on the
    // literal `layout-detection` feature so they never compile under tract.
    println!("cargo::rustc-check-cfg=cfg(layout_detection)");
    if std::env::var_os("CARGO_FEATURE_LAYOUT_DETECTION").is_some()
        || std::env::var_os("CARGO_FEATURE_LAYOUT_TRACT").is_some()
    {
        println!("cargo::rustc-cfg=layout_detection");
    }
}
