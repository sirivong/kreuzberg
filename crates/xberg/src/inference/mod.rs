//! Engine-neutral inference seam.
//!
//! xberg runs its ONNX models — layout detection, table classification, document
//! orientation, and more — through a small backend abstraction rather than
//! calling an engine directly. Two traits split the concerns:
//!
//! - [`InferenceBackend`] loads an `.onnx` artifact into a session.
//! - [`InferenceSession`] runs it, exchanging [`InferenceTensor`] values.
//!
//! [`default_backend`] selects the engine at compile time. Today that is always
//! ONNX Runtime ([`OrtBackend`]); a pure-Rust `tract` backend is added on no-ORT
//! targets (WASM, Android) in a later phase, at which point this selector becomes
//! `#[cfg]`-conditional. Until then the seam is a behavior-preserving refactor:
//! migrated models produce byte-identical ORT output.
//!
//! Not part of the language-binding surface — the whole module is `pub(crate)`
//! and its files are absent from `alef.toml` sources, so the generator emits
//! nothing for it.
//!
//! Since v5.0.0 (issue #1275).

mod backend;
mod ort_backend;
mod tensor;

pub(crate) use backend::{InferenceBackend, InferenceSession};
pub(crate) use tensor::InferenceTensor;

/// Construct the default inference backend for this build.
///
/// Native builds return the ONNX Runtime backend. When the pure-Rust `tract`
/// engine lands (issue #1275, later phase) this becomes a compile-time choice:
/// ORT where it links, tract on no-ORT targets.
pub(crate) fn default_backend() -> Box<dyn InferenceBackend> {
    Box::new(ort_backend::OrtBackend::new())
}
