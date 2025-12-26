```php
<?php

declare(strict_types=1);

/**
 * Basic Document Extraction (OOP API)
 *
 * This example demonstrates the simplest way to extract text from a document
 * using the object-oriented API.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;

// Create a new Kreuzberg instance
$kreuzberg = new Kreuzberg();

// Extract content from a PDF file
$result = $kreuzberg->extractFile('document.pdf');

// Access the extracted content
echo "Extracted Content:\n";
echo "==================\n";
echo $result->content . "\n\n";

// Access metadata
echo "Metadata:\n";
echo "=========\n";
echo "Title: " . ($result->metadata->title ?? 'N/A') . "\n";
echo "Author: " . ($result->metadata->author ?? 'N/A') . "\n";
echo "Pages: " . ($result->metadata->pageCount ?? 'N/A') . "\n";
echo "Format: " . $result->mimeType . "\n\n";

// Access extracted tables
if (count($result->tables) > 0) {
    echo "Tables Found: " . count($result->tables) . "\n";
    foreach ($result->tables as $index => $table) {
        echo "\nTable " . ($index + 1) . " (Page {$table->pageNumber}):\n";
        echo $table->markdown . "\n";
    }
}
```
