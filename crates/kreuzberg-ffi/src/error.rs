//! Centralized error codes for Kreuzberg FFI bindings.
//!
//! This module defines the authoritative error codes used across all language bindings
//! (Python, Ruby, Go, Java, TypeScript, C#). All bindings should reference these codes
//! rather than maintaining separate definitions.
//!
//! # Error Code Mapping
//!
//! Each variant maps to a specific error type encountered during document extraction:
//!
//! - **Validation (0)**: Input validation errors (invalid config, parameters)
//! - **Parsing (1)**: Document format errors (corrupt files, unsupported features)
//! - **Ocr (2)**: OCR processing failures (backend errors, image issues)
//! - **MissingDependency (3)**: Required system dependencies not found (tesseract, pandoc)
//! - **Io (4)**: File system and I/O errors (permissions, disk full)
//! - **Plugin (5)**: Plugin registration/execution errors
//! - **UnsupportedFormat (6)**: Unsupported MIME type or file format
//! - **Internal (7)**: Internal library errors (should rarely occur)
//!
//! # Usage in Bindings
//!
//! **Python** (kreuzberg/exceptions.py):
//! ```python
//! class ErrorCode(IntEnum):
//!     VALIDATION = 0
//!     PARSING = 1
//!     OCR = 2
//!     MISSING_DEPENDENCY = 3
//!     IO = 4
//!     PLUGIN = 5
//!     UNSUPPORTED_FORMAT = 6
//!     INTERNAL = 7
//! ```
//!
//! **Ruby** (packages/ruby/lib/kreuzberg.rb):
//! ```ruby
//! module Kreuzberg
//!   class ErrorCode
//!     VALIDATION = 0
//!     PARSING = 1
//!     # ... etc
//!   end
//! end
//! ```
//!
//! **Go** (packages/go/v4/errors.go):
//! ```go
//! const (
//!     ValidationError int32 = 0
//!     ParsingError int32 = 1
//!     // ... etc
//! )
//! ```
//!
//! # FFI Exports
//!
//! This module exports FFI-safe functions for binding libraries to query error codes:
//!
//! - `kreuzberg_error_code_validation()` -> 0
//! - `kreuzberg_error_code_parsing()` -> 1
//! - `kreuzberg_error_code_ocr()` -> 2
//! - `kreuzberg_error_code_missing_dependency()` -> 3
//! - `kreuzberg_error_code_io()` -> 4
//! - `kreuzberg_error_code_plugin()` -> 5
//! - `kreuzberg_error_code_unsupported_format()` -> 6
//! - `kreuzberg_error_code_internal()` -> 7
//! - `kreuzberg_error_code_count()` -> 8
//! - `kreuzberg_error_code_name(code: u32)` -> *const c_char (error name)
//!
//! # Thread Safety
//!
//! All functions are thread-safe and have no runtime overhead (compile-time constants).

use std::os::raw::c_char;

#[cfg(test)]
use std::ffi::CStr;

/// Centralized error codes for all Kreuzberg bindings.
///
/// These codes are the single source of truth for error classification across
/// all language bindings. Do not introduce new error codes without updating
/// this enum and regenerating bindings.
///
/// # Repr and Stability
///
/// - Uses `#[repr(u32)]` for C ABI compatibility
/// - Error codes are guaranteed stable (0-7, never changing)
/// - Can be safely cast to `int32_t` in C/C++ code
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Input validation error (invalid config, parameters, paths)
    Validation = 0,
    /// Document parsing error (corrupt files, unsupported format features)
    Parsing = 1,
    /// OCR processing error (backend failures, image quality issues)
    Ocr = 2,
    /// Missing system dependency (tesseract not found, pandoc not installed)
    MissingDependency = 3,
    /// File system I/O error (permissions, disk full, file not found)
    Io = 4,
    /// Plugin registration or execution error
    Plugin = 5,
    /// Unsupported MIME type or file format
    UnsupportedFormat = 6,
    /// Internal library error (indicates a bug, should rarely occur)
    Internal = 7,
}

