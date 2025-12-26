<?php

declare(strict_types=1);

namespace Kreuzberg\Types;

/**
 * Image artifact extracted from a document page.
 *
 * @property-read string $data Image data (bytes)
 * @property-read string $format Image format (e.g., "png", "jpeg")
 * @property-read int $imageIndex Image index within document
 * @property-read int|null $pageNumber Page number where image was found
 * @property-read int|null $width Image width in pixels
 * @property-read int|null $height Image height in pixels
 * @property-read string|null $colorspace Image colorspace
 * @property-read int|null $bitsPerComponent Bits per color component
 * @property-read bool $isMask Whether image is a mask
 * @property-read string|null $description Image description/alt text
 * @property-read ExtractionResult|null $ocrResult OCR result if OCR was performed on this image
 */
readonly class ExtractedImage
{
    public function __construct(
        public string $data,
        public string $format,
        public int $imageIndex,
        public ?int $pageNumber = null,
        public ?int $width = null,
        public ?int $height = null,
        public ?string $colorspace = null,
        public ?int $bitsPerComponent = null,
        public bool $isMask = false,
        public ?string $description = null,
        public ?ExtractionResult $ocrResult = null,
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $imageData = $data['data'] ?? '';
        assert(is_string($imageData));

        $format = $data['format'] ?? '';
        assert(is_string($format));

        $imageIndex = $data['image_index'] ?? 0;
        assert(is_int($imageIndex));

        $pageNumber = $data['page_number'] ?? null;
        assert($pageNumber === null || is_int($pageNumber));

        $width = $data['width'] ?? null;
        assert($width === null || is_int($width));

        $height = $data['height'] ?? null;
        assert($height === null || is_int($height));

        $colorspace = $data['colorspace'] ?? null;
        assert($colorspace === null || is_string($colorspace));

        $bitsPerComponent = $data['bits_per_component'] ?? null;
        assert($bitsPerComponent === null || is_int($bitsPerComponent));

        $isMask = $data['is_mask'] ?? false;
        assert(is_bool($isMask));

        $description = $data['description'] ?? null;
        assert($description === null || is_string($description));

        $ocrResult = null;
        if (isset($data['ocr_result'])) {
            $ocrResultData = $data['ocr_result'];
            assert(is_array($ocrResultData));
            $ocrResult = ExtractionResult::fromArray($ocrResultData);
        }

        return new self(
            data: $imageData,
            format: $format,
            imageIndex: $imageIndex,
            pageNumber: $pageNumber,
            width: $width,
            height: $height,
            colorspace: $colorspace,
            bitsPerComponent: $bitsPerComponent,
            isMask: $isMask,
            description: $description,
            ocrResult: $ocrResult,
        );
    }
}
