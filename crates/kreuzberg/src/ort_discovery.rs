//! ONNX Runtime library auto-discovery and execution provider configuration.
//!
//! Scans common installation paths and sets `ORT_DYLIB_PATH` so the `ort` crate
//! can find `libonnxruntime` via `dlopen`. Called once at init time.
//!
//! Also provides [`apply_execution_providers`] for configuring GPU acceleration
//! on ORT session builders across all subsystems (layout, embeddings, OCR, etc.).

#[cfg(not(feature = "ort-bundled"))]
use std::sync::Once;

#[cfg(not(feature = "ort-bundled"))]
static ORT_INIT: Once = Once::new();

/// Ensure ONNX Runtime is discoverable. Safe to call multiple times (no-op after first).
///
/// When the `ort-bundled` feature is enabled the ORT binaries are embedded via the
/// official Microsoft release and no system library search is needed.
pub fn ensure_ort_available() {
    #[cfg(feature = "ort-bundled")]
    {
        tracing::debug!("ONNX Runtime is bundled; skipping system library discovery");
    }

    #[cfg(not(feature = "ort-bundled"))]
    ORT_INIT.call_once(|| {
        if let Err(msg) = try_discover_ort() {
            tracing::warn!("ONNX Runtime not found: {msg}");
        }
    });
}

#[cfg(not(feature = "ort-bundled"))]
fn try_discover_ort() -> Result<(), &'static str> {
    // Already set and valid?
    if let Ok(path) = std::env::var("ORT_DYLIB_PATH")
        && std::path::Path::new(&path).exists()
    {
        return Ok(());
    }

    let candidates: &[&str] = platform_candidates();

    for path in candidates {
        if std::path::Path::new(path).exists() {
            // SAFETY: single-threaded inside Once::call_once
            #[allow(unsafe_code)]
            unsafe {
                std::env::set_var("ORT_DYLIB_PATH", path);
            }
            tracing::debug!("Auto-discovered ONNX Runtime at {path}");
            return Ok(());
        }
    }

    Err("ONNX Runtime library not found in common installation paths")
}

#[cfg(all(not(feature = "ort-bundled"), target_os = "macos"))]
fn platform_candidates() -> &'static [&'static str] {
    &[
        "/opt/homebrew/lib/libonnxruntime.dylib",
        "/usr/local/lib/libonnxruntime.dylib",
    ]
}

#[cfg(all(not(feature = "ort-bundled"), target_os = "linux"))]
fn platform_candidates() -> &'static [&'static str] {
    &[
        "/usr/lib/libonnxruntime.so",
        "/usr/local/lib/libonnxruntime.so",
        "/usr/lib/x86_64-linux-gnu/libonnxruntime.so",
        "/usr/lib/aarch64-linux-gnu/libonnxruntime.so",
    ]
}

#[cfg(all(not(feature = "ort-bundled"), target_os = "windows"))]
fn platform_candidates() -> &'static [&'static str] {
    &[
        "C:\\Program Files\\onnxruntime\\bin\\onnxruntime.dll",
        "C:\\Windows\\System32\\onnxruntime.dll",
    ]
}

#[cfg(all(
    not(feature = "ort-bundled"),
    not(any(target_os = "macos", target_os = "linux", target_os = "windows"))
))]
fn platform_candidates() -> &'static [&'static str] {
    &[]
}

/// Apply execution providers to an ORT session builder based on [`AccelerationConfig`].
///
/// Shared by all ORT consumers (layout detection, embeddings, PaddleOCR, doc orientation).
/// ORT silently falls back to CPU if the requested EP is unavailable at runtime.
///
/// [`AccelerationConfig`]: crate::core::config::acceleration::AccelerationConfig
#[cfg(any(
    feature = "layout-detection",
    feature = "embeddings",
    feature = "paddle-ocr",
    feature = "auto-rotate"
))]
pub fn apply_execution_providers(
    builder: ort::session::builder::SessionBuilder,
    accel: Option<&crate::core::config::acceleration::AccelerationConfig>,
) -> Result<ort::session::builder::SessionBuilder, ort::Error> {
    use crate::core::config::acceleration::ExecutionProviderType;

    let provider = accel.map(|a| &a.provider).unwrap_or(&ExecutionProviderType::Auto);
    let device_id = accel.map(|a| a.device_id).unwrap_or(0);

    let builder = match provider {
        ExecutionProviderType::Cpu => {
            tracing::debug!("ORT session: CPU execution provider (explicit)");
            builder
        }
        ExecutionProviderType::CoreMl => {
            tracing::debug!("ORT session: CoreML execution provider requested");
            builder
                .with_execution_providers([ort::ep::CoreML::default().build()])
                .map_err(|e| ort::Error::new(e.message()))?
        }
        ExecutionProviderType::Cuda => {
            tracing::debug!(device_id, "ORT session: CUDA execution provider requested");
            builder
                .with_execution_providers([ort::ep::CUDA::default().with_device_id(device_id as i32).build()])
                .map_err(|e| ort::Error::new(e.message()))?
        }
        ExecutionProviderType::TensorRt => {
            tracing::debug!(device_id, "ORT session: TensorRT execution provider requested");
            builder
                .with_execution_providers([ort::ep::TensorRT::default().with_device_id(device_id as i32).build()])
                .map_err(|e| ort::Error::new(e.message()))?
        }
        ExecutionProviderType::Auto => {
            #[cfg(target_os = "macos")]
            let builder = {
                tracing::debug!("ORT session: auto-selected CoreML (macOS)");
                builder
                    .with_execution_providers([ort::ep::CoreML::default().build()])
                    .map_err(|e| ort::Error::new(e.message()))?
            };
            #[cfg(target_os = "linux")]
            let builder = {
                tracing::debug!("ORT session: auto-selected CUDA (Linux)");
                builder
                    .with_execution_providers([ort::ep::CUDA::default().build()])
                    .map_err(|e| ort::Error::new(e.message()))?
            };
            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            let builder = {
                tracing::debug!("ORT session: auto-selected CPU (no platform EP)");
                builder
            };
            builder
        }
    };

    Ok(builder)
}
