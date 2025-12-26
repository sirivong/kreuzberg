<?php

declare(strict_types=1);

/**
 * Advanced Configuration Example
 *
 * Demonstrates complex configurations with all available options.
 * Shows how to fine-tune extraction behavior for specific use cases.
 *
 * This example covers:
 * - PDF-specific configuration
 * - Image extraction configuration
 * - Page extraction with markers
 * - Language detection
 * - Keyword extraction
 * - Combining multiple configuration options
 *
 * @package Kreuzberg
 */

require_once __DIR__ . '/../../packages/php/vendor/autoload.php';

use Kreuzberg\Config\ChunkingConfig;
use Kreuzberg\Config\EmbeddingConfig;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ImageExtractionConfig;
use Kreuzberg\Config\KeywordConfig;
use Kreuzberg\Config\LanguageDetectionConfig;
use Kreuzberg\Config\OcrConfig;
use Kreuzberg\Config\PageConfig;
use Kreuzberg\Config\PdfConfig;
use Kreuzberg\Config\TesseractConfig;
use Kreuzberg\Exceptions\KreuzbergException;
use Kreuzberg\Kreuzberg;

// =============================================================================
// Example 1: PDF-Specific Configuration
// =============================================================================

echo "=== Example 1: PDF-Specific Configuration ===\n\n";

