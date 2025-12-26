```php
<?php

declare(strict_types=1);

/**
 * DOCX (Word) Document Extraction
 *
 * Extract text, tables, and metadata from Microsoft Word documents.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use function Kreuzberg\extract_file;

// Simple DOCX extraction
$result = extract_file('document.docx');

echo "Word Document Extraction:\n";
echo str_repeat('=', 60) . "\n";
echo "Content:\n";
echo $result->content . "\n\n";

// Extract metadata
echo "Document Metadata:\n";
echo str_repeat('=', 60) . "\n";
echo "Title: " . ($result->metadata->title ?? 'N/A') . "\n";
echo "Author: " . ($result->metadata->author ?? 'N/A') . "\n";
echo "Created: " . ($result->metadata->createdAt?->format('Y-m-d H:i:s') ?? 'N/A') . "\n";
echo "Modified: " . ($result->metadata->modifiedAt?->format('Y-m-d H:i:s') ?? 'N/A') . "\n";
echo "Subject: " . ($result->metadata->subject ?? 'N/A') . "\n";
echo "Keywords: " . implode(', ', $result->metadata->keywords ?? []) . "\n\n";

// Extract with tables
$config = new ExtractionConfig(
    extractTables: true,
    preserveFormatting: true
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('report.docx');

// Process tables
foreach ($result->tables as $index => $table) {
    echo "Table " . ($index + 1) . ":\n";
    echo str_repeat('-', 60) . "\n";

    // Display as formatted table
    foreach ($table->cells as $rowIndex => $row) {
        echo implode(' | ', $row) . "\n";
        if ($rowIndex === 0) {
            echo str_repeat('-', 60) . "\n";
        }
    }
    echo "\n";
}

// Extract and convert to different formats
$conversions = [
    'plain' => null,
    'markdown' => 'markdown',
];

foreach ($conversions as $name => $format) {
    $config = new ExtractionConfig(
        outputFormat: $format,
        preserveFormatting: $format !== null
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile('document.docx');

    $outputFile = "output_$name.txt";
    file_put_contents($outputFile, $result->content);
    echo "Saved $name format to: $outputFile\n";
}

// Batch process multiple DOCX files
use function Kreuzberg\batch_extract_files;

$docxFiles = glob('*.docx');
if (!empty($docxFiles)) {
    echo "\nBatch processing " . count($docxFiles) . " DOCX files...\n";

    $results = batch_extract_files($docxFiles);

    foreach ($results as $index => $result) {
        $filename = basename($docxFiles[$index]);
        echo "\n$filename:\n";
        echo "  Characters: " . strlen($result->content) . "\n";
        echo "  Tables: " . count($result->tables) . "\n";
        echo "  Author: " . ($result->metadata->author ?? 'Unknown') . "\n";
    }
}

// Extract comments and revisions (if available in metadata)
$result = extract_file('reviewed_document.docx');

if (!empty($result->metadata->creator)) {
    echo "\nDocument Contributors:\n";
    echo "Creator: " . $result->metadata->creator . "\n";
}

if (!empty($result->metadata->lastModifiedBy)) {
    echo "Last modified by: " . $result->metadata->lastModifiedBy . "\n";
}

// Extract and analyze document statistics
$result = extract_file('document.docx');
$content = $result->content;

$stats = [
    'characters' => mb_strlen($content),
    'words' => str_word_count($content),
    'lines' => substr_count($content, "\n"),
    'paragraphs' => substr_count($content, "\n\n"),
    'sentences' => preg_match_all('/[.!?]+/', $content),
];

echo "\nDocument Statistics:\n";
echo str_repeat('=', 60) . "\n";
foreach ($stats as $metric => $value) {
    echo ucfirst($metric) . ": " . number_format($value) . "\n";
}
```
