```php
<?php

declare(strict_types=1);

/**
 * Batch Document Processing
 *
 * Process multiple documents in parallel for maximum performance.
 * Kreuzberg's batch API uses multiple threads to extract documents concurrently.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use function Kreuzberg\batch_extract_files;
use function Kreuzberg\batch_extract_bytes;

// Simple batch processing - procedural API
$files = [
    'document1.pdf',
    'document2.docx',
    'document3.xlsx',
    'presentation.pptx',
];

// Filter to only existing files
$files = array_filter($files, 'file_exists');

if (!empty($files)) {
    echo "Processing " . count($files) . " files in batch...\n\n";

    $start = microtime(true);
    $results = batch_extract_files($files);
    $elapsed = microtime(true) - $start;

    echo "Batch extraction completed in " . number_format($elapsed, 3) . " seconds\n";
    echo "Average: " . number_format($elapsed / count($files), 3) . " seconds per file\n\n";

    foreach ($results as $index => $result) {
        $filename = basename($files[$index]);
        echo "$filename:\n";
        echo "  Content: " . strlen($result->content) . " chars\n";
        echo "  Tables: " . count($result->tables) . "\n";
        echo "  MIME: " . $result->mimeType . "\n\n";
    }
}

// Batch processing with configuration - OOP API
$config = new ExtractionConfig(
    extractTables: true,
    extractImages: false  // Skip images for faster processing
);

$kreuzberg = new Kreuzberg($config);

$pdfFiles = glob('*.pdf');
if (!empty($pdfFiles)) {
    echo "Processing " . count($pdfFiles) . " PDF files...\n";

    $start = microtime(true);
    $results = $kreuzberg->batchExtractFiles($pdfFiles, $config);
    $elapsed = microtime(true) - $start;

    echo "Completed in " . number_format($elapsed, 2) . " seconds\n";
    echo "Throughput: " . number_format(count($pdfFiles) / $elapsed, 2) . " files/second\n\n";

    // Aggregate statistics
    $totalChars = 0;
    $totalTables = 0;

    foreach ($results as $result) {
        $totalChars += strlen($result->content);
        $totalTables += count($result->tables);
    }

    echo "Total content: " . number_format($totalChars) . " characters\n";
    echo "Total tables: $totalTables\n";
}

// Batch processing from bytes (useful for uploaded files)
$uploadedFiles = [
    ['data' => file_get_contents('file1.pdf'), 'mime' => 'application/pdf'],
    ['data' => file_get_contents('file2.docx'), 'mime' => 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'],
];

$dataList = array_column($uploadedFiles, 'data');
$mimeTypes = array_column($uploadedFiles, 'mime');

$results = batch_extract_bytes($dataList, $mimeTypes);

echo "\nProcessed " . count($results) . " files from memory\n";

// Process directory recursively
function processDirectory(string $dir, Kreuzberg $kreuzberg): array
{
    $results = [];
    $iterator = new RecursiveIteratorIterator(
        new RecursiveDirectoryIterator($dir)
    );

    $files = [];
    foreach ($iterator as $file) {
        if ($file->isFile()) {
            $ext = strtolower($file->getExtension());
            if (in_array($ext, ['pdf', 'docx', 'xlsx', 'pptx', 'txt'], true)) {
                $files[] = $file->getPathname();
            }
        }
    }

    if (empty($files)) {
        return $results;
    }

    // Process in batches of 10
    $batches = array_chunk($files, 10);

    foreach ($batches as $batchIndex => $batch) {
        echo "Processing batch " . ($batchIndex + 1) . "/" . count($batches) . "...\n";
        $batchResults = $kreuzberg->batchExtractFiles($batch);
        $results = array_merge($results, $batchResults);
    }

    return $results;
}

// Process all documents in a directory
$directory = './documents';
if (is_dir($directory)) {
    echo "\nProcessing directory: $directory\n";
    $results = processDirectory($directory, $kreuzberg);
    echo "Processed " . count($results) . " files\n";
}

// Error handling in batch processing
$mixedFiles = ['valid.pdf', 'nonexistent.pdf', 'another.docx'];

try {
    $results = batch_extract_files($mixedFiles);
} catch (\Kreuzberg\Exceptions\KreuzbergException $e) {
    echo "Batch processing error: " . $e->getMessage() . "\n";
    // Handle partial results or retry individual files
}

// Process with progress tracking
$allFiles = glob('documents/*.{pdf,docx,xlsx}', GLOB_BRACE);
$batchSize = 5;
$batches = array_chunk($allFiles, $batchSize);
$totalProcessed = 0;

echo "\nProcessing " . count($allFiles) . " files in " . count($batches) . " batches...\n";

foreach ($batches as $index => $batch) {
    $progress = (($index + 1) / count($batches)) * 100;
    echo sprintf("\rProgress: %.1f%% [%d/%d batches]",
        $progress, $index + 1, count($batches));

    $results = $kreuzberg->batchExtractFiles($batch);
    $totalProcessed += count($results);
}

echo "\n\nCompleted! Processed $totalProcessed files.\n";
```
