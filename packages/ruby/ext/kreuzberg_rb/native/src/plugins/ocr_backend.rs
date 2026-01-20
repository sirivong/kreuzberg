//! OCR backend plugin registration and management

use crate::error_handling::{kreuzberg_error, runtime_error};
use magnus::{Error, Value};
use kreuzberg::plugins::{
    unregister_ocr_backend as kz_unregister_ocr_backend,
    list_ocr_backends as kz_list_ocr_backends,
    clear_ocr_backends as kz_clear_ocr_backends,
};

/// Register an OCR backend plugin
pub fn register_ocr_backend(_name: String, _backend: Value) -> Result<(), Error> {
    // OCR backend registration would be implemented here
    // For now, return placeholder
    Err(runtime_error("OCR backend registration not yet implemented"))
}

/// Unregister an OCR backend
pub fn unregister_ocr_backend(_name: String) -> Result<(), Error> {
    kz_unregister_ocr_backend(_name.as_str())
        .map_err(kreuzberg_error)
}

/// List registered OCR backends
pub fn list_ocr_backends() -> Result<Vec<String>, Error> {
    kz_list_ocr_backends()
        .map_err(kreuzberg_error)
}

/// Clear all OCR backends
pub fn clear_ocr_backends() -> Result<(), Error> {
    kz_clear_ocr_backends()
        .map_err(kreuzberg_error)
}
