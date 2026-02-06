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
 * @property-read array<Keyword>|null $keywords Extracted keywords with scores if KeywordConfig provided
 * @property-read array<Element>|null $elements Semantic elements when output_format='element_based'
 * @property-read array<OcrElement>|null $ocrElements OCR elements with positioning and confidence when OCR element config enabled
 * @property-read DjotContent|null $djotContent Structured Djot content when output_format='djot'
 */
readonly class ExtractionResult
{
    /**
     * @param array<Table> $tables
     * @param array<string>|null $detectedLanguages
     * @param array<Chunk>|null $chunks
     * @param array<ExtractedImage>|null $images
     * @param array<PageContent>|null $pages
     * @param array<Keyword>|null $keywords
     * @param array<Element>|null $elements
     * @param array<OcrElement>|null $ocrElements
     * @param DjotContent|null $djotContent
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
        public ?array $keywords = null,
        public ?array $elements = null,
        public ?array $ocrElements = null,
        public ?DjotContent $djotContent = null,
    ) {
    }

    /**
     * Create ExtractionResult from array returned by extension.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var string $content */
        $content = $data['content'] ?? '';

        /** @var string $mimeType */
        $mimeType = $data['mime_type'] ?? 'application/octet-stream';

        /** @var array<string, mixed> $metadataData */
        $metadataData = $data['metadata'] ?? [];

        /** @var array<array<string, mixed>> $tablesData */
        $tablesData = $data['tables'] ?? [];

        /** @var array<string>|null $detectedLanguages */
        $detectedLanguages = $data['detected_languages'] ?? null;

        $chunks = null;
        if (isset($data['chunks'])) {
            /** @var array<array<string, mixed>> $chunksData */
            $chunksData = $data['chunks'];
            $chunks = array_map(
                /** @param array<string, mixed> $chunk */
                static fn (array $chunk): Chunk => Chunk::fromArray($chunk),
                $chunksData,
            );
        }

        $images = null;
        if (isset($data['images'])) {
            /** @var array<array<string, mixed>> $imagesData */
            $imagesData = $data['images'];
            $images = array_map(
                /** @param array<string, mixed> $image */
                static fn (array $image): ExtractedImage => ExtractedImage::fromArray($image),
                $imagesData,
            );
        }

        $pages = null;
        if (isset($data['pages'])) {
            /** @var array<array<string, mixed>> $pagesData */
            $pagesData = $data['pages'];
            $pages = array_map(
                /** @param array<string, mixed> $page */
                static fn (array $page): PageContent => PageContent::fromArray($page),
                $pagesData,
            );
        }

        $keywords = null;
        if (isset($data['keywords'])) {
            /** @var array<array<string, mixed>> $keywordsData */
            $keywordsData = $data['keywords'];
            if (is_array($keywordsData)) {
                $keywords = array_map(
                    /** @param array<string, mixed> $keyword */
                    static fn (array $keyword): Keyword => Keyword::fromArray($keyword),
                    $keywordsData,
                );
            }
        }

        $elements = null;
        if (isset($data['elements'])) {
            /** @var array<array<string, mixed>> $elementsData */
            $elementsData = $data['elements'];
            $elements = array_map(
                /** @param array<string, mixed> $element */
                static fn (array $element): Element => Element::fromArray($element),
                $elementsData,
            );
        }

        $ocrElements = null;
        if (isset($data['ocr_elements'])) {
            /** @var array<array<string, mixed>> $ocrElementsData */
            $ocrElementsData = $data['ocr_elements'];
            $ocrElements = array_map(
                /** @param array<string, mixed> $element */
                static fn (array $element): OcrElement => OcrElement::fromArray($element),
                $ocrElementsData,
            );
        }

        $djotContent = null;
        if (isset($data['djot_content'])) {
            /** @var array<string, mixed> $djotContentData */
            $djotContentData = $data['djot_content'];
            $djotContent = DjotContent::fromArray($djotContentData);
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
            keywords: $keywords,
            elements: $elements,
            ocrElements: $ocrElements,
            djotContent: $djotContent,
        );
    }
}
