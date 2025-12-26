<?php

declare(strict_types=1);

namespace Kreuzberg\Types;

/**
 * Chunk metadata describing offsets within the original document.
 *
 * @property-read int $byteStart Starting byte offset
 * @property-read int $byteEnd Ending byte offset
 * @property-read int|null $tokenCount Number of tokens in chunk
 * @property-read int $chunkIndex Chunk index (0-based)
 * @property-read int $totalChunks Total number of chunks
 * @property-read int|null $firstPage First page number in chunk
 * @property-read int|null $lastPage Last page number in chunk
 */
readonly class ChunkMetadata
{
    public function __construct(
        public int $byteStart,
        public int $byteEnd,
        public ?int $tokenCount,
        public int $chunkIndex,
        public int $totalChunks,
        public ?int $firstPage = null,
        public ?int $lastPage = null,
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        $byteStart = $data['byte_start'] ?? 0;
        /** @var int $byteStart */
        assert(is_int($byteStart));

        $byteEnd = $data['byte_end'] ?? 0;
        /** @var int $byteEnd */
        assert(is_int($byteEnd));

        $tokenCount = $data['token_count'] ?? null;
        /** @var int|null $tokenCount */
        assert($tokenCount === null || is_int($tokenCount));

        $chunkIndex = $data['chunk_index'] ?? 0;
        /** @var int $chunkIndex */
        assert(is_int($chunkIndex));

        $totalChunks = $data['total_chunks'] ?? 0;
        /** @var int $totalChunks */
        assert(is_int($totalChunks));

        $firstPage = $data['first_page'] ?? null;
        /** @var int|null $firstPage */
        assert($firstPage === null || is_int($firstPage));

        $lastPage = $data['last_page'] ?? null;
        /** @var int|null $lastPage */
        assert($lastPage === null || is_int($lastPage));

        return new self(
            byteStart: $byteStart,
            byteEnd: $byteEnd,
            tokenCount: $tokenCount,
            chunkIndex: $chunkIndex,
            totalChunks: $totalChunks,
            firstPage: $firstPage,
            lastPage: $lastPage,
        );
    }
}
