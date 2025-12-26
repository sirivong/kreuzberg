```php
<?php

declare(strict_types=1);

/**
 * LanguageDetectionConfig - Language Detection
 *
 * Automatically detect the languages present in a document.
 * Useful for multilingual documents and routing to appropriate OCR languages.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\LanguageDetectionConfig;

// Basic language detection
$config = new ExtractionConfig(
    languageDetection: new LanguageDetectionConfig(
        enabled: true
    )
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('multilingual.pdf');

echo "Detected languages:\n";
foreach ($result->detectedLanguages ?? [] as $lang) {
    echo "  - $lang\n";
}
echo "\n";

// Advanced language detection with confidence threshold
$advancedConfig = new ExtractionConfig(
    languageDetection: new LanguageDetectionConfig(
        enabled: true,
        maxLanguages: 3,           // Detect up to 3 languages
        confidenceThreshold: 0.8   // Only report if 80% confident
    )
);

$kreuzberg = new Kreuzberg($advancedConfig);
$result = $kreuzberg->extractFile('document.pdf');

if (!empty($result->detectedLanguages)) {
    echo "High-confidence languages detected:\n";
    echo implode(', ', $result->detectedLanguages) . "\n\n";
} else {
    echo "No languages detected with sufficient confidence\n\n";
}

// Use detected language for OCR
use Kreuzberg\Config\OcrConfig;

// First pass: detect language
$detectConfig = new ExtractionConfig(
    languageDetection: new LanguageDetectionConfig(enabled: true)
);

$kreuzberg = new Kreuzberg($detectConfig);
$result = $kreuzberg->extractFile('scanned.pdf');

// Second pass: use detected language for OCR
if (!empty($result->detectedLanguages)) {
    $primaryLanguage = $result->detectedLanguages[0];
    echo "Primary language detected: $primaryLanguage\n";
    echo "Re-processing with OCR optimized for $primaryLanguage...\n";

    $ocrConfig = new ExtractionConfig(
        ocr: new OcrConfig(
            backend: 'tesseract',
            language: $primaryLanguage
        )
    );

    $kreuzberg = new Kreuzberg($ocrConfig);
    $result = $kreuzberg->extractFile('scanned.pdf');
    echo "OCR extraction complete\n";
}

// Route documents by language
$files = ['doc1.pdf', 'doc2.pdf', 'doc3.pdf'];
$languageMap = [];

foreach ($files as $file) {
    if (!file_exists($file)) continue;

    $result = $kreuzberg->extractFile($file);
    $lang = $result->detectedLanguages[0] ?? 'unknown';

    if (!isset($languageMap[$lang])) {
        $languageMap[$lang] = [];
    }
    $languageMap[$lang][] = $file;
}

echo "\nDocuments grouped by language:\n";
foreach ($languageMap as $lang => $docs) {
    echo "$lang: " . implode(', ', $docs) . "\n";
}
```
