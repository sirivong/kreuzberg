//! Integration tests for PaddleOCR functionality.
//!
//! These tests require:
//! - Network access to download models from HuggingFace
//! - ONNX Runtime installed on the system
//!
//! Run with: `cargo test -p xberg --features paddle-ocr --test paddle_ocr_integration -- --ignored`
//!
//! Model-manager-level tests use only the public `ModelManager` surface
//! (`new`, `ensure_all_models`, `manifest`) plus documented on-disk cache
//! layout; the previous version of this suite relied on `pub(crate)` helpers
//! (`ensure_models_exist`, `ensure_v2_det_model`, `resolve_rec_model`,
//! `cache_stats`, `are_models_cached`) that the alef-migration cleanup
//! deliberately narrowed to crate-internal visibility, which is why the file
//! was fully disabled via `#![cfg(any())]` until now.

#![cfg(feature = "paddle-ocr")]

use std::path::PathBuf;

use xberg::core::config::OcrConfig;
use xberg::paddle_ocr::{ModelManager, PaddleOcrBackend, PaddleOcrConfig};
use xberg::plugins::OcrBackend;
use xberg::types::ExtractedDocument;

/// Helper to get the test documents directory
fn test_documents_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test_documents")
}

/// Helper to get a temporary cache directory for tests
fn test_cache_dir() -> PathBuf {
    std::env::temp_dir().join("xberg_paddle_test")
}

/// Test that model manager can download models from HuggingFace.
///
/// This test downloads actual models and verifies they are cached correctly,
/// using only the public `ModelManager` surface (`new`, `ensure_all_models`)
/// plus the documented on-disk cache layout (`v2/det/<tier>`, `v2/cls`, etc.).
/// It's ignored by default since it requires network access and a large
/// (all-tier) download.
#[tokio::test]
#[ignore = "requires network access and a large (all-tier) download"]
async fn test_model_download_from_huggingface() {
    let cache_dir = test_cache_dir();

    // Clean up any existing cache
    let _ = std::fs::remove_dir_all(&cache_dir);

    let manager = ModelManager::new(cache_dir.clone());

    // Download all models (synchronous)
    let result = manager.ensure_all_models();
    assert!(result.is_ok(), "Model download failed: {:?}", result.err());

    // Verify shared model directories exist (documented cache layout)
    let det_dir = cache_dir.join("v2").join("det").join("server");
    let cls_dir = cache_dir.join("v2").join("cls");
    let ori_dir = cache_dir.join("v2").join("doc_ori");

    assert!(det_dir.join("model.onnx").exists(), "Detection ONNX file not found");
    assert!(
        cls_dir.join("model.onnx").exists(),
        "Classification ONNX file not found"
    );
    assert!(
        ori_dir.join("model.onnx").exists(),
        "Document orientation ONNX file not found"
    );

    // Verify the manifest describes at least the shared + per-script + v6 entries
    let manifest = ModelManager::manifest();
    assert!(
        manifest.len() >= 3,
        "Expected at least 3 manifest entries, got {}",
        manifest.len()
    );

    println!("Manifest entries: {}", manifest.len());
    println!("Detection model: {:?}", det_dir);
    println!("Classification model: {:?}", cls_dir);
    println!("Document orientation model: {:?}", ori_dir);
}

/// Test OCR on a simple English "Hello World" image.
///
/// This test requires ONNX Runtime and downloaded models.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_ocr_hello_world_english() {
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();
    let text = extraction.content.to_lowercase();

    println!("OCR result: {}", extraction.content);

    // Should contain "hello" and "world"
    assert!(
        text.contains("hello") || text.contains("helo"),
        "Expected 'hello' in OCR result: {}",
        text
    );
    assert!(
        text.contains("world") || text.contains("worid"),
        "Expected 'world' in OCR result: {}",
        text
    );
}

