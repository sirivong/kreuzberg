```php
<?php

declare(strict_types=1);

/**
 * KeywordConfig - Keyword Extraction
 *
 * Automatically extract keywords and key phrases from documents.
 * Useful for document categorization, search indexing, and summarization.
 */

require_once __DIR__ . '/vendor/autoload.php';

use Kreuzberg\Kreuzberg;
use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\KeywordConfig;

// Basic keyword extraction using RAKE algorithm
$config = new ExtractionConfig(
    keyword: new KeywordConfig(
        enabled: true,
        algorithm: 'rake',
        maxKeywords: 10
    )
);

$kreuzberg = new Kreuzberg($config);
$result = $kreuzberg->extractFile('article.pdf');

echo "Top Keywords:\n";
echo str_repeat('=', 40) . "\n";
foreach ($result->metadata->keywords ?? [] as $keyword) {
    echo "  • $keyword\n";
}
echo "\n";

// Extract more keywords for detailed analysis
$detailedConfig = new ExtractionConfig(
    keyword: new KeywordConfig(
        enabled: true,
        algorithm: 'rake',
        maxKeywords: 25
    )
);

$kreuzberg = new Kreuzberg($detailedConfig);
$result = $kreuzberg->extractFile('research_paper.pdf');

echo "Detailed keyword analysis:\n";
echo "Total keywords: " . count($result->metadata->keywords ?? []) . "\n";

if (!empty($result->metadata->keywords)) {
    // Group keywords by first letter for organization
    $grouped = [];
    foreach ($result->metadata->keywords as $keyword) {
        $first = strtoupper($keyword[0]);
        if (!isset($grouped[$first])) {
            $grouped[$first] = [];
        }
        $grouped[$first][] = $keyword;
    }

    foreach ($grouped as $letter => $keywords) {
        echo "\n$letter:\n";
        foreach ($keywords as $keyword) {
            echo "  - $keyword\n";
        }
    }
}

// Process multiple documents and find common keywords
$files = ['doc1.pdf', 'doc2.pdf', 'doc3.pdf'];
$allKeywords = [];

foreach ($files as $file) {
    if (!file_exists($file)) continue;

    $result = $kreuzberg->extractFile($file);
    foreach ($result->metadata->keywords ?? [] as $keyword) {
        if (!isset($allKeywords[$keyword])) {
            $allKeywords[$keyword] = 0;
        }
        $allKeywords[$keyword]++;
    }
}

// Find most common keywords across documents
arsort($allKeywords);
echo "\n\nMost common keywords across documents:\n";
$count = 0;
foreach ($allKeywords as $keyword => $frequency) {
    if ($count++ >= 10) break;
    echo sprintf("  %2d. %-30s (appears in %d documents)\n",
        $count, $keyword, $frequency);
}

// Use keywords for document categorization
$categoryKeywords = [
    'technology' => ['software', 'computer', 'algorithm', 'data', 'system'],
    'business' => ['market', 'revenue', 'sales', 'customer', 'profit'],
    'science' => ['research', 'experiment', 'hypothesis', 'analysis', 'study'],
];

$docKeywords = $result->metadata->keywords ?? [];
$scores = [];

foreach ($categoryKeywords as $category => $terms) {
    $score = 0;
    foreach ($terms as $term) {
        if (in_array($term, $docKeywords, true)) {
            $score++;
        }
    }
    $scores[$category] = $score;
}

arsort($scores);
$topCategory = array_key_first($scores);
echo "\nDocument category: $topCategory (score: {$scores[$topCategory]})\n";
```
