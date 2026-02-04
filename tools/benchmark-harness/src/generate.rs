//! Fixture generation from vendored test documents
//!
//! Generates benchmark fixture JSON files by scanning vendored test documents
//! and optionally linking to ground truth files.

use crate::fixture::{Fixture, GroundTruth};
use crate::{Error, Result};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Framework support matrix by file type
fn get_expected_frameworks(file_type: &str) -> Vec<String> {
    match file_type {
        "pdf" => vec![
            "kreuzberg",
            "docling",
            "markitdown",
            "unstructured",
            "pdfplumber",
            "pymupdf4llm",
        ],
        "docx" => vec!["kreuzberg", "docling", "markitdown", "unstructured"],
        "doc" => vec!["kreuzberg", "unstructured"],
        "xlsx" | "xls" | "xlsm" => vec!["kreuzberg", "docling"],
        "pptx" | "ppt" => vec!["kreuzberg", "docling"],
        "html" | "htm" => vec!["kreuzberg", "markitdown", "unstructured"],
        "csv" => vec!["kreuzberg", "unstructured"],
        "json" | "xml" | "yaml" | "yml" => vec!["kreuzberg"],
        "md" | "markdown" => vec!["kreuzberg", "markitdown"],
        "txt" | "text" => vec!["kreuzberg", "unstructured"],
        "rtf" => vec!["kreuzberg", "unstructured"],
        "eml" | "msg" => vec!["kreuzberg"],
        "epub" => vec!["kreuzberg"],
        "odt" | "ods" | "odp" => vec!["kreuzberg"],
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => vec!["kreuzberg"],
        _ => vec!["kreuzberg"],
    }
    .into_iter()
    .map(String::from)
    .collect()
}

/// Get the ground truth source type for a file format
fn get_ground_truth_source(file_type: &str) -> &'static str {
    match file_type {
        "pdf" => "pdftotext",
        "docx" | "doc" => "python-docx",
        "pptx" | "ppt" => "python-pptx",
        "xlsx" | "xls" | "xlsm" => "openpyxl",
        "csv" | "html" | "htm" | "json" | "xml" | "yaml" | "yml" | "txt" | "md" | "markdown" => {
            "raw_source"
        }
        _ => "manual",
    }
}

/// Configuration for fixture generation
#[derive(Debug, Clone)]
pub struct GenerateConfig {
    /// Directory containing vendored test documents
    pub vendored_dir: PathBuf,
    /// Output directory for generated fixtures
    pub output_dir: PathBuf,
    /// Optional ground truth directory
    pub ground_truth_dir: Option<PathBuf>,
    /// Overwrite existing fixtures
    pub overwrite: bool,
}

/// Statistics from fixture generation
#[derive(Debug, Default)]
pub struct GenerateStats {
    /// Number of fixtures created
    pub created: usize,
    /// Number of fixtures skipped (already exist)
    pub skipped: usize,
    /// Number of files with errors
    pub errors: usize,
    /// Counts by file type
    pub by_type: HashMap<String, usize>,
}

/// Generate fixture JSON files from vendored test documents
pub fn generate_fixtures(config: &GenerateConfig) -> Result<GenerateStats> {
    let mut stats = GenerateStats::default();

    if !config.vendored_dir.exists() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Vendored directory not found: {}", config.vendored_dir.display()),
        )));
    }

    // Create output directory
    fs::create_dir_all(&config.output_dir).map_err(Error::Io)?;

    // Scan for documents
    scan_directory(&config.vendored_dir, config, &mut stats)?;

    Ok(stats)
}

fn scan_directory(dir: &Path, config: &GenerateConfig, stats: &mut GenerateStats) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and groundtruth
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "groundtruth" {
                    continue;
                }
            }
            scan_directory(&path, config, stats)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            // Skip non-document files
            let file_type = ext.to_lowercase();
            if is_supported_format(&file_type) {
                if let Err(e) = process_document(&path, &file_type, config, stats) {
                    eprintln!(
                        "Warning: Failed to process {}: {}",
                        path.display(),
                        e
                    );
                    stats.errors += 1;
                }
            }
        }
    }

    Ok(())
}

fn is_supported_format(ext: &str) -> bool {
    matches!(
        ext,
        "pdf"
            | "docx"
            | "doc"
            | "xlsx"
            | "xls"
            | "xlsm"
            | "pptx"
            | "ppt"
            | "html"
            | "htm"
            | "csv"
            | "json"
            | "xml"
            | "yaml"
            | "yml"
            | "md"
            | "markdown"
            | "txt"
            | "rtf"
            | "eml"
            | "msg"
            | "epub"
            | "odt"
            | "ods"
            | "odp"
            | "jpg"
            | "jpeg"
            | "png"
            | "gif"
            | "bmp"
            | "tiff"
            | "webp"
    )
}

