//! Framework size measurement
//!
//! Measures the installation footprint of document extraction frameworks.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Information about a framework's disk size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkSize {
    /// Size in bytes
    pub size_bytes: u64,
    /// Method used to measure (pip_package, npm_package, binary_size, jar_size, etc.)
    pub method: String,
    /// Human-readable description
    pub description: String,
    /// Whether this is from actual measurement or an estimate
    #[serde(default)]
    pub estimated: bool,
}

/// Framework size measurement results
pub type FrameworkSizes = HashMap<String, FrameworkSize>;

/// Known frameworks with their measurement methods and descriptions
const FRAMEWORKS: &[(&str, &str, &str)] = &[
    // Kreuzberg bindings
    ("kreuzberg-rust", "binary_size", "Native Rust core binary"),
    ("kreuzberg-python", "pip_package", "Python wheel package"),
    ("kreuzberg-node", "npm_package", "Node.js native addon"),
    ("kreuzberg-wasm", "wasm_bundle", "WebAssembly binary"),
    ("kreuzberg-ruby", "gem_package", "Ruby gem native extension"),
    ("kreuzberg-go", "binary_size", "Go binary with CGO"),
    ("kreuzberg-java", "jar_size", "Java JAR with JNI"),
    ("kreuzberg-csharp", "nuget_package", ".NET NuGet package"),
    ("kreuzberg-elixir", "hex_package", "Elixir hex package with NIF"),
    ("kreuzberg-php", "php_extension", "PHP extension"),
    // Batch variants (same size as non-batch)
    (
        "kreuzberg-ruby-batch",
        "gem_package",
        "Ruby gem native extension (batch)",
    ),
    (
        "kreuzberg-go-batch",
        "binary_size",
        "Go binary with CGO (batch)",
    ),
    (
        "kreuzberg-java-batch",
        "jar_size",
        "Java JAR with JNI (batch)",
    ),
    (
        "kreuzberg-elixir-batch",
        "hex_package",
        "Elixir hex package (batch)",
    ),
    (
        "kreuzberg-php-batch",
        "php_extension",
        "PHP extension (batch)",
    ),
    (
        "kreuzberg-python-batch",
        "pip_package",
        "Python wheel (batch)",
    ),
    (
        "kreuzberg-node-batch",
        "npm_package",
        "Node.js addon (batch)",
    ),
    ("kreuzberg-wasm-batch", "wasm_bundle", "WASM binary (batch)"),
    // Third-party frameworks
    ("docling", "pip_package", "IBM Docling document processing"),
    ("docling-batch", "pip_package", "IBM Docling (batch mode)"),
    ("markitdown", "pip_package", "Mark It Down markdown converter"),
    ("pandoc", "binary_size", "Pandoc universal converter"),
    ("unstructured", "pip_package", "Unstructured document processing"),
    ("tika", "jar_size", "Apache Tika content analysis"),
    ("tika-batch", "jar_size", "Apache Tika (batch mode)"),
    ("pymupdf4llm", "pip_package", "PyMuPDF for LLM"),
    ("pdfplumber", "pip_package", "pdfplumber PDF extraction"),
    ("pdfplumber-batch", "pip_package", "pdfplumber (batch mode)"),
    ("mineru", "pip_package", "MinerU document intelligence"),
    ("mineru-batch", "pip_package", "MinerU (batch mode)"),
];

/// Measure framework sizes
pub fn measure_framework_sizes() -> Result<FrameworkSizes> {
    let mut sizes = HashMap::new();

    for (name, method, description) in FRAMEWORKS {
        let size = measure_framework(name, method)?;
        sizes.insert(
            name.to_string(),
            FrameworkSize {
                size_bytes: size.unwrap_or(0),
                method: method.to_string(),
                description: description.to_string(),
                estimated: size.is_none(),
            },
        );
    }

    Ok(sizes)
}

/// Measure a single framework
fn measure_framework(name: &str, method: &str) -> Result<Option<u64>> {
    match method {
        "pip_package" => measure_pip_package(extract_package_name(name)),
        "npm_package" => measure_npm_package(extract_package_name(name)),
        "binary_size" => measure_binary(name),
        "jar_size" => measure_jar(name),
        // The following methods are not yet implemented - they return None (estimated)
        // gem_package: Ruby gem size measurement
        // wasm_bundle: WebAssembly bundle size
        // nuget_package: .NET NuGet package size
        // hex_package: Elixir Hex package size
        // php_extension: PHP extension size
        _ => Ok(None),
    }
}

