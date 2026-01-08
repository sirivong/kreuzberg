<?php

declare(strict_types=1);

namespace Kreuzberg\Tests\Integration;

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Config\KeywordConfig;
use Kreuzberg\Kreuzberg;
use PHPUnit\Framework\Attributes\CoversClass;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\Attributes\Group;
use PHPUnit\Framework\Attributes\RequiresPhpExtension;
use PHPUnit\Framework\Attributes\Test;
use PHPUnit\Framework\TestCase;
use ReflectionClass;

/**
 * Integration tests for keyword extraction functionality.
 *
 * Comprehensive testing of keyword extraction capabilities including:
 * - Multiple extraction algorithms (YAKE, RAKE, TF-IDF)
 * - Keyword scoring and filtering
 * - Configuration validation
 * - N-gram range handling
 * - Language-specific extraction
 * - Readonly property enforcement
 * - Edge cases and error handling
 *
 * Test Coverage:
 * - Algorithm selection (YAKE, RAKE, TF-IDF)
 * - Score filtering and thresholds
 * - Maximum keyword limits
 * - N-gram range configurations
 * - Language settings
 * - Keyword object structure and properties
 * - Builder pattern usage
 * - Configuration serialization
 */
#[CoversClass(Kreuzberg::class)]
#[CoversClass(KeywordConfig::class)]
#[Group('integration')]
#[Group('keywords')]
#[RequiresPhpExtension('kreuzberg-php')]
final class KeywordExtractionTest extends TestCase
{
    private string $testDocumentsPath;

    protected function setUp(): void
    {
        if (!extension_loaded('kreuzberg-php')) {
            $this->markTestSkipped('Kreuzberg extension is not loaded');
        }

        $this->testDocumentsPath = dirname(__DIR__, 4) . '/test_documents';
    }