/// Test PP-OCRv6 recognition on English across all three v6 tiers.
///
/// v6 routes English (a v6-unified family) to the unified recognition model at the
/// configured tier (medium/small/tiny), with a v6 detector and the shared PP-LCNet
/// classifier. Verifies the version-aware wiring end to end against the live models.
#[tokio::test]
#[ignore = "requires network access and ONNX Runtime (PP-OCRv6 model download)"]
async fn test_ocr_pp_ocrv6_english_tiers() {
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);
    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    for tier in ["medium", "small", "tiny"] {
        let config = PaddleOcrConfig::new("en")
            .with_model_version("pp-ocrv6")
            .with_model_tier(tier)
            .with_cache_dir(test_cache_dir());

        let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

        let ocr_config = OcrConfig {
            backend: "paddle-ocr".to_string(),
            language: vec!["en".to_string()],
            ..Default::default()
        };

        let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
        assert!(result.is_ok(), "PP-OCRv6 {tier} OCR failed: {:?}", result.err());

        let text = result.unwrap().content.to_lowercase();
        println!("PP-OCRv6 {tier} OCR result: {text}");
        assert!(
            text.contains("hello") || text.contains("helo"),
            "Expected 'hello' in PP-OCRv6 {tier} result: {text}"
        );
        assert!(
            text.contains("world") || text.contains("worid"),
            "Expected 'world' in PP-OCRv6 {tier} result: {text}"
        );
    }
}

/// Test OCR on a complex English document (newspaper).
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_ocr_newspaper_english() {
    let image_path = test_documents_dir().join("images/ocr_image.jpg");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();
    let text = extraction.content.to_uppercase();

    println!(
        "OCR result (first 500 chars): {}",
        &extraction.content[..extraction.content.len().min(500)]
    );

    // Should contain "NASDAQ" and "AMEX" from the header
    assert!(
        text.contains("NASDAQ") || text.contains("NASOAQ"),
        "Expected 'NASDAQ' in OCR result"
    );
    assert!(
        text.contains("AMEX") || text.contains("STOCK"),
        "Expected 'AMEX' or 'STOCK' in OCR result"
    );
}

/// Test OCR on Chinese text image.
///
/// Note: Uses per-family PP-OCRv5 recognition models.
/// This test verifies the pipeline handles non-English images without crashing,
/// but requires the Chinese recognition model to be cached for accurate results.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_ocr_chinese_text() {
    let image_path = test_documents_dir().join("images/chi_sim_image.jpeg");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    // Use Chinese language setting
    let config = PaddleOcrConfig::new("ch").with_cache_dir(test_cache_dir());

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["ch".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();

    println!("OCR result: {}", extraction.content);

    // The pipeline should produce some output without crashing.
    // With the English-only model, Chinese characters are not recognized,
    // but the detection and recognition pipeline should still function.
    assert!(
        !extraction.content.is_empty(),
        "Expected non-empty OCR result for Chinese image"
    );
}

/// Test that the backend correctly reports supported languages.
#[test]
fn test_supported_languages() {
    let backend = PaddleOcrBackend::new().expect("Failed to create backend");

    // Direct PaddleOCR codes
    assert!(backend.supports_language("ch"));
    assert!(backend.supports_language("en"));
    assert!(backend.supports_language("japan"));
    assert!(backend.supports_language("korean"));

    // Mapped Tesseract/ISO codes
    assert!(backend.supports_language("chi_sim"));
    assert!(backend.supports_language("eng"));
    assert!(backend.supports_language("jpn"));
    assert!(backend.supports_language("fra"));
    assert!(backend.supports_language("deu"));

    // Unsupported
    assert!(!backend.supports_language("xyz"));
    assert!(!backend.supports_language("klingon"));
}

/// Test that empty image returns an error.
#[tokio::test]
async fn test_empty_image_error() {
    let backend = PaddleOcrBackend::new().expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&[], &ocr_config).await;
    assert!(result.is_err(), "Expected error for empty image");
}

/// Test that invalid image data returns an error (requires ONNX Runtime).
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_invalid_image_error() {
    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());
    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    // Random bytes that aren't a valid image
    let invalid_bytes = vec![0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&invalid_bytes, &ocr_config).await;
    assert!(result.is_err(), "Expected error for invalid image data");
}

