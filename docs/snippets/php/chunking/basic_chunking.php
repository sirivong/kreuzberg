```php
<?php

declare(strict_types=1);

/**
 * Basic Text Chunking
 *
 * Split documents into smaller chunks for RAG (Retrieval Augmented Generation),
 * vector databases, and context-aware processing.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\ChunkingConfig;

// Basic chunking with default settings
$config = new ExtractionConfig(
    chunking: new ChunkingConfig(
        maxChunkSize: 512,
        chunkOverlap: 50
    )
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('long_document.pdf');

echo "Document Chunking Results:\n";
echo str_repeat('=', 60) . "\n";
echo "Total chunks: " . count($result->chunks ?? []) . "\n";
echo "Total content length: " . strlen($result->content) . "\n\n";

// Display each chunk
foreach ($result->chunks ?? [] as $chunk) {
    echo "Chunk {$chunk->metadata->chunkIndex}:\n";
    echo str_repeat('-', 60) . "\n";
    echo "Length: " . strlen($chunk->content) . " chars\n";
    echo "Content: " . substr($chunk->content, 0, 100) . "...\n\n";
}

// Custom chunk sizes for different use cases
$sizes = [
    'Small (256)' => 256,   // For tight context windows
    'Medium (512)' => 512,  // Balanced
    'Large (1024)' => 1024, // For more context
    'XLarge (2048)' => 2048, // Maximum context
];

foreach ($sizes as $name => $size) {
    $config = new ExtractionConfig(
        chunking: new ChunkingConfig(
            maxChunkSize: $size,
            chunkOverlap: (int)($size * 0.1)  // 10% overlap
        )
    );

    $kreuzberg = new Kreuzberg($config);
    $result = $kreuzberg->extractFile('document.pdf');

    echo "$name chunks:\n";
    echo "  Total: " . count($result->chunks ?? []) . "\n";
    echo "  Avg size: " . number_format(
        array_sum(array_map(
            fn($c) => strlen($c->content),
            $result->chunks ?? []
        )) / count($result->chunks ?? [1])
    ) . " chars\n\n";
}

// Chunking with sentence respect
$sentenceConfig = new ExtractionConfig(
    chunking: new ChunkingConfig(
        maxChunkSize: 512,
        chunkOverlap: 50,
        respectSentences: true,  // Don't split mid-sentence
        respectParagraphs: false
    )
);

$kreuzberg = new Kreuzberg($sentenceConfig);
$result = $kreuzberg->extractFile('article.pdf');

echo "Sentence-respecting chunks:\n";
echo str_repeat('=', 60) . "\n";

foreach ($result->chunks ?? [] as $chunk) {
    // Count sentences in chunk
    $sentences = preg_match_all('/[.!?]+/', $chunk->content);
    echo "Chunk {$chunk->metadata->chunkIndex}: $sentences sentences\n";
    echo "  Starts with: " . substr($chunk->content, 0, 50) . "...\n";
    echo "  Ends with: ..." . substr($chunk->content, -50) . "\n\n";
}

// Chunking with paragraph respect
$paragraphConfig = new ExtractionConfig(
    chunking: new ChunkingConfig(
        maxChunkSize: 1000,
        chunkOverlap: 100,
        respectSentences: true,
        respectParagraphs: true  // Keep paragraphs together when possible
    )
);

$kreuzberg = new Kreuzberg($paragraphConfig);
$result = $kreuzberg->extractFile('essay.pdf');

echo "Paragraph-respecting chunks:\n";
echo str_repeat('=', 60) . "\n";

foreach ($result->chunks ?? [] as $chunk) {
    $paragraphs = substr_count($chunk->content, "\n\n");
    echo "Chunk {$chunk->metadata->chunkIndex}: ~$paragraphs paragraphs\n";
    echo "  " . strlen($chunk->content) . " characters\n\n";
}

// Process chunks for vector database insertion
$config = new ExtractionConfig(
    chunking: new ChunkingConfig(
        maxChunkSize: 512,
        chunkOverlap: 50,
        respectSentences: true
    )
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('knowledge_base.pdf');

// Prepare chunks for database
$chunksForDb = [];
foreach ($result->chunks ?? [] as $chunk) {
    $chunksForDb[] = [
        'id' => uniqid('chunk_', true),
        'document_id' => 'doc_' . md5($result->content),
        'chunk_index' => $chunk->metadata->chunkIndex,
        'content' => $chunk->content,
        'length' => strlen($chunk->content),
        'metadata' => [
            'source_file' => 'knowledge_base.pdf',
            'mime_type' => $result->mimeType,
            'created_at' => date('Y-m-d H:i:s'),
        ],
    ];
}

echo "Prepared " . count($chunksForDb) . " chunks for database:\n";
foreach (array_slice($chunksForDb, 0, 3) as $chunk) {
    echo "  ID: {$chunk['id']}\n";
    echo "  Index: {$chunk['chunk_index']}\n";
    echo "  Length: {$chunk['length']} chars\n\n";
}

// Save chunks to JSON file
file_put_contents(
    'chunks.json',
    json_encode($chunksForDb, JSON_PRETTY_PRINT)
);
echo "Saved chunks to: chunks.json\n";
```