try {
    // Configure PDF extraction with page range and image extraction
    $config = new ExtractionConfig(
        pdf: new PdfConfig(
            extractImages: true,
            extractMetadata: true,
            ocrFallback: false,
            startPage: 1,        // Start from page 1 (0-indexed)
            endPage: 5,          // Extract only first 5 pages
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/sample.pdf');

    echo "Extracted pages 1-5:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Pages: {$result->metadata->pageCount}\n";
    echo "  Images found: " . count($result->images ?? []) . "\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 2: Advanced OCR Configuration
// =============================================================================

echo "=== Example 2: Advanced OCR Configuration ===\n\n";

try {
    // Configure OCR with Tesseract-specific options
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng+deu',  // Support English and German
            tesseractConfig: new TesseractConfig(
                psm: 6,                      // Assume uniform block of text
                enableTableDetection: true,  // Detect tables in scanned documents
            ),
        ),
        pdf: new PdfConfig(
            ocrFallback: true,  // Use OCR if text extraction fails
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/scanned.pdf');

    echo "OCR extraction complete:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Tables found: " . count($result->tables) . "\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 3: Image Extraction with OCR
// =============================================================================

echo "=== Example 3: Image Extraction with OCR ===\n\n";

try {
    // Configure image extraction with size filters and OCR
    $config = new ExtractionConfig(
        imageExtraction: new ImageExtractionConfig(
            extractImages: true,
            performOcr: true,
            minWidth: 100,   // Minimum width in pixels
            minHeight: 100,  // Minimum height in pixels
        ),
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/presentation.pptx');

    echo "Image extraction results:\n";
    echo "  Total images: " . count($result->images ?? []) . "\n";

    if ($result->images !== null) {
        foreach (array_slice($result->images, 0, 3) as $i => $image) {
            echo "\n  Image " . ($i + 1) . ":\n";
            echo "    Format: {$image->format}\n";
            echo "    Size: {$image->width}x{$image->height} pixels\n";
            echo "    Page: {$image->pageNumber}\n";

            if ($image->ocrResult !== null) {
                echo "    OCR text length: " . strlen($image->ocrResult->content) . " characters\n";
                echo "    First 100 chars: " . substr($image->ocrResult->content, 0, 100) . "...\n";
            }
        }
    }

    echo "\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 4: Page Extraction with Markers
// =============================================================================

echo "=== Example 4: Page Extraction with Markers ===\n\n";

try {
    // Configure page-by-page extraction with custom markers
    $config = new ExtractionConfig(
        page: new PageConfig(
            extractPages: true,
            insertPageMarkers: true,
            markerFormat: '--- Page {page_number} ---',
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/sample.pdf');

    echo "Page extraction results:\n";
    echo "  Total pages: " . count($result->pages ?? []) . "\n";

    if ($result->pages !== null) {
        foreach (array_slice($result->pages, 0, 2) as $page) {
            echo "\n=== Page {$page->pageNumber} ===\n";
            echo "Content length: " . strlen($page->content) . " characters\n";
            echo "Tables: " . count($page->tables) . "\n";
            echo "Images: " . count($page->images) . "\n";
            echo "\nFirst 200 characters:\n";
            echo substr($page->content, 0, 200) . "...\n";
        }
    }

    // The full content also has page markers
    echo "\n--- Content with page markers ---\n";
    echo substr($result->content, 0, 500) . "...\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 5: Language Detection
// =============================================================================

echo "=== Example 5: Language Detection ===\n\n";

try {
    // Configure language detection
    $config = new ExtractionConfig(
        languageDetection: new LanguageDetectionConfig(
            enabled: true,
            maxLanguages: 3,           // Detect up to 3 languages
            confidenceThreshold: 0.8,  // Minimum confidence level
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/multilingual.pdf');

    echo "Language detection results:\n";

    if ($result->detectedLanguages !== null) {
        echo "  Detected languages: " . implode(', ', $result->detectedLanguages) . "\n";
    }

    echo "  Primary language: " . ($result->metadata->language ?? 'N/A') . "\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 6: Keyword Extraction
// =============================================================================

echo "=== Example 6: Keyword Extraction ===\n\n";

try {
    // Configure keyword extraction
    $config = new ExtractionConfig(
        keyword: new KeywordConfig(
            enabled: true,
            algorithm: 'rake',  // RAKE (Rapid Automatic Keyword Extraction) algorithm
            maxKeywords: 10,    // Extract top 10 keywords
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/article.pdf');

    echo "Keyword extraction results:\n";

    if ($result->metadata->keywords !== null) {
        echo "  Keywords: " . implode(', ', $result->metadata->keywords) . "\n";
    }

    echo "\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 7: Comprehensive Configuration (All Options)
// =============================================================================

echo "=== Example 7: Comprehensive Configuration (All Options) ===\n\n";

try {
    // Create a configuration using ALL available options
    $config = new ExtractionConfig(
        // OCR configuration
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
                enableTableDetection: true,
            ),
        ),

        // PDF-specific settings
        pdf: new PdfConfig(
            extractImages: true,
            extractMetadata: true,
            ocrFallback: true,
        ),

        // Text chunking
        chunking: new ChunkingConfig(
            maxChunkSize: 512,
            chunkOverlap: 50,
            respectSentences: true,
            respectParagraphs: true,
        ),

        // Embedding generation
        embedding: new EmbeddingConfig(
            model: 'all-MiniLM-L6-v2',
            normalize: true,
            batchSize: 32,
        ),

        // Image extraction
        imageExtraction: new ImageExtractionConfig(
            extractImages: true,
            performOcr: true,
            minWidth: 100,
            minHeight: 100,
        ),

        // Page extraction
        page: new PageConfig(
            extractPages: true,
            insertPageMarkers: true,
            markerFormat: '=== Page {page_number} ===',
        ),

        // Language detection
        languageDetection: new LanguageDetectionConfig(
            enabled: true,
            maxLanguages: 3,
            confidenceThreshold: 0.8,
        ),

        // Keyword extraction
        keyword: new KeywordConfig(
            enabled: true,
            algorithm: 'rake',
            maxKeywords: 10,
        ),

        // General extraction options
        extractImages: true,
        extractTables: true,
        preserveFormatting: false,
        outputFormat: 'markdown',
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/sample.pdf');

    echo "Comprehensive extraction results:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  MIME type: {$result->mimeType}\n";
    echo "  Tables: " . count($result->tables) . "\n";
    echo "  Images: " . count($result->images ?? []) . "\n";
    echo "  Pages: " . count($result->pages ?? []) . "\n";
    echo "  Chunks: " . count($result->chunks ?? []) . "\n";
    echo "  Detected languages: " . (
        $result->detectedLanguages
            ? implode(', ', $result->detectedLanguages)
            : 'N/A'
    ) . "\n";
    echo "  Keywords: " . (
        $result->metadata->keywords
            ? implode(', ', $result->metadata->keywords)
            : 'N/A'
    ) . "\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 8: Dynamic Configuration Based on File Type
// =============================================================================

echo "=== Example 8: Dynamic Configuration Based on File Type ===\n\n";

try {
    $filePath = __DIR__ . '/../sample-documents/sample.pdf';

    // Detect MIME type first
    $mimeType = \Kreuzberg\detect_mime_type_from_path($filePath);
    echo "Detected MIME type: {$mimeType}\n";

    // Configure based on file type
    $config = match (true) {
        str_contains($mimeType, 'pdf') => new ExtractionConfig(
            pdf: new PdfConfig(extractImages: true),
            ocr: new OcrConfig(backend: 'tesseract', language: 'eng'),
        ),
        str_contains($mimeType, 'image') => new ExtractionConfig(
            ocr: new OcrConfig(backend: 'tesseract', language: 'eng'),
        ),
        str_contains($mimeType, 'spreadsheet') => new ExtractionConfig(
            extractTables: true,
        ),
        default => new ExtractionConfig(),
    };

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile($filePath);

    echo "Extracted with dynamic config:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

echo "Done!\n";