/// Test processing an image file directly.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_process_image_file() {
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());
    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image_file(&image_path, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();
    let text = extraction.content.to_lowercase();

    assert!(
        text.contains("hello") || text.contains("helo"),
        "Expected 'hello' in OCR result"
    );
}

/// Test that explicit cache_dir in config overrides default.
#[test]
fn test_cache_dir_explicit_config() {
    // Set explicit config - this should always work regardless of env vars
    let config = PaddleOcrConfig::new("en").with_cache_dir(PathBuf::from("/explicit/path"));
    let resolved = config.resolve_cache_dir();

    // Explicit config should always win
    assert_eq!(resolved, PathBuf::from("/explicit/path"));
}

/// Test that OCR elements have proper geometry (quadrilateral bounding boxes).
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_paddle_ocr_elements_geometry() {
    use xberg::types::OcrBoundingGeometry;

    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());
    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();

    // Check that OCR elements are present
    assert!(
        extraction.ocr_elements.is_some(),
        "Expected ocr_elements to be populated"
    );

    let elements = extraction.ocr_elements.as_ref().unwrap();
    assert!(!elements.is_empty(), "Expected at least one OCR element");

    // Verify each element has geometry
    for element in elements {
        // Check geometry based on variant
        match &element.geometry {
            OcrBoundingGeometry::Quadrilateral { points } => {
                // Quadrilateral should have 4 points
                assert_eq!(points.len(), 4, "Quadrilateral should have 4 points");
                println!("Quadrilateral with 4 points");
            }
            OcrBoundingGeometry::Rectangle {
                left,
                top,
                width,
                height,
            } => {
                assert!(*width > 0, "Width should be positive");
                assert!(*height > 0, "Height should be positive");
                println!("Rectangle at ({}, {}) size {}x{}", left, top, width, height);
            }
        }
    }

    println!("Found {} OCR elements with valid geometry", elements.len());
}

/// Test that OCR elements have confidence scores.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_paddle_ocr_elements_confidence() {
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());
    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();

    assert!(
        extraction.ocr_elements.is_some(),
        "Expected ocr_elements to be populated"
    );

    let elements = extraction.ocr_elements.as_ref().unwrap();
    assert!(!elements.is_empty(), "Expected at least one OCR element");

    // Verify each element has confidence score
    for element in elements {
        // Recognition confidence should be between 0 and 1
        assert!(
            element.confidence.recognition >= 0.0 && element.confidence.recognition <= 1.0,
            "Recognition confidence should be between 0 and 1, got {}",
            element.confidence.recognition
        );

        // PaddleOCR also provides detection confidence
        if let Some(det_conf) = element.confidence.detection {
            assert!(
                (0.0..=1.0).contains(&det_conf),
                "Detection confidence should be between 0 and 1, got {}",
                det_conf
            );
        }

        println!(
            "Element '{}' has recognition confidence: {:.2}%",
            element.text,
            element.confidence.recognition * 100.0
        );
    }
}

/// Test rotation detection via angle classification.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_paddle_ocr_rotation_detection() {
    // Use an image that might have rotated text
    let image_path = test_documents_dir().join("images/ocr_image.jpg");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    // Enable angle classification
    let config = PaddleOcrConfig::new("en").with_cache_dir(test_cache_dir());

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();

    assert!(
        extraction.ocr_elements.is_some(),
        "Expected ocr_elements to be populated"
    );

    let elements = extraction.ocr_elements.as_ref().unwrap();

    // Check that rotation info is populated when available
    let elements_with_rotation = elements.iter().filter(|e| e.rotation.is_some()).count();

    println!(
        "Found {} elements total, {} with rotation info",
        elements.len(),
        elements_with_rotation
    );

    // For elements with rotation, verify the angle is valid
    for element in elements.iter().filter(|e| e.rotation.is_some()) {
        let rotation = element.rotation.as_ref().unwrap();
        // Rotation should be in degrees (typically 0, 90, 180, 270)
        assert!(
            rotation.angle_degrees >= 0.0 && rotation.angle_degrees < 360.0,
            "Rotation angle should be between 0 and 360, got {}",
            rotation.angle_degrees
        );
    }
}

