<?php

declare(strict_types=1);

/**
 * OCR Example
 *
 * Demonstrates OCR extraction from scanned PDFs and images using Tesseract.
 * Shows various OCR configurations and use cases.
 *
 * This example covers:
 * - Basic OCR extraction
 * - Multi-language OCR
 * - Advanced Tesseract configuration
 * - Image preprocessing for better OCR results
 * - OCR fallback for hybrid documents
 * - Table detection in scanned documents
 * - Character whitelisting/blacklisting
 *
 * @package Kreuzberg
 */

require_once __DIR__ . '/../../packages/php/vendor/autoload.php';

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ImagePreprocessingConfig;
use Kreuzberg\Config\OcrConfig;
use Kreuzberg\Config\PdfConfig;
use Kreuzberg\Config\TesseractConfig;
use Kreuzberg\Exceptions\KreuzbergException;
use Kreuzberg\Kreuzberg;
use function Kreuzberg\extract_file;

// =============================================================================
// Example 1: Basic OCR Extraction (OOP API)
// =============================================================================

echo "=== Example 1: Basic OCR Extraction (OOP API) ===\n\n";

try {
    // Configure basic OCR with Tesseract
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',  // English language
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/scanned_document.pdf');

    echo "OCR extraction complete:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  MIME type: {$result->mimeType}\n";
    echo "  Page count: {$result->metadata->pageCount}\n";
    echo "\nFirst 300 characters:\n";
    echo substr($result->content, 0, 300) . "...\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 2: Basic OCR Extraction (Procedural API)
// =============================================================================

echo "=== Example 2: Basic OCR Extraction (Procedural API) ===\n\n";

try {
    // Use procedural API
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
        ),
    );

    $result = extract_file(
        __DIR__ . '/../sample-documents/scanned_document.pdf',
        config: $config
    );

    echo "Content extracted: " . strlen($result->content) . " characters\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 3: Multi-Language OCR
// =============================================================================

echo "=== Example 3: Multi-Language OCR ===\n\n";

