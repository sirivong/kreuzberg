#![allow(clippy::print_stdout)]
#![cfg(all(
    feature = "pdf",
    feature = "ocr",
    feature = "bundle-tessdata-eng",
    feature = "tokio-runtime"
))]

#[path = "ocr_pixel_phase_2711/fixture.rs"]
mod fixture;

use sha2::{Digest, Sha256};
use xberg::types::{ExtractionMethod, ImagePreprocessingConfig};
use xberg::{
    ExtractInput, ExtractedDocument, ExtractionConfig, ImageExtractionConfig, OcrConfig, PageConfig, TesseractConfig,
};

fn expected_words() -> Vec<String> {
    words(&fixture::LINES.join(" "))
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .collect()
}

async fn read(
    bytes: Vec<u8>,
    images: Option<ImageExtractionConfig>,
    preprocessing: Option<ImagePreprocessingConfig>,
) -> ExtractedDocument {
    let models = tempfile::tempdir().unwrap();
    let model = xberg_tesseract::bundled_eng_traineddata().expect("bundled English");
    assert_eq!(
        hex::encode(Sha256::digest(model)),
        "7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2"
    );
    std::fs::write(models.path().join("eng.traineddata"), model).unwrap();
    let config = ExtractionConfig {
        force_ocr: true,
        use_cache: false,
        images,
        pages: Some(PageConfig {
            extract_pages: true,
            ..Default::default()
        }),
        ocr: Some(OcrConfig {
            tessdata_path: Some(models.path().to_path_buf()),
            tesseract_config: Some(TesseractConfig {
                use_cache: false,
                preprocessing,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let native = xberg_native_pdf::PdfDocument::from_bytes(bytes.clone()).unwrap();
    assert_eq!(words(&native.extract_text(0).unwrap()), expected_words());
    let output = xberg::extract(ExtractInput::from_bytes(bytes, "application/pdf", None), &config)
        .await
        .unwrap();
    assert!(output.errors.is_empty(), "{:?}", output.errors);
    let [document]: [ExtractedDocument; 1] = output.results.try_into().expect("one document");
    assert_eq!(document.extraction_method, Some(ExtractionMethod::Ocr));
    assert_eq!(document.pages.as_ref().unwrap().len(), 1);
    document
}

fn original_dimensions(document: &ExtractedDocument) -> (usize, usize) {
    let metadata = document.pages.as_ref().unwrap()[0]
        .image_preprocessing
        .as_ref()
        .expect("render provenance");
    (metadata.original_dimensions.width, metadata.original_dimensions.height)
}

#[tokio::test]
async fn configured_preprocessing_density_preserves_source_words_before_resampling() {
    let bytes = fixture::localized_pdf("gap-left");
    assert_eq!(
        hex::encode(Sha256::digest(&bytes)),
        "b260b82cb305f91c23ba5575bdaf4bfb3943324b82389467773162fdc15b0937"
    );
    let document = read(bytes, None, Some(ImagePreprocessingConfig::default())).await;
    println!(
        "render={:?} words={}",
        original_dimensions(&document),
        words(&document.content).len()
    );
    assert_eq!(
        words(&document.content),
        expected_words(),
        "render original detail instead of interpolating a lost word boundary"
    );
    assert_eq!(original_dimensions(&document), (1800, 900));
}

#[tokio::test]
async fn explicit_image_density_keeps_precedence_over_preprocessing() {
    let images = ImageExtractionConfig {
        target_dpi: 150,
        min_dpi: 150,
        max_dpi: 150,
        auto_adjust_dpi: false,
        ..Default::default()
    };
    let document = read(
        fixture::localized_pdf("gap-left"),
        Some(images),
        Some(ImagePreprocessingConfig::default()),
    )
    .await;
    assert_eq!(original_dimensions(&document), (900, 450));
}

#[tokio::test]
async fn ordinary_text_without_preprocessing_keeps_the_default_render_density() {
    let document = read(fixture::pdf(12, 0), None, None).await;
    assert_eq!(original_dimensions(&document), (900, 450));
}

#[tokio::test]
async fn default_render_preserves_small_text_across_glyph_shifts() {
    for bytes in [
        fixture::localized_pdf("gap-left"),
        fixture::localized_pdf("I-right"),
        fixture::pdf(7, 0),
        fixture::pdf(7, 1),
    ] {
        let document = read(bytes, None, None).await;
        assert_eq!(original_dimensions(&document), (1800, 900));
        assert_eq!(
            document.content.split_whitespace().collect::<Vec<_>>(),
            fixture::LINES.join(" ").split_whitespace().collect::<Vec<_>>()
        );
    }
}

#[tokio::test]
async fn one_pixel_first_line_suffix_shift_at_150_dpi_merges_a_word_boundary() {
    let density = ImageExtractionConfig {
        target_dpi: 150,
        min_dpi: 150,
        max_dpi: 150,
        auto_adjust_dpi: false,
        ..Default::default()
    };
    let base = fixture::pdf(7, 0);
    let shifted = fixture::localized_pdf("gap-left");
    let chars = |bytes: &[u8]| {
        xberg_native_pdf::PdfDocument::from_bytes(bytes.to_vec())
            .unwrap()
            .extract_chars(0)
            .unwrap()
    };
    let first_shifted = fixture::LINES[0].find("in a").unwrap() + 3;
    let original_chars = chars(&base);
    let shifted_chars = chars(&shifted);
    assert_eq!(original_chars.len(), shifted_chars.len());
    for (index, (original, shifted)) in original_chars.iter().zip(&shifted_chars).enumerate() {
        assert_eq!(original.char, shifted.char);
        assert_eq!(original.origin_y, shifted.origin_y);
        let expected = if (first_shifted..fixture::LINES[0].len()).contains(&index) {
            1.0
        } else {
            0.0
        };
        let shift_pixels = (original.origin_x - shifted.origin_x) * 150.0 / 72.0;
        assert!((shift_pixels - expected).abs() < 0.001, "glyph {index}: {shift_pixels}");
    }

    for (bytes, expected) in [
        (base, fixture::LINES.join(" ")),
        (shifted, fixture::LINES.join(" ").replacen("in a", "ina", 1)),
    ] {
        let document = read(bytes, Some(density.clone()), None).await;
        assert_eq!(original_dimensions(&document), (900, 450));
        assert_eq!(
            document.content.split_whitespace().collect::<Vec<_>>(),
            expected.split_whitespace().collect::<Vec<_>>()
        );
    }
}

#[tokio::test]
async fn promoted_density_preserves_origin_and_glyph_phase_controls() {
    for bytes in [
        fixture::pdf(7, 0),
        fixture::pdf(7, 1),
        fixture::localized_pdf("I-right"),
    ] {
        let document = read(bytes, None, Some(ImagePreprocessingConfig::default())).await;
        assert_eq!(words(&document.content), expected_words());
        assert_eq!(original_dimensions(&document), (1800, 900));
    }
}

#[tokio::test]
async fn denoise_only_block_uses_its_default_target_density() {
    let preprocessing: ImagePreprocessingConfig = serde_json::from_value(serde_json::json!({"denoise": true})).unwrap();
    assert_eq!(preprocessing.target_dpi, 300);
    let document = read(fixture::localized_pdf("gap-left"), None, Some(preprocessing)).await;
    assert_eq!(original_dimensions(&document), (1800, 900));
}
