<?php

declare(strict_types=1);

namespace Kreuzberg\Types;

/**
 * Result of document extraction.
 *
 * @property-read string $content Extracted text content
 * @property-read string $mimeType MIME type of the processed document
 * @property-read Metadata $metadata Document metadata
 * @property-read array<Table> $tables Extracted tables
 * @property-read array<string>|null $detectedLanguages Detected language codes (ISO 639-1)
 * @property-read array<Chunk>|null $chunks Text chunks with embeddings and metadata
 * @property-read array<ExtractedImage>|null $images Extracted images (with nested OCR results)
 * @property-read array<PageContent>|null $pages Per-page content when page extraction is enabled
 */
readonly class ExtractionResult
{
    /**
     * @param array<Table> $tables
     * @param array<string>|null $detectedLanguages
     * @param array<Chunk>|null $chunks
     * @param array<ExtractedImage>|null $images
     * @param array<PageContent>|null $pages
     */
    public function __construct(
        public string $content,
        public string $mimeType,
        public Metadata $metadata,
        public array $tables = [],
        public ?array $detectedLanguages = null,
        public ?array $chunks = null,
        public ?array $images = null,
        public ?array $pages = null,
    ) {
    }

    /**
     * Create ExtractionResult from array returned by extension.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $content = $data['content'] ?? '';
        /** @var string $content */
        assert(is_string($content));

        $mimeType = $data['mime_type'] ?? 'application/octet-stream';
        /** @var string $mimeType */
        assert(is_string($mimeType));

        $metadataData = $data['metadata'] ?? [];
        /** @var array<string, mixed> $metadataData */
        assert(is_array($metadataData));

        $tablesData = $data['tables'] ?? [];
        /** @var array<array<string, mixed>> $tablesData */
        assert(is_array($tablesData));

        $detectedLanguages = $data['detected_languages'] ?? null;
        /** @var array<string>|null $detectedLanguages */
        assert($detectedLanguages === null || is_array($detectedLanguages));

        $chunks = null;
        if (isset($data['chunks'])) {
            $chunksData = $data['chunks'];
            /** @var array<array<string, mixed>> $chunksData */
            assert(is_array($chunksData));
            $chunks = array_map(
                /** @param array<string, mixed> $chunk */
                static fn (array $chunk): Chunk => Chunk::fromArray($chunk),
                $chunksData,
            );
        }

        $images = null;
        if (isset($data['images'])) {
            $imagesData = $data['images'];
            /** @var array<array<string, mixed>> $imagesData */
            assert(is_array($imagesData));
            $images = array_map(
                /** @param array<string, mixed> $image */
                static fn (array $image): ExtractedImage => ExtractedImage::fromArray($image),
                $imagesData,
            );
        }

        $pages = null;
        if (isset($data['pages'])) {
            $pagesData = $data['pages'];
            /** @var array<array<string, mixed>> $pagesData */
            assert(is_array($pagesData));
            $pages = array_map(
                /** @param array<string, mixed> $page */
                static fn (array $page): PageContent => PageContent::fromArray($page),
                $pagesData,
            );
        }

        return new self(
            content: $content,
            mimeType: $mimeType,
            metadata: Metadata::fromArray($metadataData),
            tables: array_map(
                /** @param array<string, mixed> $table */
                static fn (array $table): Table => Table::fromArray($table),
                $tablesData,
            ),
            detectedLanguages: $detectedLanguages,
            chunks: $chunks,
            images: $images,
            pages: $pages,
        );
    }
}