/// Test table reconstruction from OCR elements.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_paddle_ocr_table_reconstruction() {
    let image_path = test_documents_dir().join("images/simple_table.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    // Enable table detection
    let config = PaddleOcrConfig::new("en")
        .with_cache_dir(test_cache_dir())
        .with_table_detection(true);

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "OCR failed: {:?}", result.err());

    let extraction: ExtractedDocument = result.unwrap();

    println!(
        "OCR result (first 500 chars): {}",
        &extraction.content[..extraction.content.len().min(500)]
    );

    // Check if tables were detected
    if !extraction.tables.is_empty() {
        println!("Found {} tables", extraction.tables.len());
        for (i, table) in extraction.tables.iter().enumerate() {
            println!(
                "Table {}: {} rows x {} cols",
                i,
                table.cells.len(),
                table.cells.first().map(|r| r.len()).unwrap_or(0)
            );
        }
    }

    // OCR elements should also be populated
    if let Some(elements) = &extraction.ocr_elements {
        println!("Found {} OCR elements", elements.len());

        // Elements should have text content
        let non_empty_elements = elements.iter().filter(|e| !e.text.is_empty()).count();
        assert!(non_empty_elements > 0, "Expected at least one element with text");
    }
}

// ============================================================================
// Mobile tier integration tests with quality measurement (TF1)
// ============================================================================

/// Compute Text F1 score: token-level precision/recall between predicted and reference text.
fn compute_tf1(predicted: &str, reference: &str) -> f64 {
    let pred_tokens: Vec<&str> = predicted.split_whitespace().collect();
    let ref_tokens: Vec<&str> = reference.split_whitespace().collect();

    if pred_tokens.is_empty() && ref_tokens.is_empty() {
        return 1.0;
    }
    if pred_tokens.is_empty() || ref_tokens.is_empty() {
        return 0.0;
    }

    let pred_set: std::collections::HashSet<&str> = pred_tokens.iter().copied().collect();
    let ref_set: std::collections::HashSet<&str> = ref_tokens.iter().copied().collect();

    let intersection = pred_set.intersection(&ref_set).count() as f64;
    let precision = intersection / pred_set.len() as f64;
    let recall = intersection / ref_set.len() as f64;

    if precision + recall == 0.0 {
        return 0.0;
    }
    2.0 * precision * recall / (precision + recall)
}

/// Ground truth for the complex_document test image.
const COMPLEX_DOC_GT: &str = "Sales Report 2024 This report contains quarterly sales data for our products. Q1 Sales: Product Units Revenue Widget A 150 ,500 Widget B 200 ,000 Widget C 100 ,000 Q2 Sales: Product Units Revenue Widget A 180 ,000 Widget B 220 ,200 Widget C 130 ,400 Summary: Total Q1 Revenue: ,500 Total Q2 Revenue: ,600 Prepared by: John Doe Date: 2024-03-15 Department: Finance";

/// Test mobile tier OCR on a document image, measuring TF1.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_mobile_tier_ocr_quality() {
    let image_path = test_documents_dir().join("images/complex_document.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    // Mobile tier config
    let config = PaddleOcrConfig::new("en")
        .with_cache_dir(test_cache_dir())
        .with_model_tier("mobile");

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        paddle_ocr_config: Some(serde_json::json!({"model_tier": "mobile"})),
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let result = backend.process_image(&image_bytes, &ocr_config).await;
    let elapsed_ms = start.elapsed().as_millis();

    assert!(result.is_ok(), "Mobile tier OCR failed: {:?}", result.err());

    let extraction = result.unwrap();
    let tf1 = compute_tf1(&extraction.content, COMPLEX_DOC_GT);

    println!("Mobile tier TF1: {:.1}% ({} ms)", tf1 * 100.0, elapsed_ms);
    println!(
        "Extracted text: {}",
        &extraction.content[..extraction.content.len().min(200)]
    );

    // Mobile tier should achieve at least 50% TF1 on this document
    assert!(
        tf1 > 0.5,
        "Mobile tier TF1 too low: {:.1}% (expected >50%)",
        tf1 * 100.0
    );
}

