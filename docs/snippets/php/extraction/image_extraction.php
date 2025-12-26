```php
<?php

declare(strict_types=1);

/**
 * Image Extraction from Documents
 *
 * Extract embedded images from PDFs, Office documents, and other formats.
 * Optionally perform OCR on extracted images.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ImageExtractionConfig;
use Kreuzberg\Config\OcrConfig;

// Basic image extraction
$config = new ExtractionConfig(
    imageExtraction: new ImageExtractionConfig(
        extractImages: true,
        minWidth: 100,      // Skip small images
        minHeight: 100
    ),
    extractImages: true
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('presentation.pptx');

echo "Image Extraction Results:\n";
echo str_repeat('=', 60) . "\n";
echo "Images found: " . count($result->images ?? []) . "\n\n";

// Save all extracted images
foreach ($result->images ?? [] as $image) {
    $filename = sprintf(
        'extracted_p%d_i%d_%dx%d.%s',
        $image->pageNumber,
        $image->imageIndex,
        $image->width,
        $image->height,
        $image->format
    );

    file_put_contents($filename, $image->data);
    echo "Saved: $filename\n";
    echo "  Size: {$image->width}x{$image->height} pixels\n";
    echo "  Format: {$image->format}\n";
    echo "  Data: " . number_format(strlen($image->data)) . " bytes\n\n";
}

// Extract images with OCR
$ocrConfig = new ExtractionConfig(
    imageExtraction: new ImageExtractionConfig(
        extractImages: true,
        performOcr: true,
        minWidth: 200,
        minHeight: 100
    ),
    ocr: new OcrConfig(
        backend: 'tesseract',
        language: 'eng'
    )
);

$kreuzberg = new Kreuzberg($ocrConfig);
$result = $kreuzberg->extractFile('scanned_images.pdf');

echo "Images with OCR:\n";
echo str_repeat('=', 60) . "\n";

foreach ($result->images ?? [] as $image) {
    echo "Image {$image->imageIndex} from page {$image->pageNumber}:\n";

    if ($image->ocrResult !== null) {
        echo "  OCR Text: " . substr($image->ocrResult->content, 0, 100) . "...\n";
        echo "  Confidence: " . ($image->ocrResult->confidence ?? 'N/A') . "\n";
    } else {
        echo "  No OCR result\n";
    }
    echo "\n";
}

// Filter images by size and save only large ones
$largeImageConfig = new ExtractionConfig(
    imageExtraction: new ImageExtractionConfig(
        extractImages: true,
        minWidth: 500,      // Only extract large images
        minHeight: 500
    ),
    extractImages: true
);

$kreuzberg = new Kreuzberg($largeImageConfig);
$result = $kreuzberg->extractFile('photo_album.pdf');

echo "Large images (>500x500):\n";
foreach ($result->images ?? [] as $image) {
    $filename = "large_image_{$image->imageIndex}.{$image->format}";
    file_put_contents($filename, $image->data);
    echo "Saved: $filename ({$image->width}x{$image->height})\n";
}

// Extract and categorize images by type
$result = $kreuzberg->extractFile('document.pdf');

$imageTypes = [];
foreach ($result->images ?? [] as $image) {
    if (!isset($imageTypes[$image->format])) {
        $imageTypes[$image->format] = [];
    }
    $imageTypes[$image->format][] = $image;
}

echo "\nImages by format:\n";
foreach ($imageTypes as $format => $images) {
    echo "  $format: " . count($images) . " images\n";

    // Save each format to its own directory
    $dir = "images_$format";
    if (!is_dir($dir)) {
        mkdir($dir, 0755, true);
    }

    foreach ($images as $index => $image) {
        $filename = "$dir/image_$index.$format";
        file_put_contents($filename, $image->data);
    }
    echo "    Saved to: $dir/\n";
}

// Generate image thumbnails (requires GD extension)
if (extension_loaded('gd')) {
    foreach ($result->images ?? [] as $image) {
        if ($image->format === 'png' || $image->format === 'jpg') {
            // Create image from data
            $gdImage = imagecreatefromstring($image->data);

            if ($gdImage !== false) {
                // Create thumbnail (200px wide)
                $width = imagesx($gdImage);
                $height = imagesy($gdImage);
                $thumbWidth = 200;
                $thumbHeight = (int)(($height / $width) * $thumbWidth);

                $thumb = imagecreatetruecolor($thumbWidth, $thumbHeight);
                imagecopyresampled($thumb, $gdImage, 0, 0, 0, 0,
                    $thumbWidth, $thumbHeight, $width, $height);

                $thumbFile = "thumb_{$image->imageIndex}.{$image->format}";
                if ($image->format === 'png') {
                    imagepng($thumb, $thumbFile);
                } else {
                    imagejpeg($thumb, $thumbFile, 85);
                }

                echo "Created thumbnail: $thumbFile\n";

                imagedestroy($gdImage);
                imagedestroy($thumb);
            }
        }
    }
}
```
