<?php

declare(strict_types=1);

namespace Kreuzberg\Types;

/**
 * Content for a single page/slide.
 *
 * When page extraction is enabled, documents are split into per-page content
 * with associated tables and images mapped to each page.
 *
 * @property-read int $pageNumber Page number (1-based)
 * @property-read string $content Page text content
 * @property-read array<Table> $tables Tables found on this page
 * @property-read array<ExtractedImage> $images Images found on this page
 */
readonly class PageContent
{
    /**
     * @param array<Table> $tables
     * @param array<ExtractedImage> $images
     */
    public function __construct(
        public int $pageNumber,
        public string $content,
        public array $tables = [],
        public array $images = [],
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $pageNumber = $data['page_number'] ?? 0;
        /** @var int $pageNumber */
        assert(is_int($pageNumber));

        $content = $data['content'] ?? '';
        /** @var string $content */
        assert(is_string($content));

        $tablesData = $data['tables'] ?? [];
        /** @var array<array<string, mixed>> $tablesData */
        assert(is_array($tablesData));

        $imagesData = $data['images'] ?? [];
        /** @var array<array<string, mixed>> $imagesData */
        assert(is_array($imagesData));

        return new self(
            pageNumber: $pageNumber,
            content: $content,
            tables: array_map(
                /** @param array<string, mixed> $table */
                static fn (array $table): Table => Table::fromArray($table),
                $tablesData,
            ),
            images: array_map(
                /** @param array<string, mixed> $image */
                static fn (array $image): ExtractedImage => ExtractedImage::fromArray($image),
                $imagesData,
            ),
        );
    }
}
