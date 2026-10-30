---
status: accepted
date: 2025-12-09
decision-makers: ["Na'aman Hirschfeld"]
---

# Sync-only WASM beside an async Tokio core

## Context and Problem Statement

The core crate's extraction paths were async on Tokio, which the crate always pulled in. A WebAssembly binding for
browsers, Deno and Cloudflare Workers was in progress and landed four days later (872e1202cf, "feat(wasm): add WASM
bindings implementation", 2025-12-13). That target needed the core to compile and extract without the Tokio
runtime.

## Considered Options

* Keep Tokio behind a default `tokio-runtime` feature, and give the WASM build synchronous internal extraction paths

No alternatives were recorded.

## Decision Outcome

Chosen option: "Keep Tokio behind a default `tokio-runtime` feature, and give the WASM build synchronous internal
extraction paths", because native callers keep the async API unchanged while the WASM crate builds the same core with
the runtime compiled out.

954236fd24 (2025-12-09) made `tokio-runtime` a default feature, and the async extractor paths were gated on it the
same day (ebc6d8c0cd). Today:

* Native builds get Tokio by default. Extraction paths are fully async, CPU-bound work runs in `spawn_blocking`, and
  timeouts use `tokio::time::timeout` (`.ai-rulez/rules/async-and-concurrency.md`).
* `crates/xberg-wasm` depends on the core with `default-features = false` and the `wasm-target` aggregate, which does
  not include `tokio-runtime`.
* Every WASM-capable built-in extractor implements the crate-private `SyncExtractor` trait. Public callers still use
  `extract` and `extract_batch`.

### Consequences

* Good, because one crate serves both the native async bindings and the browser and edge runtimes.
* Bad, because an extractor that only has an async path is unavailable on WASM, and out-of-crate plugins cannot
  implement `SyncExtractor`.
* Bad, because a feature that needs Tokio, such as `pdf-pdfium`, `excel` or `sqlite`, must list `tokio-runtime`
  itself, which spreads 362 `feature = "tokio-runtime"` gates across `crates/xberg/src`.

### Confirmation

* `crates/xberg/Cargo.toml:116` sets `default = ["tokio-runtime", "simd-utf8"]`, `:127` defines
  `tokio-runtime = ["dep:tokio"]`, and the `wasm-target` aggregate at `:835` leaves it out.
* `crates/xberg-wasm/Cargo.toml:83` depends on `xberg` with `default-features = false` and `wasm-target`.
* `crates/xberg/src/extractors/mod.rs:41` declares `pub(crate) trait SyncExtractor`.
* The `ci-rust.yaml` job "Narrow feature leg (wasm-target, wasm32)" runs
  `cargo check --locked -p xberg --no-default-features --features wasm-target --target wasm32-unknown-unknown`
  (`.github/workflows/ci-rust.yaml:686`).