try {
    // Configure for English and German
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng+deu',  // Multiple languages separated by +
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/german_document.pdf');

    echo "Multi-language OCR:\n";
    echo "  Languages: eng+deu\n";
    echo "  Content length: " . strlen($result->content) . " characters\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 4: OCR with Page Segmentation Mode (PSM)
// =============================================================================

echo "=== Example 4: OCR with Page Segmentation Mode ===\n\n";

try {
    // Different PSM modes for different document types:
    // PSM 3: Fully automatic page segmentation (default)
    // PSM 6: Assume a single uniform block of text
    // PSM 11: Sparse text. Find as much text as possible in no particular order
    // PSM 13: Raw line (treat the image as a single text line)

    $psmModes = [
        3 => 'Fully automatic page segmentation',
        6 => 'Single uniform block of text',
        11 => 'Sparse text',
        13 => 'Raw line (single text line)',
    ];

    foreach ($psmModes as $psm => $description) {
        $config = new ExtractionConfig(
            ocr: new OcrConfig(
                backend: 'tesseract',
                language: 'eng',
                tesseractConfig: new TesseractConfig(
                    psm: $psm,
                ),
            ),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/scanned.pdf');

        echo "PSM {$psm} ({$description}):\n";
        echo "  Extracted: " . strlen($result->content) . " characters\n\n";
    }

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 5: Table Detection in Scanned Documents
// =============================================================================

echo "=== Example 5: Table Detection in Scanned Documents ===\n\n";

try {
    // Enable table detection for scanned documents
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
                enableTableDetection: true,  // Enable table detection
            ),
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/scanned_table.pdf');

    echo "OCR with table detection:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Tables found: " . count($result->tables) . "\n";

    foreach ($result->tables as $i => $table) {
        echo "\n  Table " . ($i + 1) . ":\n";
        echo "    Page: {$table->pageNumber}\n";
        echo "    Rows: {$table->rowCount}, Columns: {$table->columnCount}\n";
        echo "    Markdown:\n";
        echo "    " . str_replace("\n", "\n    ", $table->markdown) . "\n";
    }

    echo "\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 6: OCR Fallback for Hybrid Documents
// =============================================================================

echo "=== Example 6: OCR Fallback for Hybrid Documents ===\n\n";

try {
    // Use OCR as fallback when normal text extraction fails
    // Useful for documents that mix digital text and scanned images
    $config = new ExtractionConfig(
        pdf: new PdfConfig(
            ocrFallback: true,  // Use OCR only if text extraction fails
        ),
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/mixed_document.pdf');

    echo "OCR fallback mode:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Note: OCR only applied where needed\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 7: Character Whitelisting and Blacklisting
// =============================================================================

echo "=== Example 7: Character Whitelisting and Blacklisting ===\n\n";

try {
    // Whitelist: Extract only numeric characters (useful for invoices, receipts)
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
                tesseditCharWhitelist: '0123456789.,',  // Only digits and punctuation
            ),
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/invoice_scan.pdf');

    echo "Character whitelist (numbers only):\n";
    echo "  Extracted: {$result->content}\n\n";

    // Blacklist: Remove certain characters that cause issues
    $config2 = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
                tesseditCharBlacklist: '|@#$%',  // Exclude these characters
            ),
        ),
    );

    $kreuzberg2 = new Kreuzberg($config2);
    $result2 = $kreuzberg2->extractFile(__DIR__ . '/../sample-documents/scanned.pdf');

    echo "Character blacklist (exclude special chars):\n";
    echo "  Content length: " . strlen($result2->content) . " characters\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 8: OCR Engine Mode (OEM)
// =============================================================================

echo "=== Example 8: OCR Engine Mode ===\n\n";

try {
    // OEM modes:
    // 0: Legacy engine only
    // 1: Neural nets LSTM engine only (default in Tesseract 4.0+)
    // 2: Legacy + LSTM engines
    // 3: Default, based on what is available

    $oemModes = [
        1 => 'LSTM engine only (best quality)',
        2 => 'Legacy + LSTM engines',
        3 => 'Default (automatic)',
    ];

    foreach ($oemModes as $oem => $description) {
        $config = new ExtractionConfig(
            ocr: new OcrConfig(
                backend: 'tesseract',
                language: 'eng',
                tesseractConfig: new TesseractConfig(
                    oem: $oem,
                    psm: 6,
                ),
            ),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/scanned.pdf');

        echo "OEM {$oem} ({$description}):\n";
        echo "  Extracted: " . strlen($result->content) . " characters\n\n";
    }

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 9: OCR from Image Files
// =============================================================================

echo "=== Example 9: OCR from Image Files ===\n\n";

try {
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
            ),
        ),
    );

    $imageFiles = [
        __DIR__ . '/../sample-documents/screenshot.png',
        __DIR__ . '/../sample-documents/photo.jpg',
        __DIR__ . '/../sample-documents/scan.tiff',
    ];

    $kreuzberg = new Kreuzberg($config);

    foreach ($imageFiles as $imageFile) {
        try {
            $result = $kreuzberg->extractFile($imageFile);
            $filename = basename($imageFile);

            echo "{$filename}:\n";
            echo "  MIME type: {$result->mimeType}\n";
            echo "  Content length: " . strlen($result->content) . " characters\n";
            echo "  Preview: " . substr($result->content, 0, 100) . "...\n\n";

        } catch (KreuzbergException $e) {
            echo basename($imageFile) . ": Error - {$e->getMessage()}\n\n";
        }
    }

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 10: Image Preprocessing for Better OCR
// =============================================================================

echo "=== Example 10: Image Preprocessing for Better OCR ===\n\n";

try {
    // Configure image preprocessing to improve OCR accuracy
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            imagePreprocessing: new ImagePreprocessingConfig(
                // Preprocessing options would go here
                // (actual options depend on the implementation)
            ),
            tesseractConfig: new TesseractConfig(
                psm: 6,
            ),
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/low_quality_scan.pdf');

    echo "OCR with image preprocessing:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Note: Preprocessing can improve accuracy on poor quality scans\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 11: Comprehensive OCR Configuration
// =============================================================================

echo "=== Example 11: Comprehensive OCR Configuration ===\n\n";

try {
    // Create a comprehensive OCR configuration
    $config = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng+fra+deu',  // Multi-language support
            tesseractConfig: new TesseractConfig(
                psm: 6,                      // Uniform block of text
                oem: 1,                      // LSTM engine only
                enableTableDetection: true,  // Detect tables
            ),
        ),
        pdf: new PdfConfig(
            ocrFallback: true,  // Use OCR as fallback
        ),
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile(__DIR__ . '/../sample-documents/complex_scan.pdf');

    echo "Comprehensive OCR results:\n";
    echo "  Content length: " . strlen($result->content) . " characters\n";
    echo "  Tables: " . count($result->tables) . "\n";
    echo "  Language: " . ($result->metadata->language ?? 'N/A') . "\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

// =============================================================================
// Example 12: Comparing OCR Configurations
// =============================================================================

echo "=== Example 12: Comparing OCR Configurations ===\n\n";

try {
    $filePath = __DIR__ . '/../sample-documents/scanned.pdf';

    // Config 1: Fast (PSM 3, automatic)
    $fastConfig = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(psm: 3),
        ),
    );

    // Config 2: Accurate (PSM 6, LSTM only)
    $accurateConfig = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: 'eng',
            tesseractConfig: new TesseractConfig(
                psm: 6,
                oem: 1,
            ),
        ),
    );

    echo "Comparing configurations:\n\n";

    // Fast extraction
    $startTime = microtime(true);
    $kreuzberg1 = new Kreuzberg($fastConfig);
    $result1 = $kreuzberg1->extractFile($filePath);
    $time1 = microtime(true) - $startTime;

    echo "Fast config (PSM 3):\n";
    echo "  Time: " . round($time1, 3) . " seconds\n";
    echo "  Content length: " . strlen($result1->content) . " characters\n\n";

    // Accurate extraction
    $startTime = microtime(true);
    $kreuzberg2 = new Kreuzberg($accurateConfig);
    $result2 = $kreuzberg2->extractFile($filePath);
    $time2 = microtime(true) - $startTime;

    echo "Accurate config (PSM 6, OEM 1):\n";
    echo "  Time: " . round($time2, 3) . " seconds\n";
    echo "  Content length: " . strlen($result2->content) . " characters\n\n";

} catch (KreuzbergException $e) {
    echo "Error: {$e->getMessage()}\n\n";
}

echo "Done!\n";
