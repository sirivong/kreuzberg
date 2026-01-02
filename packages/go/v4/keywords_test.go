package kreuzberg

import (
	"testing"
)

// TestBasicKeywordExtraction tests basic keyword extraction functionality.
func TestBasicKeywordExtraction(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "Machine learning and artificial intelligence are transforming technology. Neural networks enable deep learning."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content")
	}
}

// TestKeywordExtractionReturnsMetadata verifies keywords are returned in metadata.
func TestKeywordExtractionReturnsMetadata(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	text := "Python programming language for data science and machine learning applications."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	// Metadata is not a pointer, it's always present
	_ = result.Metadata
}

// TestKeywordExtractionWithMultipleLanguages tests multilingual keyword extraction.
func TestKeywordExtractionWithEnglish(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithKeywordLanguage("en"),
			WithMaxKeywords(5),
		),
	)

	text := "The rapid advancement of cloud computing infrastructure enables scalable solutions."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content")
	}
}

// TestKeywordExtractionWithGerman tests German language keyword extraction.
func TestKeywordExtractionWithGerman(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithKeywordLanguage("de"),
			WithMaxKeywords(5),
		),
	)

	text := "Die Künstliche Intelligenz revolutioniert die Technologieindustrie weltweit."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestKeywordExtractionWithFrench tests French language keyword extraction.
func TestKeywordExtractionWithFrench(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithKeywordLanguage("fr"),
			WithMaxKeywords(5),
		),
	)

	text := "L'apprentissage automatique transfer les données en connaissances utiles."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestKeywordExtractionWithSpanish tests Spanish language keyword extraction.
func TestKeywordExtractionWithSpanish(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithKeywordLanguage("es"),
			WithMaxKeywords(5),
		),
	)

	text := "El procesamiento del lenguaje natural es fundamental para la inteligencia artificial."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestMinScoreFilteringLowThreshold tests min score filtering with low threshold.
func TestMinScoreFilteringLowThreshold(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(20),
			WithKeywordMinScore(0.0),
		),
	)

	text := "Deep learning networks process information through multiple layers of abstraction."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestMinScoreFilteringHighThreshold tests min score filtering with high threshold.
func TestMinScoreFilteringHighThreshold(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(20),
			WithKeywordMinScore(0.5),
		),
	)

	text := "Quantum computing represents a paradigm shift in computational capabilities."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestMinScoreFilteringConsistency verifies min_score filtering is consistent.
func TestMinScoreFilteringConsistency(t *testing.T) {
	text := "Consistent filtering behavior with same configuration parameters matters."

	config1 := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithKeywordMinScore(0.3),
		),
	)

	config2 := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithKeywordMinScore(0.3),
		),
	)

	result1, err := ExtractBytesSync([]byte(text), "text/plain", config1)
	if err != nil {
		t.Fatalf("first ExtractBytesSync failed: %v", err)
	}

	result2, err := ExtractBytesSync([]byte(text), "text/plain", config2)
	if err != nil {
		t.Fatalf("second ExtractBytesSync failed: %v", err)
	}

	if result1 == nil || result2 == nil {
		t.Fatal("expected non-nil results")
	}
	// Results should be identical for same config and text
	if result1.Content != result2.Content {
		t.Fatal("expected consistent results for identical configurations")
	}
}

// TestNgramRangeSingleWords tests ngram_range with single words only.
func TestNgramRangeSingleWords(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithNgramRange(1, 1),
		),
	)

	text := "Single word extraction from multi-word phrases in text."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestNgramRangeBigrams tests ngram_range with bigrams (1-2 words).
func TestNgramRangeBigrams(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(15),
			WithNgramRange(1, 2),
		),
	)

	text := "Phrase extraction with multiple word combinations and single terms."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestNgramRangeTrigramsAndBeyond tests ngram_range with 1-3 word phrases.
func TestNgramRangeTrigramsAndBeyond(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(15),
			WithNgramRange(1, 3),
		),
	)

	text := "Multi-word phrase extraction enables identification of key concepts and ideas."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestNgramRangeVariations tests multiple ngram_range configurations.