/// Extract Python/npm package name from framework name
fn extract_package_name(framework: &str) -> &str {
    // Strip -batch suffix and kreuzberg- prefix for lookups
    let name = framework.strip_suffix("-batch").unwrap_or(framework);

    match name {
        "kreuzberg-python" => "kreuzberg",
        "kreuzberg-node" => "@anthropic/kreuzberg",
        "docling" => "docling",
        "markitdown" => "markitdown",
        "unstructured" => "unstructured",
        "pymupdf4llm" => "pymupdf4llm",
        "pdfplumber" => "pdfplumber",
        "mineru" => "mineru",
        _ => name,
    }
}

/// Measure Python package size using pip show
fn measure_pip_package(package: &str) -> Result<Option<u64>> {
    let output = Command::new("pip")
        .args(["show", "-f", package])
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Find Location line
            if let Some(location_line) = stdout.lines().find(|l| l.starts_with("Location:")) {
                let location = location_line.strip_prefix("Location:").unwrap().trim();

                // Calculate total size of package files
                let package_dir = Path::new(location).join(package.replace('-', "_"));
                if package_dir.exists() {
                    return Ok(Some(dir_size(&package_dir)));
                }
            }
        }
    }

    Ok(None)
}

/// Measure npm package size
fn measure_npm_package(package: &str) -> Result<Option<u64>> {
    // Try npm pack --dry-run to get package size
    let output = Command::new("npm")
        .args(["pack", "--dry-run", "--json", package])
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse JSON output for size
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(size) = json.get(0).and_then(|v| v.get("size")).and_then(|v| v.as_u64())
                {
                    return Ok(Some(size));
                }
            }
        }
    }

    Ok(None)
}

/// Measure binary size
fn measure_binary(name: &str) -> Result<Option<u64>> {
    let binary_name = match name {
        "pandoc" => "pandoc",
        "kreuzberg-rust" => "kreuzberg",
        _ => return Ok(None),
    };

    // Try which to find binary
    let output = Command::new("which").arg(binary_name).output().ok();

    if let Some(output) = output {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(metadata) = fs::metadata(&path) {
                return Ok(Some(metadata.len()));
            }
        }
    }

    Ok(None)
}

/// Measure JAR size (Apache Tika)
fn measure_jar(name: &str) -> Result<Option<u64>> {
    // Common locations for Tika JAR
    let possible_paths = [
        "/usr/share/java/tika-app.jar",
        "/opt/tika/tika-app.jar",
        "~/.local/share/tika/tika-app.jar",
    ];

    if name.starts_with("tika") {
        for path in possible_paths {
            let expanded = shellexpand::tilde(path);
            let expanded_path: &str = expanded.as_ref();
            if let Ok(metadata) = fs::metadata(expanded_path) {
                return Ok(Some(metadata.len()));
            }
        }
    }

    Ok(None)
}

/// Calculate total size of a directory
fn dir_size(path: &Path) -> u64 {
    let mut size = 0;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                size += dir_size(&path);
            } else if let Ok(metadata) = path.metadata() {
                size += metadata.len();
            }
        }
    }

    size
}

/// Load framework sizes from a JSON file
pub fn load_framework_sizes(path: &Path) -> Result<FrameworkSizes> {
    let contents = fs::read_to_string(path).map_err(Error::Io)?;
    serde_json::from_str(&contents).map_err(|e| Error::Benchmark(format!("Invalid JSON: {}", e)))
}

/// Save framework sizes to a JSON file
pub fn save_framework_sizes(sizes: &FrameworkSizes, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(sizes)
        .map_err(|e| Error::Benchmark(format!("JSON serialization failed: {}", e)))?;
    fs::write(path, json).map_err(Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        assert_eq!(extract_package_name("kreuzberg-python"), "kreuzberg");
        assert_eq!(extract_package_name("docling"), "docling");
        assert_eq!(extract_package_name("docling-batch"), "docling");
        assert_eq!(extract_package_name("pdfplumber-batch"), "pdfplumber");
    }

    #[test]
    fn test_frameworks_list_complete() {
        assert!(FRAMEWORKS.len() >= 26);

        // Check all kreuzberg bindings present
        let names: Vec<&str> = FRAMEWORKS.iter().map(|(n, _, _)| *n).collect();
        assert!(names.contains(&"kreuzberg-rust"));
        assert!(names.contains(&"kreuzberg-python"));
        assert!(names.contains(&"kreuzberg-node"));

        // Check third-party frameworks present
        assert!(names.contains(&"docling"));
        assert!(names.contains(&"tika"));
        assert!(names.contains(&"pandoc"));
    }

    #[test]
    fn test_dir_size_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        let size = dir_size(temp.path());
        assert_eq!(size, 0);
    }

    #[test]
    fn test_dir_size_with_files() {
        let temp = tempfile::TempDir::new().unwrap();
        fs::write(temp.path().join("a.txt"), "hello").unwrap();
        fs::write(temp.path().join("b.txt"), "world!").unwrap();

        let size = dir_size(temp.path());
        assert_eq!(size, 11); // "hello" (5) + "world!" (6)
    }
}