/// Test server tier OCR on the same document for comparison.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_server_tier_ocr_quality() {
    let image_path = test_documents_dir().join("images/complex_document.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);

    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    // Server tier config (default)
    let config = PaddleOcrConfig::new("en")
        .with_cache_dir(test_cache_dir())
        .with_model_tier("server");

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let result = backend.process_image(&image_bytes, &ocr_config).await;
    let elapsed_ms = start.elapsed().as_millis();

    assert!(result.is_ok(), "Server tier OCR failed: {:?}", result.err());

    let extraction = result.unwrap();
    let tf1 = compute_tf1(&extraction.content, COMPLEX_DOC_GT);

    println!("Server tier TF1: {:.1}% ({} ms)", tf1 * 100.0, elapsed_ms);

    // Server tier should achieve at least 60% TF1
    assert!(
        tf1 > 0.6,
        "Server tier TF1 too low: {:.1}% (expected >60%)",
        tf1 * 100.0
    );
}

/// Test mobile tier with auto_rotate on rotated images.
/// Verifies that doc_ori detection + rotation correction produces consistent TF1.
#[tokio::test]
#[ignore = "requires ONNX Runtime and downloaded models"]
async fn test_mobile_tier_auto_rotate() {
    let base_dir = test_documents_dir().join("images");

    let test_cases = vec![
        ("complex_document.png", "original (0°)"),
        ("complex_document_rotated_90.png", "rotated 90°"),
        ("complex_document_rotated_180.png", "rotated 180°"),
        ("complex_document_rotated_270.png", "rotated 270°"),
    ];

    let config = PaddleOcrConfig::new("en")
        .with_cache_dir(test_cache_dir())
        .with_model_tier("mobile");

    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let mut tf1_scores = Vec::new();

    for (filename, label) in &test_cases {
        let image_path = base_dir.join(filename);
        assert!(image_path.exists(), "Test image not found: {:?}", image_path);

        let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

        let ocr_config = OcrConfig {
            backend: "paddle-ocr".to_string(),
            language: vec!["en".to_string()],
            auto_rotate: true,
            paddle_ocr_config: Some(serde_json::json!({"model_tier": "mobile"})),
            ..Default::default()
        };

        let start = std::time::Instant::now();
        let result = backend.process_image(&image_bytes, &ocr_config).await;
        let elapsed_ms = start.elapsed().as_millis();

        assert!(result.is_ok(), "OCR failed on {}: {:?}", label, result.err());

        let extraction = result.unwrap();
        let tf1 = compute_tf1(&extraction.content, COMPLEX_DOC_GT);
        tf1_scores.push(tf1);

        println!("{}: TF1={:.1}% ({} ms)", label, tf1 * 100.0, elapsed_ms);
    }

    // All orientations should produce consistent quality (within 20% of each other)
    let min_tf1 = tf1_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_tf1 = tf1_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!(
        "TF1 range: {:.1}% - {:.1}% (spread: {:.1}%)",
        min_tf1 * 100.0,
        max_tf1 * 100.0,
        (max_tf1 - min_tf1) * 100.0
    );

    // Auto-rotate should make all orientations achieve at least 40% TF1
    assert!(
        min_tf1 > 0.4,
        "Worst orientation TF1 too low: {:.1}% (expected >40% with auto_rotate)",
        min_tf1 * 100.0
    );

    // Spread should be <30% — auto_rotate should normalize quality across orientations
    assert!(
        max_tf1 - min_tf1 < 0.3,
        "TF1 spread too large: {:.1}% (expected <30% with auto_rotate)",
        (max_tf1 - min_tf1) * 100.0
    );
}