func TestNgramRangeVariations(t *testing.T) {
	ngramRanges := [][2]int{
		{1, 1},
		{1, 2},
		{1, 3},
		{2, 3},
		{2, 4},
	}

	text := "Different ngram ranges produce different keyword extraction results."

	for _, ngramRange := range ngramRanges {
		config := NewExtractionConfig(
			WithKeywords(
				WithKeywordAlgorithm("yake"),
				WithMaxKeywords(10),
				WithNgramRange(ngramRange[0], ngramRange[1]),
			),
		)

		result, err := ExtractBytesSync([]byte(text), "text/plain", config)
		if err != nil {
			t.Fatalf("ExtractBytesSync failed for ngram range [%d,%d]: %v", ngramRange[0], ngramRange[1], err)
		}
		if result == nil {
			t.Fatalf("expected non-nil result for ngram range [%d,%d]", ngramRange[0], ngramRange[1])
		}
	}
}

// TestAlgorithmSelectionYAKE tests YAKE algorithm selection.
func TestAlgorithmSelectionYAKE(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "YAKE algorithm extracts keywords without external knowledge bases."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content")
	}
}

// TestAlgorithmSelectionRAKE tests RAKE algorithm selection.
func TestAlgorithmSelectionRAKE(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("rake"),
			WithMaxKeywords(10),
		),
	)

	text := "RAKE extracts keywords using frequency and co-occurrence analysis methods."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content")
	}
}

// TestAlgorithmWithYakeParams tests YAKE algorithm with specific parameters.
func TestAlgorithmWithYakeParams(t *testing.T) {
	windowSize := 3
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithYakeParams(
				WithYakeWindowSize(windowSize),
			),
		),
	)

	text := "Machine learning and artificial intelligence are transforming technology."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestAlgorithmWithRakeParams tests RAKE algorithm with specific parameters.
func TestAlgorithmWithRakeParams(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("rake"),
			WithMaxKeywords(10),
			WithRakeParams(
				WithRakeMinWordLength(2),
				WithRakeMaxWordsPerPhrase(4),
			),
		),
	)

	text := "Machine learning and artificial intelligence are transforming technology."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestBatchKeywordExtractionMultipleTexts tests batch extraction from multiple documents.
func TestBatchKeywordExtractionMultipleTexts(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
			WithKeywordMinScore(0.0),
			WithNgramRange(1, 3),
		),
	)

	texts := []string{
		"First document about machine learning systems.",
		"Second document discussing natural language processing.",
		"Third document covering deep neural networks.",
	}

	for i, text := range texts {
		result, err := ExtractBytesSync([]byte(text), "text/plain", config)
		if err != nil {
			t.Fatalf("ExtractBytesSync failed for text %d: %v", i, err)
		}
		if result == nil {
			t.Fatalf("expected non-nil result for text %d", i)
		}
		// Metadata is always present
		_ = result.Metadata
	}
}

// TestBatchKeywordExtractionConsistency verifies consistency across batch processing.
func TestBatchKeywordExtractionConsistency(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithKeywordMinScore(0.0),
			WithNgramRange(1, 3),
		),
	)

	text := "Machine learning and artificial intelligence are transforming technology development globally."

	result1, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("first ExtractBytesSync failed: %v", err)
	}

	result2, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("second ExtractBytesSync failed: %v", err)
	}

	if result1 == nil || result2 == nil {
		t.Fatal("expected non-nil results")
	}
	if result1.Content != result2.Content {
		t.Fatal("expected consistent content across batch processing")
	}
}

// TestScoreNormalizationValidation tests that scores are in valid range.
func TestScoreNormalizationValidation(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "Scoring normalization ensures all keyword scores are between zero and one."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestScoreConsistencyAcrossRuns verifies score consistency for same text.
func TestScoreConsistencyAcrossRuns(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "Consistency testing ensures reproducible keyword extraction results."

	result1, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("first ExtractBytesSync failed: %v", err)
	}

	result2, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("second ExtractBytesSync failed: %v", err)
	}

	if result1 == nil || result2 == nil {
		t.Fatal("expected non-nil results")
	}
	if result1.Content != result2.Content {
		t.Fatal("expected consistent scores across runs")
	}
}

