```php
<?php

declare(strict_types=1);

/**
 * Installing Kreuzberg via Composer
 *
 * This snippet shows how to install the Kreuzberg PHP package using Composer.
 * The package provides the object-oriented and procedural APIs, while the
 * native extension (kreuzberg.so/.dll) must be installed separately.
 */

// Step 1: Install via Composer
// Run in terminal:
// composer require kreuzberg/kreuzberg

// Step 2: Verify installation
require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;

// Check if the extension is loaded
if (!extension_loaded('kreuzberg')) {
    echo "Error: kreuzberg extension is not loaded\n";
    echo "Please add 'extension=kreuzberg.so' (or .dll on Windows) to your php.ini\n";
    exit(1);
}

echo "Kreuzberg extension is loaded successfully!\n";
echo "Version: " . Kreuzberg::version() . "\n";

// Test basic functionality
$kreuzberg = new Kreuzberg();
echo "Kreuzberg client initialized successfully!\n";
```