impl ErrorCode {
    /// Returns the human-readable name for this error code.
    ///
    /// Used for debugging and logging. Names match the enum variant names
    /// in lowercase.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(ErrorCode::Validation.name(), "validation");
    /// assert_eq!(ErrorCode::Ocr.name(), "ocr");
    /// ```
    #[inline]
    pub fn name(self) -> &'static str {
        match self {
            ErrorCode::Validation => "validation",
            ErrorCode::Parsing => "parsing",
            ErrorCode::Ocr => "ocr",
            ErrorCode::MissingDependency => "missing_dependency",
            ErrorCode::Io => "io",
            ErrorCode::Plugin => "plugin",
            ErrorCode::UnsupportedFormat => "unsupported_format",
            ErrorCode::Internal => "internal",
        }
    }

    /// Returns a brief description of the error code.
    ///
    /// Used for user-facing error messages and documentation.
    #[inline]
    pub fn description(self) -> &'static str {
        match self {
            ErrorCode::Validation => "Input validation error",
            ErrorCode::Parsing => "Document parsing error",
            ErrorCode::Ocr => "OCR processing error",
            ErrorCode::MissingDependency => "Missing system dependency",
            ErrorCode::Io => "File system I/O error",
            ErrorCode::Plugin => "Plugin error",
            ErrorCode::UnsupportedFormat => "Unsupported format",
            ErrorCode::Internal => "Internal library error",
        }
    }

    /// Converts from numeric error code to enum variant.
    ///
    /// Returns `None` if the code is outside the valid range [0, 7].
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(ErrorCode::from_code(0), Some(ErrorCode::Validation));
    /// assert_eq!(ErrorCode::from_code(2), Some(ErrorCode::Ocr));
    /// assert_eq!(ErrorCode::from_code(99), None);
    /// ```
    #[inline]
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(ErrorCode::Validation),
            1 => Some(ErrorCode::Parsing),
            2 => Some(ErrorCode::Ocr),
            3 => Some(ErrorCode::MissingDependency),
            4 => Some(ErrorCode::Io),
            5 => Some(ErrorCode::Plugin),
            6 => Some(ErrorCode::UnsupportedFormat),
            7 => Some(ErrorCode::Internal),
            _ => None,
        }
    }

    /// Checks if a numeric code is valid (within [0, 7]).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert!(ErrorCode::is_valid(0));
    /// assert!(ErrorCode::is_valid(7));
    /// assert!(!ErrorCode::is_valid(8));
    /// ```
    #[inline]
    pub fn is_valid(code: u32) -> bool {
        code <= 7
    }
}

// FFI exports - these functions provide C-compatible access to error codes.
// They allow bindings to reference error codes without hardcoding numeric values.

/// Returns the validation error code (0).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_validation(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_validation() -> u32 {
    ErrorCode::Validation as u32
}

/// Returns the parsing error code (1).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_parsing(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_parsing() -> u32 {
    ErrorCode::Parsing as u32
}

/// Returns the OCR error code (2).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_ocr(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_ocr() -> u32 {
    ErrorCode::Ocr as u32
}

/// Returns the missing dependency error code (3).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_missing_dependency(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_missing_dependency() -> u32 {
    ErrorCode::MissingDependency as u32
}

/// Returns the I/O error code (4).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_io(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_io() -> u32 {
    ErrorCode::Io as u32
}

/// Returns the plugin error code (5).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_plugin(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_plugin() -> u32 {
    ErrorCode::Plugin as u32
}

/// Returns the unsupported format error code (6).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_unsupported_format(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_unsupported_format() -> u32 {
    ErrorCode::UnsupportedFormat as u32
}

/// Returns the internal error code (7).
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_internal(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_internal() -> u32 {
    ErrorCode::Internal as u32
}

/// Returns the total count of valid error codes.
///
/// Currently 8 error codes (0-7). This helps bindings validate error codes.
///
/// # C Signature
///
/// ```c
/// uint32_t kreuzberg_error_code_count(void);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_count() -> u32 {
    8
}