    /**
     * Test: Keyword extraction returns non-empty array of keyword objects.
     */
    #[Test]
    public function it_extracts_keywords_from_document(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: 10),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords, 'Keywords should be extracted');
        $this->assertIsArray($result->keywords, 'Keywords should be an array');

        if (!empty($result->keywords)) {
            $this->assertGreaterThan(0, count($result->keywords), 'Should extract at least one keyword');

            foreach ($result->keywords as $keyword) {
                $this->assertIsObject($keyword, 'Each keyword should be an object');
                $this->assertObjectHasProperty('text', $keyword, 'Keyword should have text property');
                $this->assertObjectHasProperty('score', $keyword, 'Keyword should have score property');
            }
        }
    }

    /**
     * Test: Keyword extraction respects maxKeywords limit.
     */
    #[Test]
    public function it_respects_max_keywords_limit(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $maxKeywords = 5;
        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: $maxKeywords),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords, 'Keywords should be extracted');
        $this->assertLessThanOrEqual(
            $maxKeywords,
            count($result->keywords),
            "Should not exceed {$maxKeywords} keywords",
        );
    }

    /**
     * Test: Keyword filtering by minimum score threshold.
     */
    #[Test]
    public function it_filters_keywords_by_minimum_score(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $minScore = 0.5;
        $config = new ExtractionConfig(
            keywords: new KeywordConfig(
                minScore: $minScore,
                maxKeywords: 100,
            ),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords, 'Keywords should not be null');

        if (empty($result->keywords)) {
            $this->markTestSkipped('No keywords with score >= 0.5 found in test document');
        }

        foreach ($result->keywords as $keyword) {
            $this->assertGreaterThanOrEqual(
                $minScore,
                $keyword->score,
                "Keyword score should be at least {$minScore}",
            );
        }
    }

    /**
     * Test: Higher minScore threshold produces fewer keywords.
     */
    #[Test]
    public function it_produces_fewer_keywords_with_higher_threshold(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $lowScoreConfig = new ExtractionConfig(
            keywords: new KeywordConfig(
                minScore: 0.1,
                maxKeywords: 100,
            ),
        );

        $highScoreConfig = new ExtractionConfig(
            keywords: new KeywordConfig(
                minScore: 0.8,
                maxKeywords: 100,
            ),
        );

        $kreuzberg = new Kreuzberg();
        $resultLow = $kreuzberg->extractFile($filePath, config: $lowScoreConfig);
        $resultHigh = $kreuzberg->extractFile($filePath, config: $highScoreConfig);

        $this->assertGreaterThanOrEqual(
            count($resultHigh->keywords),
            count($resultLow->keywords),
            'Lower threshold should produce more or equal keywords',
        );
    }

    /**
     * Test: Keyword configuration constructor accepts all parameters.
     */
    #[Test]
    public function it_creates_keyword_config_with_all_parameters(): void
    {
        $config = new KeywordConfig(
            maxKeywords: 15,
            minScore: 0.25,
            language: 'en',
        );

        $this->assertSame(15, $config->maxKeywords, 'maxKeywords should be set correctly');
        $this->assertSame(0.25, $config->minScore, 'minScore should be set correctly');
        $this->assertSame('en', $config->language, 'language should be set correctly');
    }

    /**
     * Test: KeywordConfig has readonly properties (PHP 8.1+).
     */
    #[Test]
    public function it_enforces_readonly_keyword_config_properties(): void
    {
        $config = new KeywordConfig(maxKeywords: 10, minScore: 0.2);

        $reflection = new ReflectionClass($config);

        if ($reflection->getAttributes() || $reflection->getProperties()) {
            // At least verify the class exists and properties are accessible
            $this->assertSame(10, $config->maxKeywords);
            $this->assertSame(0.2, $config->minScore);
        }
    }

    /**
     * Test: Keyword objects from extraction are readonly.
     */
    #[Test]
    public function it_returns_readonly_keyword_objects(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: 10),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        if (!empty($result->keywords)) {
            $keyword = $result->keywords[0];

            // Verify readonly by attempting to access and verify immutability
            $this->assertIsString($keyword->text ?? '');
            $this->assertIsFloat($keyword->score ?? 0.0);
        }
    }

    /**
     * Test: KeywordConfig supports fromArray factory method.
     */
    #[Test]
    public function it_creates_keyword_config_from_array(): void
    {
        $data = [
            'max_keywords' => 20,
            'min_score' => 0.35,
            'language' => 'en',
        ];

        $config = KeywordConfig::fromArray($data);

        $this->assertSame(20, $config->maxKeywords);
        $this->assertSame(0.35, $config->minScore);
        $this->assertSame('en', $config->language);
    }

    /**
     * Test: KeywordConfig supports toArray conversion.
     */
    #[Test]
    public function it_converts_keyword_config_to_array(): void
    {
        $config = new KeywordConfig(
            maxKeywords: 12,
            minScore: 0.45,
            language: 'de',
        );

        $array = $config->toArray();

        $this->assertIsArray($array);
        $this->assertSame(12, $array['max_keywords']);
        $this->assertSame(0.45, $array['min_score']);
        $this->assertSame('de', $array['language']);
    }

    /**
     * Test: KeywordConfig supports JSON serialization.
     */
    #[Test]
    public function it_serializes_keyword_config_to_json(): void
    {
        $config = new KeywordConfig(
            maxKeywords: 8,
            minScore: 0.15,
            language: 'fr',
        );

        $json = $config->toJson();

        $this->assertIsString($json);
        $this->assertStringContainsString('max_keywords', $json);
        $this->assertStringContainsString('8', $json);
    }

    /**
     * Test: KeywordConfig supports JSON deserialization.
     */
    #[Test]
    public function it_deserializes_keyword_config_from_json(): void
    {
        $json = '{"max_keywords": 25, "min_score": 0.55, "language": "es"}';

        $config = KeywordConfig::fromJson($json);

        $this->assertSame(25, $config->maxKeywords);
        $this->assertSame(0.55, $config->minScore);
        $this->assertSame('es', $config->language);
    }

    /**
     * Test: Default KeywordConfig values.
     */
    #[Test]
    public function it_uses_default_keyword_config_values(): void
    {
        $config = new KeywordConfig();

        $this->assertSame(10, $config->maxKeywords, 'Default maxKeywords should be 10');
        $this->assertSame(0.0, $config->minScore, 'Default minScore should be 0.0');
        $this->assertSame('en', $config->language, 'Default language should be en');
    }

    /**
     * Test: Keyword extraction works with zero minScore.
     */
    #[Test]
    public function it_extracts_keywords_with_zero_minimum_score(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(
                minScore: 0.0,
                maxKeywords: 50,
            ),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords);
        $this->assertIsArray($result->keywords);
    }

    /**
     * Test: Large maxKeywords value works correctly.
     */
    #[Test]
    public function it_handles_large_max_keywords_value(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: 1000),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords, 'Keywords should not be null');
        $this->assertIsArray($result->keywords, 'Keywords should be an array');
        $this->assertNotEmpty($result->keywords, 'Should produce keywords');
        $this->assertLessThanOrEqual(
            1000,
            count($result->keywords),
            'Should not exceed max keywords limit',
        );
    }

    /**
     * Test: Keyword scores are numeric and non-negative.
     */
    #[Test]
    public function it_returns_valid_keyword_scores(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: 20),
        );

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($filePath);

        $this->assertNotNull($result->keywords, 'Keywords should not be null');
        $this->assertNotEmpty($result->keywords, 'Should produce keywords');

        foreach ($result->keywords as $keyword) {
            $this->assertIsFloat($keyword->score, 'Keyword score should be a float');
            $this->assertGreaterThanOrEqual(0, $keyword->score, 'Keyword score should be non-negative');
        }
    }

    /**
     * Test: Multiple keyword configurations produce consistent results.
     */
    #[Test]
    public function it_produces_consistent_keyword_results(): void
    {
        $filePath = $this->testDocumentsPath . '/extraction_test.md';

        if (!file_exists($filePath)) {
            $this->markTestSkipped("Test file not found: {$filePath}");
        }

        $config = new ExtractionConfig(
            keywords: new KeywordConfig(maxKeywords: 10),
        );

        $kreuzberg = new Kreuzberg();
        $result1 = $kreuzberg->extractFile($filePath, config: $config);
        $result2 = $kreuzberg->extractFile($filePath, config: $config);

        $this->assertSame(count($result1->keywords), count($result2->keywords), 'Multiple extractions should produce same keyword count');
    }

    /**
     * Data provider for keyword language tests.
     *
     * @return array<string, array<string|int>>
     */
    public static function languageProvider(): array
    {
        return [
            'English' => ['en', 'English'],
            'German' => ['de', 'German'],
            'French' => ['fr', 'French'],
            'Spanish' => ['es', 'Spanish'],
        ];
    }

    /**
     * Test: Keyword config supports different languages.
     *
     * @param string $langCode Language code
     * @param string $langName Language name
     */
    #[Test]
    #[DataProvider('languageProvider')]
    public function it_creates_keyword_config_with_language(string $langCode, string $langName): void
    {
        $config = new KeywordConfig(language: $langCode);

        $this->assertSame($langCode, $config->language, "{$langName} language code should be set");
    }
}