/// Test that mobile tier model download caches correctly.
///
/// Drives the download through the mobile-tier OCR path (`PaddleOcrBackend`
/// with `model_tier = "mobile"`) and verifies the resulting cache layout on
/// disk, since tier-specific model resolution (`ensure_v2_det_model`,
/// `resolve_rec_model`) is a crate-internal `ModelManager` detail.
#[tokio::test]
#[ignore = "requires network access"]
async fn test_mobile_tier_model_cache() {
    let cache_dir = test_cache_dir();
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);
    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    let config = PaddleOcrConfig::new("en")
        .with_cache_dir(cache_dir.clone())
        .with_model_tier("mobile");
    let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

    let ocr_config = OcrConfig {
        backend: "paddle-ocr".to_string(),
        language: vec!["en".to_string()],
        paddle_ocr_config: Some(serde_json::json!({"model_tier": "mobile"})),
        ..Default::default()
    };

    let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
    assert!(result.is_ok(), "Mobile tier OCR failed: {:?}", result.err());

    // Mobile det model should be cached under the documented layout and be
    // much smaller than the server tier (~4.7MB vs ~88MB).
    let det_file = cache_dir.join("v2").join("det").join("mobile").join("model.onnx");
    assert!(det_file.exists(), "Mobile det model not cached at {:?}", det_file);

    let det_size = std::fs::metadata(&det_file).unwrap().len();
    assert!(
        det_size < 10_000_000,
        "Mobile det model too large: {} bytes (expected <10MB)",
        det_size
    );
    println!(
        "Mobile det model size: {} bytes ({:.1} MB)",
        det_size,
        det_size as f64 / 1_048_576.0
    );
}

/// Test that server and mobile tiers produce different cached model files.
#[tokio::test]
#[ignore = "requires network access"]
async fn test_tier_model_differentiation() {
    let cache_dir = test_cache_dir();
    let image_path = test_documents_dir().join("images/test_hello_world.png");
    assert!(image_path.exists(), "Test image not found: {:?}", image_path);
    let image_bytes = std::fs::read(&image_path).expect("Failed to read image");

    for tier in ["server", "mobile"] {
        let config = PaddleOcrConfig::new("en")
            .with_cache_dir(cache_dir.clone())
            .with_model_tier(tier);
        let backend = PaddleOcrBackend::with_config(config).expect("Failed to create backend");

        let ocr_config = OcrConfig {
            backend: "paddle-ocr".to_string(),
            language: vec!["en".to_string()],
            paddle_ocr_config: Some(serde_json::json!({"model_tier": tier})),
            ..Default::default()
        };

        let result: xberg::Result<ExtractedDocument> = backend.process_image(&image_bytes, &ocr_config).await;
        assert!(result.is_ok(), "{tier} tier OCR failed: {:?}", result.err());
    }

    let server_det = cache_dir.join("v2").join("det").join("server").join("model.onnx");
    let mobile_det = cache_dir.join("v2").join("det").join("mobile").join("model.onnx");
    assert!(server_det.exists(), "Server det model not cached");
    assert!(mobile_det.exists(), "Mobile det model not cached");
    assert_ne!(server_det, mobile_det, "Server and mobile det paths should differ");

    println!("Server det: {:?}", server_det);
    println!("Mobile det: {:?}", mobile_det);
}

/// Test default cache directory when no explicit config is set.
#[test]
#[allow(unsafe_code)]
fn test_cache_dir_default() {
    // Save and clear env var to test default behavior
    let original = std::env::var("XBERG_CACHE_DIR").ok();

    // SAFETY: This is a test that manipulates environment variables.
    // Tests should be run with --test-threads=1 if this causes issues.
    unsafe {
        std::env::remove_var("XBERG_CACHE_DIR");
    }

    let config = PaddleOcrConfig::new("en");
    let resolved = config.resolve_cache_dir();

    // Default should use the platform-appropriate global xberg cache dir,
    // e.g. `~/Library/Caches/xberg/paddle-ocr` (macOS), `$XDG_CACHE_HOME/xberg/paddle-ocr`
    // (Linux), or `~/.cache/xberg/paddle-ocr` as a home-dir fallback.
    assert!(resolved.to_string_lossy().contains("xberg"));
    assert!(resolved.to_string_lossy().contains("paddle-ocr"));

    // Restore
    unsafe {
        if let Some(val) = original {
            std::env::set_var("XBERG_CACHE_DIR", val);
        }
    }
}
