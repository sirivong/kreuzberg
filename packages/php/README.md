# Kreuzberg

[![Rust](https://img.shields.io/crates/v/kreuzberg?label=Rust&color=007ec6)](https://crates.io/crates/kreuzberg)
[![Python](https://img.shields.io/pypi/v/kreuzberg?label=Python&color=007ec6)](https://pypi.org/project/kreuzberg/)
[![TypeScript](https://img.shields.io/npm/v/@kreuzberg/node?label=TypeScript&color=007ec6)](https://www.npmjs.com/package/@kreuzberg/node)
[![WASM](https://img.shields.io/npm/v/@kreuzberg/wasm?label=WASM&color=007ec6)](https://www.npmjs.com/package/@kreuzberg/wasm)
[![Ruby](https://img.shields.io/gem/v/kreuzberg?label=Ruby&color=007ec6)](https://rubygems.org/gems/kreuzberg)
[![Java](https://img.shields.io/maven-central/v/dev.kreuzberg/kreuzberg?label=Java&color=007ec6)](https://central.sonatype.com/artifact/dev.kreuzberg/kreuzberg)
[![Go](https://img.shields.io/github/v/tag/kreuzberg-dev/kreuzberg?label=Go&color=007ec6)](https://pkg.go.dev/github.com/kreuzberg-dev/kreuzberg)
[![C#](https://img.shields.io/nuget/v/Goldziher.Kreuzberg?label=C%23&color=007ec6)](https://www.nuget.org/packages/Goldziher.Kreuzberg/)
[![Packagist](https://img.shields.io/packagist/v/kreuzberg/kreuzberg?color=007ec6)](https://packagist.org/packages/kreuzberg/kreuzberg)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-kreuzberg.dev-007ec6)](https://kreuzberg.dev/)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-007ec6)](https://discord.gg/pXxagNK2zN)

High-performance document intelligence for PHP. Extract text, metadata, and structured information from PDFs, Office documents, images, and 56 formats.

**Powered by a Rust core** – Native performance for document extraction.

> **Version 4.0.0 Release Candidate**
> This is a pre-release version. Please test the library and [report any issues](https://github.com/kreuzberg-dev/kreuzberg/issues) you encounter.

## Features

- **56+ file formats** – PDF, DOCX, XLSX, PPTX, images, HTML, XML, email, archives, and more
- **OCR support** – Tesseract integration for scanned documents and images
- **Table extraction** – Extract structured tables from PDFs and documents
- **Metadata extraction** – Rich metadata for all supported formats
- **High performance** – Rust-powered extraction
- **Batch processing** – Process multiple documents in parallel
- **Text chunking** – Intelligent text segmentation for RAG applications
- **Embeddings** – Generate vector embeddings for semantic search
- **Type-safe** – Full PHP 8.2+ type hints and readonly classes

## System Requirements

- PHP 8.2 or higher
- Kreuzberg PHP extension (kreuzberg.so/.dll)
- Tesseract OCR (optional, for OCR functionality)
- ONNX Runtime (optional, for embeddings)

### Installing Tesseract

```bash
# macOS
brew install tesseract

# Ubuntu/Debian
sudo apt install tesseract-ocr

# Windows
# Download from: https://github.com/UB-Mannheim/tesseract/wiki
```

### Installing ONNX Runtime

```bash
# macOS
brew install onnxruntime

# Ubuntu/Debian
sudo apt install libonnxruntime libonnxruntime-dev

# Windows (MSVC)
scoop install onnxruntime
# OR download from https://github.com/microsoft/onnxruntime/releases
```

## Installation

### Option 1: Using PIE (Recommended)

[PIE (PHP Installer for Extensions)](https://github.com/php/pie) is the modern way to install PHP extensions:

```bash
# Install PIE if you haven't already
composer global require php/pie

# Install Kreuzberg extension
pie install kreuzberg/kreuzberg
```

PIE will automatically:
- Download the extension source or pre-built binary
- Compile it for your system (if needed)
- Install it to the correct PHP extension directory
- Update your php.ini configuration

Then install the PHP library:

```bash
composer require kreuzberg/kreuzberg
```

### Option 2: Manual Installation

Download the appropriate extension for your platform from the [releases page](https://github.com/kreuzberg-dev/kreuzberg/releases).

Add to your `php.ini`:

```ini
extension=kreuzberg.so  ; Linux/macOS
; or
extension=kreuzberg.dll  ; Windows
```

Then install the PHP library:

```bash
composer require kreuzberg/kreuzberg
```

### Verifying Installation

Check that the extension is loaded:

```bash
php -m | grep kreuzberg
```

Or use the version function:

```php
<?php
echo kreuzberg_version(); // Should output: 4.0.0-rc.20
```

## Quick Start

### Simple Extraction

```php
<?php

use Kreuzberg\Kreuzberg;

$kreuzberg = new Kreuzberg();
$result = $kreuzberg->extractFile('document.pdf');

echo $result->content;
print_r($result->metadata);
print_r($result->tables);
```

### Procedural API

```php
<?php

use function Kreuzberg\extract_file;

$result = extract_file('document.pdf');
echo $result->content;
```

### Batch Processing

```php
<?php

use Kreuzberg\Kreuzberg;

$kreuzberg = new Kreuzberg();
$files = ['doc1.pdf', 'doc2.docx', 'doc3.xlsx'];
$results = $kreuzberg->batchExtractFiles($files);

foreach ($results as $result) {
    echo $result->content . "\n";
}
```

## OCR Support

### Basic OCR with Tesseract

```php
<?php

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\OcrConfig;

$config = new ExtractionConfig(
    ocr: new OcrConfig(
        backend: 'tesseract',
        language: 'eng'
    )
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('scanned.pdf');
```

### Advanced OCR Configuration

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\OcrConfig;
use Kreuzberg\Config\TesseractConfig;
use Kreuzberg\Config\ImagePreprocessingConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    ocr: new OcrConfig(
        backend: 'tesseract',
        language: 'eng',
        tesseractConfig: new TesseractConfig(
            psm: 6,
            enableTableDetection: true,
            tesseditCharWhitelist: '0123456789'
        ),
        imagePreprocessing: new ImagePreprocessingConfig(
            targetDpi: 300,
            denoise: true,
            sharpen: true
        )
    )
);

$result = extract_file('invoice.pdf', config: $config);
```

## Table Extraction

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\OcrConfig;
use Kreuzberg\Config\TesseractConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    ocr: new OcrConfig(
        backend: 'tesseract',
        tesseractConfig: new TesseractConfig(
            enableTableDetection: true
        )
    )
);

$result = extract_file('financial_report.pdf', config: $config);

foreach ($result->tables as $table) {
    echo "Table on page {$table->pageNumber}:\n";
    echo $table->markdown . "\n\n";

    // Or access raw cells
    foreach ($table->cells as $row) {
        foreach ($row as $cell) {
            echo $cell . "\t";
        }
        echo "\n";
    }
}
```

## Text Chunking & Embeddings

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ChunkingConfig;
use Kreuzberg\Config\EmbeddingConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    chunking: new ChunkingConfig(
        maxChunkSize: 512,
        chunkOverlap: 50,
        respectSentences: true
    ),
    embedding: new EmbeddingConfig(
        model: 'all-MiniLM-L6-v2',
        normalize: true
    )
);

$result = extract_file('long_document.pdf', config: $config);

foreach ($result->chunks as $chunk) {
    echo "Chunk {$chunk->metadata->chunkIndex}:\n";
    echo $chunk->content . "\n";

    if ($chunk->embedding !== null) {
        echo "Embedding dimension: " . count($chunk->embedding) . "\n";
    }
}
```

## Image Extraction

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ImageExtractionConfig;
use Kreuzberg\Config\OcrConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    imageExtraction: new ImageExtractionConfig(
        extractImages: true,
        performOcr: true,  // OCR on extracted images
        minWidth: 100,
        minHeight: 100
    ),
    ocr: new OcrConfig(backend: 'tesseract', language: 'eng')
);

$result = extract_file('presentation.pptx', config: $config);

foreach ($result->images as $image) {
    echo "Image {$image->imageIndex} from page {$image->pageNumber}\n";
    echo "Format: {$image->format}, Size: {$image->width}x{$image->height}\n";

    // Save image
    file_put_contents("image_{$image->imageIndex}.{$image->format}", $image->data);

    // Access OCR result if available
    if ($image->ocrResult !== null) {
        echo "OCR Text: {$image->ocrResult->content}\n";
    }
}
```

## Page Extraction

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\PageConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    page: new PageConfig(
        extractPages: true,
        insertPageMarkers: true,
        markerFormat: '--- Page {page_number} ---'
    )
);

$result = extract_file('report.pdf', config: $config);

foreach ($result->pages as $page) {
    echo "=== Page {$page->pageNumber} ===\n";
    echo $page->content . "\n";

    echo "Tables: " . count($page->tables) . "\n";
    echo "Images: " . count($page->images) . "\n";
}
```

## Language Detection

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\LanguageDetectionConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    languageDetection: new LanguageDetectionConfig(
        enabled: true,
        maxLanguages: 3,
        confidenceThreshold: 0.8
    )
);

$result = extract_file('multilingual.pdf', config: $config);

if ($result->detectedLanguages !== null) {
    echo "Detected languages: " . implode(', ', $result->detectedLanguages) . "\n";
}
```

## Keyword Extraction

```php
<?php

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\KeywordConfig;
use function Kreuzberg\extract_file;

$config = new ExtractionConfig(
    keyword: new KeywordConfig(
        maxKeywords: 10,
        minScore: 0.0,
        language: 'en'
    )
);

$result = extract_file('article.pdf', config: $config);

// Keywords are in metadata
if ($result->metadata->keywords !== null) {
    echo "Keywords: " . implode(', ', $result->metadata->keywords) . "\n";
}
```

## Supported Formats

| Format | Extension | MIME Type | Notes |
|--------|-----------|-----------|-------|
| PDF | .pdf | application/pdf | Full support with OCR fallback |
| Word | .docx, .doc | application/vnd.openxmlformats-officedocument.wordprocessingml.document | Text, tables, images |
| Excel | .xlsx, .xls | application/vnd.openxmlformats-officedocument.spreadsheetml.sheet | Multiple sheets |
| PowerPoint | .pptx, .ppt | application/vnd.openxmlformats-officedocument.presentationml.presentation | Slides, notes |
| Images | .png, .jpg, .jpeg, .tiff, .bmp, .webp | image/* | OCR support |
| HTML | .html, .htm | text/html | Metadata extraction |
| Markdown | .md | text/markdown | Preserves structure |
| Email | .eml, .msg | message/rfc822 | Attachments, headers |
| Archives | .zip, .tar, .7z | application/zip | File listing |
| XML | .xml | text/xml | Structure analysis |
| CSV | .csv | text/csv | Delimiter detection |
| JSON | .json | application/json | Schema extraction |

...and 40+ more formats.

## API Reference

### Main Classes

- **`Kreuzberg`**: Main OOP API class
- **`ExtractionResult`**: Extraction result with content, metadata, tables
- **`Metadata`**: Document metadata (title, author, dates, etc.)
- **`Table`**: Extracted table structure
- **`Chunk`**: Text chunk with embedding
- **`ExtractedImage`**: Extracted image with optional OCR

### Configuration Classes

- **`ExtractionConfig`**: Main configuration
- **`OcrConfig`**: OCR settings
- **`TesseractConfig`**: Tesseract-specific settings
- **`ImagePreprocessingConfig`**: Image preprocessing options
- **`PdfConfig`**: PDF extraction settings
- **`ChunkingConfig`**: Text chunking settings
- **`EmbeddingConfig`**: Embedding generation settings
- **`ImageExtractionConfig`**: Image extraction settings
- **`PageConfig`**: Page extraction settings
- **`LanguageDetectionConfig`**: Language detection settings
- **`KeywordConfig`**: Keyword extraction settings

### Procedural Functions

```php
// Extraction
extract_file(string $filePath, ?string $mimeType = null, ?ExtractionConfig $config = null): ExtractionResult
extract_bytes(string $data, string $mimeType, ?ExtractionConfig $config = null): ExtractionResult
batch_extract_files(array $paths, ?ExtractionConfig $config = null): array
batch_extract_bytes(array $dataList, array $mimeTypes, ?ExtractionConfig $config = null): array

// Utilities
detect_mime_type(string $data): string
detect_mime_type_from_path(string $path): string
```

## Error Handling

```php
<?php

use Kreuzberg\Exceptions\KreuzbergException;
use function Kreuzberg\extract_file;

try {
    $result = extract_file('document.pdf');
    echo $result->content;
} catch (KreuzbergException $e) {
    echo "Extraction failed: {$e->getMessage()}\n";
    echo "Error code: {$e->getCode()}\n";
}
```

## Performance Tips

1. **Use batch processing** for multiple files
2. **Disable unnecessary features** (OCR, embeddings) if not needed
3. **Set appropriate chunk sizes** for your use case
4. **Use page extraction** only when you need per-page content
5. **Limit image extraction** with min width/height filters

## Troubleshooting

### Extension Not Loaded

**Problem:** `extension_loaded('kreuzberg')` returns false

**Solutions:**

1. Verify extension file exists in PHP extension directory:
   ```bash
   php -i | grep extension_dir
   ```

2. Check the extension file is in the correct location:
   ```bash
   ls -la $(php -i | grep extension_dir | cut -d' ' -f 5)/kreuzberg.so
   ```

3. Verify php.ini configuration:
   ```bash
   php --ini
   ```

4. Add extension to php.ini:
   ```ini
   extension=kreuzberg.so
   ```

5. Restart PHP:
   ```bash
   sudo systemctl restart php-fpm
   ```

6. Check for loading errors:
   ```bash
   php -m 2>&1 | grep kreuzberg
   ```

### Version Mismatch

**Problem:** Extension and PHP package versions don't match

**Solution:**
```bash
# Check extension version
php -r "echo phpversion('kreuzberg');"

# Check package version
composer show kreuzberg/kreuzberg

# Update to match
composer update kreuzberg/kreuzberg
```

### Memory Limits

**Problem:** Out of memory errors when processing large files

**Solutions:**

1. Increase PHP memory limit:
   ```ini
   memory_limit = 512M  ; or higher
   ```

2. Use chunking for large documents:
   ```php
   $config = new ExtractionConfig(
       chunking: new ChunkingConfig(maxChunkSize: 1000)
   );
   ```

3. Process files in batches:
   ```php
   $chunks = array_chunk($files, 10);
   foreach ($chunks as $chunk) {
       $results = batch_extract_files($chunk);
       // Process and clear memory
       unset($results);
       gc_collect_cycles();
   }
   ```

### OCR Not Working

**Problem:** Tesseract OCR not detecting text

**Solutions:**

1. Verify Tesseract is installed:
   ```bash
   tesseract --version
   ```

2. Check language data files:
   ```bash
   # macOS
   ls /usr/local/share/tessdata/

   # Linux
   ls /usr/share/tesseract-ocr/*/tessdata/
   ```

3. Install missing language packs:
   ```bash
   # macOS
   brew install tesseract-lang

   # Ubuntu/Debian
   sudo apt install tesseract-ocr-eng tesseract-ocr-fra
   ```

4. Specify correct language code:
   ```php
   $config = new ExtractionConfig(
       ocr: new OcrConfig(
           backend: 'tesseract',
           language: 'eng'  // Use correct ISO 639-3 code
       )
   );
   ```

5. Try different PSM modes:
   ```php
   $config = new ExtractionConfig(
       ocr: new OcrConfig(
           backend: 'tesseract',
           language: 'eng',
           tesseractConfig: new TesseractConfig(
               psm: 6  // Try values 3, 6, 11
           )
       )
   );
   ```

### Permission Errors

**Problem:** Cannot read/write files

**Solutions:**

1. Check file permissions:
   ```bash
   ls -la document.pdf
   ```

2. Ensure PHP can read the file:
   ```bash
   sudo chown www-data:www-data document.pdf
   sudo chmod 644 document.pdf
   ```

3. Check directory permissions for writing:
   ```bash
   sudo chown www-data:www-data output_directory/
   sudo chmod 755 output_directory/
   ```

### Poor OCR Accuracy

**Problem:** OCR produces incorrect or garbled text

**Solutions:**

1. Increase target DPI:
   ```php
   $config = new ExtractionConfig(
       ocr: new OcrConfig(
           backend: 'tesseract',
           language: 'eng',
           imagePreprocessing: new ImagePreprocessingConfig(
               targetDpi: 600  // Higher DPI for better accuracy
           )
       )
   );
   ```

2. Enable denoising:
   ```php
   $config = new ExtractionConfig(
       ocr: new OcrConfig(
           backend: 'tesseract',
           language: 'eng',
           imagePreprocessing: new ImagePreprocessingConfig(
               denoise: true
           )
       )
   );
   ```

3. Use character whitelisting:
   ```php
   $config = new ExtractionConfig(
       ocr: new OcrConfig(
           backend: 'tesseract',
           language: 'eng',
           tesseractConfig: new TesseractConfig(
               tesseditCharWhitelist: 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 '
           )
       )
   );
   ```

### Common Error Messages

**"Extension not loaded"**
- Install and enable the Kreuzberg extension in php.ini

**"File not found"**
- Check file path is absolute or relative to current directory
- Verify file exists and is readable

**"Unsupported format"**
- Check MIME type is supported
- Try specifying MIME type explicitly:
  ```php
  $result = $kreuzberg->extractFile('file.unknown', 'application/pdf');
  ```

**"OCR backend not available"**
- Install Tesseract OCR
- Verify Tesseract is in system PATH

**"Out of memory"**
- Increase PHP memory_limit
- Use chunking for large documents
- Process files in smaller batches

## Development

### Running Tests

```bash
composer test
```

### Code Quality

```bash
# PHPStan analysis
composer lint

# Code formatting
composer format

# Check formatting
composer format:check
```

## Documentation

For comprehensive documentation, visit [https://kreuzberg.dev](https://kreuzberg.dev)

## License

MIT License - see [LICENSE](LICENSE) for details.
