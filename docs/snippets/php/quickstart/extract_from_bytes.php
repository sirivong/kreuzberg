```php
<?php

declare(strict_types=1);

/**
 * Extracting from Bytes
 *
 * Extract content from file data in memory instead of from disk.
 * Useful for processing uploaded files or data from remote sources.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use function Kreuzberg\extract_bytes;

// Read file into memory
$fileData = file_get_contents('document.pdf');
$mimeType = 'application/pdf';

// Method 1: Using procedural API
$result = extract_bytes($fileData, $mimeType);
echo "Extracted using procedural API:\n";
echo substr($result->content, 0, 200) . "...\n\n";

// Method 2: Using OOP API
$kreuzberg = new Kreuzberg();
$result = $kreuzberg->extractBytes($fileData, $mimeType);
echo "Extracted using OOP API:\n";
echo substr($result->content, 0, 200) . "...\n\n";

// Example: Processing an uploaded file
// Simulating $_FILES array
$uploadedFile = [
    'tmp_name' => '/tmp/uploaded_document.pdf',
    'type' => 'application/pdf',
    'size' => 1024000,
];

if (file_exists($uploadedFile['tmp_name'])) {
    $data = file_get_contents($uploadedFile['tmp_name']);
    $result = extract_bytes($data, $uploadedFile['type']);

    echo "Uploaded file processed:\n";
    echo "Size: " . strlen($data) . " bytes\n";
    echo "Content length: " . strlen($result->content) . " characters\n";
}
```