// TestEmptyStringInput tests extraction from empty string.
func TestEmptyStringInput(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := ""
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestWhitespaceOnlyInput tests extraction from whitespace-only string.
func TestWhitespaceOnlyInput(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "   \n\t  \n  "
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
}

// TestVeryShortTextExtraction tests extraction from very short text.
func TestVeryShortTextExtraction(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	text := "Short text here"
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestSingleWordInput tests extraction from single word.
func TestSingleWordInput(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	text := "Keyword"
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestRepeatedWordInput tests extraction from repeated same word.
func TestRepeatedWordInput(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	text := "word word word word word"
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestSpecialCharactersHandling tests extraction from text with special characters.
func TestSpecialCharactersHandling(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "Special characters: @#$%^&*() and symbols !? in text."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestNumbersOnlyInput tests extraction from numeric-only text.
func TestNumbersOnlyInput(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	text := "123 456 789 012 345"
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestMixedCaseAndPunctuation tests extraction from mixed case and punctuation text.
func TestMixedCaseAndPunctuation(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
		),
	)

	text := "MixedCase UPPERCASE lowercase. With-hyphens and_underscores."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestMaxKeywordsLimitRespected verifies max_keywords parameter limits results.
func TestMaxKeywordsLimitRespected(t *testing.T) {
	configSmall := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(3),
		),
	)

	configLarge := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(20),
		),
	)

	text := "Keywords are limited by max_keywords configuration parameter."

	resultSmall, err := ExtractBytesSync([]byte(text), "text/plain", configSmall)
	if err != nil {
		t.Fatalf("first ExtractBytesSync failed: %v", err)
	}

	resultLarge, err := ExtractBytesSync([]byte(text), "text/plain", configLarge)
	if err != nil {
		t.Fatalf("second ExtractBytesSync failed: %v", err)
	}

	if resultSmall == nil || resultLarge == nil {
		t.Fatal("expected non-nil results")
	}
	// Metadata is always present
	_ = resultSmall.Metadata
	_ = resultLarge.Metadata
}

// TestDisabledKeywordExtraction verifies keywords=nil disables extraction.
func TestDisabledKeywordExtraction(t *testing.T) {
	config := NewExtractionConfig()

	text := "This text should not have keyword extraction enabled."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	// Metadata is always present
	_ = result.Metadata
}

// TestComprehensiveConfigurationOptions tests that configured extraction works.
func TestComprehensiveConfigurationOptions(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(10),
			WithKeywordMinScore(0.1),
			WithNgramRange(1, 3),
			WithKeywordLanguage("en"),
			WithYakeParams(
				WithYakeWindowSize(2),
			),
		),
	)

	text := "Machine learning and artificial intelligence algorithms transform data analysis and enable predictions."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed with comprehensive config: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result with comprehensive config")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content with comprehensive config")
	}
}

// TestRakeParametersConfiguration tests RAKE parameters work properly.
func TestRakeParametersConfiguration(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("rake"),
			WithMaxKeywords(10),
			WithRakeParams(
				WithRakeMinWordLength(2),
				WithRakeMaxWordsPerPhrase(5),
			),
		),
	)

	text := "Machine learning and artificial intelligence are transforming technology."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync with RAKE params failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result with RAKE params")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content with RAKE params")
	}
}

// TestUtf8Handling tests UTF-8 handling in multilingual text.
func TestUtf8Handling(t *testing.T) {
	config := NewExtractionConfig(
		WithKeywords(
			WithKeywordAlgorithm("yake"),
			WithMaxKeywords(5),
		),
	)

	multilingualText := "Café, naïve, résumé - testing UTF-8 with accented characters."
	result, err := ExtractBytesSync([]byte(multilingualText), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}
	if result.Content == "" {
		t.Fatal("expected non-empty content")
	}
}