/// Returns the name of an error code as a C string.
///
/// # Arguments
///
/// - `code`: Numeric error code (0-7)
///
/// # Returns
///
/// Pointer to a null-terminated C string with the error name (e.g., "validation", "ocr").
/// Returns a pointer to "unknown" if the code is invalid.
///
/// The returned pointer is valid for the lifetime of the program and should not be freed.
///
/// # Examples
///
/// ```c
/// const char* name = kreuzberg_error_code_name(0);
/// printf("%s\n", name);  // prints: validation
/// ```
///
/// # C Signature
///
/// ```c
/// const char* kreuzberg_error_code_name(uint32_t code);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_name(code: u32) -> *const c_char {
    match ErrorCode::from_code(code) {
        Some(err_code) => {
            let name = err_code.name();
            // SAFETY: name() returns &'static str from a match statement on valid variants.
            // All static strings are guaranteed to be valid C strings (null-terminated).
            name.as_ptr() as *const c_char
        }
        None => {
            // SAFETY: "unknown" is a string literal and is valid for the lifetime of the program.
            "unknown".as_ptr() as *const c_char
        }
    }
}

/// Returns the description of an error code as a C string.
///
/// # Arguments
///
/// - `code`: Numeric error code (0-7)
///
/// # Returns
///
/// Pointer to a null-terminated C string with a description (e.g., "Input validation error").
/// Returns a pointer to "Unknown error code" if the code is invalid.
///
/// The returned pointer is valid for the lifetime of the program and should not be freed.
///
/// # C Signature
///
/// ```c
/// const char* kreuzberg_error_code_description(uint32_t code);
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn kreuzberg_error_code_description(code: u32) -> *const c_char {
    match ErrorCode::from_code(code) {
        Some(err_code) => {
            let desc = err_code.description();
            // SAFETY: description() returns &'static str. Same reasoning as name().
            desc.as_ptr() as *const c_char
        }
        None => {
            // SAFETY: string literal, valid for program lifetime
            "Unknown error code".as_ptr() as *const c_char
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_values() {
        assert_eq!(ErrorCode::Validation as u32, 0);
        assert_eq!(ErrorCode::Parsing as u32, 1);
        assert_eq!(ErrorCode::Ocr as u32, 2);
        assert_eq!(ErrorCode::MissingDependency as u32, 3);
        assert_eq!(ErrorCode::Io as u32, 4);
        assert_eq!(ErrorCode::Plugin as u32, 5);
        assert_eq!(ErrorCode::UnsupportedFormat as u32, 6);
        assert_eq!(ErrorCode::Internal as u32, 7);
    }

    #[test]
    fn test_error_code_names() {
        assert_eq!(ErrorCode::Validation.name(), "validation");
        assert_eq!(ErrorCode::Parsing.name(), "parsing");
        assert_eq!(ErrorCode::Ocr.name(), "ocr");
        assert_eq!(ErrorCode::MissingDependency.name(), "missing_dependency");
        assert_eq!(ErrorCode::Io.name(), "io");
        assert_eq!(ErrorCode::Plugin.name(), "plugin");
        assert_eq!(ErrorCode::UnsupportedFormat.name(), "unsupported_format");
        assert_eq!(ErrorCode::Internal.name(), "internal");
    }

    #[test]
    fn test_error_code_descriptions() {
        assert_eq!(ErrorCode::Validation.description(), "Input validation error");
        assert_eq!(ErrorCode::Parsing.description(), "Document parsing error");
        assert_eq!(ErrorCode::Ocr.description(), "OCR processing error");
        assert_eq!(ErrorCode::MissingDependency.description(), "Missing system dependency");
        assert_eq!(ErrorCode::Io.description(), "File system I/O error");
        assert_eq!(ErrorCode::Plugin.description(), "Plugin error");
        assert_eq!(ErrorCode::UnsupportedFormat.description(), "Unsupported format");
        assert_eq!(ErrorCode::Internal.description(), "Internal library error");
    }

    #[test]
    fn test_from_code_valid() {
        assert_eq!(ErrorCode::from_code(0), Some(ErrorCode::Validation));
        assert_eq!(ErrorCode::from_code(1), Some(ErrorCode::Parsing));
        assert_eq!(ErrorCode::from_code(2), Some(ErrorCode::Ocr));
        assert_eq!(ErrorCode::from_code(3), Some(ErrorCode::MissingDependency));
        assert_eq!(ErrorCode::from_code(4), Some(ErrorCode::Io));
        assert_eq!(ErrorCode::from_code(5), Some(ErrorCode::Plugin));
        assert_eq!(ErrorCode::from_code(6), Some(ErrorCode::UnsupportedFormat));
        assert_eq!(ErrorCode::from_code(7), Some(ErrorCode::Internal));
    }

    #[test]
    fn test_from_code_invalid() {
        assert_eq!(ErrorCode::from_code(8), None);
        assert_eq!(ErrorCode::from_code(99), None);
        assert_eq!(ErrorCode::from_code(u32::MAX), None);
    }

    #[test]
    fn test_is_valid() {
        for code in 0..=7 {
            assert!(ErrorCode::is_valid(code), "Code {} should be valid", code);
        }

        assert!(!ErrorCode::is_valid(8));
        assert!(!ErrorCode::is_valid(99));
        assert!(!ErrorCode::is_valid(u32::MAX));
    }

    #[test]
    fn test_error_code_count() {
        assert_eq!(kreuzberg_error_code_count(), 8);
    }

    #[test]
    fn test_ffi_error_code_functions() {
        assert_eq!(kreuzberg_error_code_validation(), 0);
        assert_eq!(kreuzberg_error_code_parsing(), 1);
        assert_eq!(kreuzberg_error_code_ocr(), 2);
        assert_eq!(kreuzberg_error_code_missing_dependency(), 3);
        assert_eq!(kreuzberg_error_code_io(), 4);
        assert_eq!(kreuzberg_error_code_plugin(), 5);
        assert_eq!(kreuzberg_error_code_unsupported_format(), 6);
        assert_eq!(kreuzberg_error_code_internal(), 7);
    }

    #[test]
    fn test_error_code_name_ffi() {
        // SAFETY: test only, the returned pointer is valid
        unsafe {
            let name = CStr::from_ptr(kreuzberg_error_code_name(0)).to_str().unwrap();
            assert_eq!(name, "validation");

            let name = CStr::from_ptr(kreuzberg_error_code_name(2)).to_str().unwrap();
            assert_eq!(name, "ocr");

            let name = CStr::from_ptr(kreuzberg_error_code_name(99)).to_str().unwrap();
            assert_eq!(name, "unknown");
        }
    }

    #[test]
    fn test_error_code_description_ffi() {
        // SAFETY: test only, the returned pointer is valid
        unsafe {
            let desc = CStr::from_ptr(kreuzberg_error_code_description(0)).to_str().unwrap();
            assert_eq!(desc, "Input validation error");

            let desc = CStr::from_ptr(kreuzberg_error_code_description(4)).to_str().unwrap();
            assert_eq!(desc, "File system I/O error");

            let desc = CStr::from_ptr(kreuzberg_error_code_description(99)).to_str().unwrap();
            assert_eq!(desc, "Unknown error code");
        }
    }

    #[test]
    fn test_error_code_round_trip() {
        for code in 0u32..=7 {
            let err = ErrorCode::from_code(code).unwrap();
            assert_eq!(err as u32, code);

            // Verify name and description are non-empty
            assert!(!err.name().is_empty());
            assert!(!err.description().is_empty());
        }
    }

    #[test]
    fn test_error_code_copy_clone() {
        let err = ErrorCode::Validation;
        let err_copy = err;
        let err_clone = err.clone();

        assert_eq!(err, err_copy);
        assert_eq!(err, err_clone);
    }

    #[test]
    fn test_error_code_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ErrorCode::Validation);
        set.insert(ErrorCode::Parsing);
        set.insert(ErrorCode::Validation); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&ErrorCode::Validation));
        assert!(set.contains(&ErrorCode::Parsing));
    }

    #[test]
    fn test_error_code_debug() {
        let err = ErrorCode::Ocr;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Ocr"));
    }
}