fn process_document(
    doc_path: &Path,
    file_type: &str,
    config: &GenerateConfig,
    stats: &mut GenerateStats,
) -> Result<()> {
    let file_stem = doc_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::Benchmark("Invalid filename".to_string()))?;

    // Determine output subdirectory and fixture path
    let output_subdir = config.output_dir.join(file_type);
    fs::create_dir_all(&output_subdir).map_err(Error::Io)?;

    let fixture_path = output_subdir.join(format!("{}.json", file_stem));

    // Skip if exists and not overwriting
    if fixture_path.exists() && !config.overwrite {
        stats.skipped += 1;
        return Ok(());
    }

    // Get file size
    let file_size = fs::metadata(doc_path).map_err(Error::Io)?.len();

    // Calculate relative path from fixture to document
    let doc_rel_path = calculate_relative_path(&fixture_path, doc_path)?;

    // Check for ground truth
    let ground_truth = if let Some(gt_dir) = &config.ground_truth_dir {
        let gt_file = gt_dir.join(file_type).join(format!("{}.txt", file_stem));
        if gt_file.exists() {
            let gt_rel_path = calculate_relative_path(&fixture_path, &gt_file)?;
            Some(GroundTruth {
                text_file: gt_rel_path,
                source: get_ground_truth_source(file_type).to_string(),
            })
        } else {
            None
        }
    } else {
        None
    };

    // Build fixture
    let fixture = Fixture {
        document: doc_rel_path,
        file_type: file_type.to_string(),
        file_size,
        expected_frameworks: get_expected_frameworks(file_type),
        metadata: build_metadata(doc_path, config),
        ground_truth,
    };

    // Write fixture
    let json = serde_json::to_string_pretty(&fixture)
        .map_err(|e| Error::Benchmark(format!("JSON serialization failed: {}", e)))?;
    fs::write(&fixture_path, json).map_err(Error::Io)?;

    stats.created += 1;
    *stats.by_type.entry(file_type.to_string()).or_insert(0) += 1;

    Ok(())
}

fn calculate_relative_path(from: &Path, to: &Path) -> Result<PathBuf> {
    // Get canonical paths
    let from_dir = from.parent().ok_or_else(|| {
        Error::Benchmark(format!("Cannot get parent directory of: {}", from.display()))
    })?;
    let from_abs = fs::canonicalize(from_dir).map_err(|e| {
        Error::Benchmark(format!(
            "Cannot canonicalize fixture directory {}: {}",
            from_dir.display(),
            e
        ))
    })?;
    let to_abs = fs::canonicalize(to).map_err(Error::Io)?;

    // Calculate relative path
    let rel = pathdiff::diff_paths(&to_abs, &from_abs).ok_or_else(|| {
        Error::Benchmark(format!(
            "Cannot calculate relative path from {} to {}",
            from_abs.display(),
            to_abs.display()
        ))
    })?;

    Ok(rel)
}

fn build_metadata(doc_path: &Path, config: &GenerateConfig) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();

    // Try to determine source (vendor) from path
    if let Ok(rel) = doc_path.strip_prefix(&config.vendored_dir) {
        if let Some(first_component) = rel.components().next() {
            if let Some(vendor) = first_component.as_os_str().to_str() {
                metadata.insert("source".to_string(), json!(vendor));
            }
        }
    }

    // Add description
    if let Some(stem) = doc_path.file_stem().and_then(|s| s.to_str()) {
        metadata.insert(
            "description".to_string(),
            json!(format!("Document: {}", stem)),
        );
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_expected_frameworks_pdf() {
        let frameworks = get_expected_frameworks("pdf");
        assert!(frameworks.contains(&"kreuzberg".to_string()));
        assert!(frameworks.contains(&"docling".to_string()));
        assert!(frameworks.len() >= 4);
    }

    #[test]
    fn test_get_expected_frameworks_unknown() {
        let frameworks = get_expected_frameworks("xyz123");
        assert_eq!(frameworks, vec!["kreuzberg".to_string()]);
    }

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format("pdf"));
        assert!(is_supported_format("docx"));
        assert!(is_supported_format("html"));
        assert!(!is_supported_format("exe"));
        assert!(!is_supported_format("zip"));
    }

    #[test]
    fn test_get_ground_truth_source() {
        assert_eq!(get_ground_truth_source("pdf"), "pdftotext");
        assert_eq!(get_ground_truth_source("docx"), "python-docx");
        assert_eq!(get_ground_truth_source("csv"), "raw_source");
        assert_eq!(get_ground_truth_source("unknown"), "manual");
    }

    #[test]
    fn test_generate_fixtures_empty_dir() {
        let temp = TempDir::new().unwrap();
        let vendored = temp.path().join("vendored");
        let output = temp.path().join("fixtures");
        fs::create_dir_all(&vendored).unwrap();

        let config = GenerateConfig {
            vendored_dir: vendored,
            output_dir: output,
            ground_truth_dir: None,
            overwrite: false,
        };

        let stats = generate_fixtures(&config).unwrap();
        assert_eq!(stats.created, 0);
        assert_eq!(stats.skipped, 0);
    }

    #[test]
    fn test_generate_fixtures_with_document() {
        let temp = TempDir::new().unwrap();
        let vendored = temp.path().join("vendored");
        let output = temp.path().join("fixtures");
        fs::create_dir_all(&vendored).unwrap();

        // Create a test document
        fs::write(vendored.join("test.txt"), "Hello, world!").unwrap();

        let config = GenerateConfig {
            vendored_dir: vendored,
            output_dir: output.clone(),
            ground_truth_dir: None,
            overwrite: false,
        };

        let stats = generate_fixtures(&config).unwrap();
        assert_eq!(stats.created, 1);
        assert!(output.join("txt").join("test.json").exists());
    }
}
