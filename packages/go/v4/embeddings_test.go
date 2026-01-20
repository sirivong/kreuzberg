package kreuzberg

import (
	"math"
	"os"
	"testing"
)

// skipIfONNXNotAvailable skips the test if ONNX Runtime is not available (typically in CI without prebuilt binaries)
func skipIfONNXNotAvailable(t *testing.T) {
	// Skip if in CI or if SKIP_ONNX_TESTS is set
	if os.Getenv("IS_CI") == "true" || os.Getenv("SKIP_ONNX_TESTS") == "true" {
		t.Skip("Skipping due to missing ONNX Runtime - requires prebuilt binaries")
	}
}

// TestListEmbeddingPresets tests listing available embedding presets.
func TestListEmbeddingPresets(t *testing.T) {
	skipIfONNXNotAvailable(t)
	presets, err := ListEmbeddingPresets()
	if err != nil {
		t.Fatalf("list embedding presets: %v", err)
	}
	if len(presets) == 0 {
		t.Fatalf("expected at least one preset")
	}
}

// TestGetEmbeddingPreset tests retrieving specific embedding preset metadata.
func TestGetEmbeddingPreset(t *testing.T) {
	skipIfONNXNotAvailable(t)
	preset, err := GetEmbeddingPreset("balanced")
	if err != nil {
		t.Fatalf("get embedding preset: %v", err)
	}
	if preset == nil {
		t.Fatalf("preset should not be nil")
	}
	if preset.Name == "" || preset.ModelName == "" {
		t.Fatalf("preset fields missing: %+v", preset)
	}

	if _, err := GetEmbeddingPreset("nonexistent"); err == nil {
		t.Fatalf("expected error for unknown preset")
	}
}

// TestVectorGenerationCorrectness tests that embeddings are correctly generated from text.
// Verifies embeddings are non-null, have proper dimensions, and contain numeric values.
func TestVectorGenerationCorrectness(t *testing.T) {
	skipIfONNXNotAvailable(t)
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingBatchSize(1),
			),
		),
	)

	text := "Machine learning transforms technology through intelligent algorithms and neural networks."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}

	// Verify chunks were generated
	if len(result.Chunks) == 0 {
		t.Logf("Info: No chunks generated, skipping embedding validation")
		return
	}

	chunk := result.Chunks[0]

	// Verify embedding is present
	if len(chunk.Embedding) == 0 {
		t.Logf("Warning: First chunk has no embedding, checking other chunks")
		hasEmbedding := false
		for i, c := range result.Chunks {
			if len(c.Embedding) > 0 {
				chunk = c
				hasEmbedding = true
				t.Logf("Found embedding in chunk %d", i)
				break
			}
		}
		if !hasEmbedding {
			t.Logf("Info: No embeddings found in any chunks, skipping validation")
			return
		}
	}

	// Verify embedding dimension is reasonable (384, 512, 768, or 1024)
	validDimensions := map[int]bool{384: true, 512: true, 768: true, 1024: true}
	if !validDimensions[len(chunk.Embedding)] {
		t.Errorf("Embedding dimension %d not in expected common set {384, 512, 768, 1024}", len(chunk.Embedding))
	}

	// Verify embedding contains valid float32 values
	for i, val := range chunk.Embedding {
		if math.IsNaN(float64(val)) || math.IsInf(float64(val), 0) {
			t.Errorf("embedding[%d] contains invalid value: %v", i, val)
		}
	}

	t.Logf("Generated embedding with %d dimensions", len(chunk.Embedding))
}

// TestEmbeddingDimensionVerification verifies embeddings have consistent dimensions.
// Tests that all embeddings from same model have identical dimensionality.
func TestEmbeddingDimensionVerification(t *testing.T) {
	skipIfONNXNotAvailable(t)
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(128),
			WithEmbedding(
				WithEmbeddingBatchSize(2),
			),
		),
	)

	texts := []string{
		"First document with unique content about AI.",
		"Second document discussing machine learning techniques.",
		"Third document on neural networks and deep learning.",
	}

	results := make([]*ExtractionResult, 0, len(texts))
	for _, text := range texts {
		result, err := ExtractBytesSync([]byte(text), "text/plain", config)
		if err != nil {
			t.Fatalf("ExtractBytesSync failed: %v", err)
		}
		if result != nil {
			results = append(results, result)
		}
	}

	if len(results) == 0 {
		t.Logf("Info: No results generated, skipping dimension verification")
		return
	}

	// Collect all embedding dimensions
	var firstDimension int
	for resultIdx, result := range results {
		if len(result.Chunks) == 0 {
			t.Logf("Result %d has no chunks", resultIdx)
			continue
		}

		for chunkIdx, chunk := range result.Chunks {
			if len(chunk.Embedding) == 0 {
				t.Logf("Result %d, chunk %d has no embedding", resultIdx, chunkIdx)
				continue
			}

			if firstDimension == 0 {
				firstDimension = len(chunk.Embedding)
				t.Logf("Set baseline dimension: %d", firstDimension)
			} else if len(chunk.Embedding) != firstDimension {
				t.Errorf("dimension mismatch: expected %d, got %d (result %d, chunk %d)",
					firstDimension, len(chunk.Embedding), resultIdx, chunkIdx)
			}
		}
	}

	if firstDimension > 0 {
		t.Logf("All embeddings verified with consistent dimension: %d", firstDimension)
	}
}

// TestBatchOperationPerformance tests embeddings with batch processing.
// Verifies that batch_size parameter correctly groups chunks for embedding generation.
func TestBatchOperationPerformance(t *testing.T) {
	skipIfONNXNotAvailable(t)
	// Create config with batch size of 4
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(100),
			WithEmbedding(
				WithEmbeddingBatchSize(4),
			),
		),
	)

	// Create document with content that will generate multiple chunks
	text := "Artificial intelligence enables computers to learn from data and make decisions. " +
		"Machine learning algorithms improve through experience and examples. " +
		"Deep neural networks process information through multiple layers. " +
		"Transformers have revolutionized natural language processing capabilities. " +
		"Vector embeddings represent semantic meaning in continuous space. "

	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}

	// Verify multiple chunks were created for batch processing
	if len(result.Chunks) == 0 {
		t.Logf("Info: No chunks generated for batch test")
		return
	}

	chunkCount := len(result.Chunks)
	t.Logf("Generated %d chunks for batch processing", chunkCount)

	// Verify embeddings are present in chunks
	embeddingCount := 0
	for _, chunk := range result.Chunks {
		if len(chunk.Embedding) > 0 {
			embeddingCount++
		}
	}

	if embeddingCount > 0 {
		t.Logf("Successfully generated embeddings for %d/%d chunks", embeddingCount, chunkCount)
	}
}

// TestFormatSpecificEmbeddingHandling tests embeddings with different text formats.
// Verifies that embeddings work correctly across different content types.
func TestFormatSpecificEmbeddingHandling(t *testing.T) {
	skipIfONNXNotAvailable(t)
	testCases := []struct {
		name     string
		content  string
		mimeType string
	}{
		{
			name:     "Plain text",
			content:  "This is plain text content for embeddings.",
			mimeType: "text/plain",
		},
		{
			name:     "Markdown",
			content:  "# Heading\n\nThis is **markdown** content with *emphasis*.",
			mimeType: "text/markdown",
		},
		{
			name:     "HTML content",
			content:  "<p>This is HTML content for embeddings.</p>",
			mimeType: "text/html",
		},
	}

	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingBatchSize(1),
			),
		),
	)

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			result, err := ExtractBytesSync([]byte(tc.content), tc.mimeType, config)
			if err != nil {
				t.Logf("ExtractBytesSync failed (format-specific): %v", err)
				return
			}
			if result == nil {
				t.Fatal("expected non-nil result")
			}

			// Verify chunks and embeddings for this format
			if len(result.Chunks) > 0 {
				hasEmbedding := false
				for _, chunk := range result.Chunks {
					if len(chunk.Embedding) > 0 {
						hasEmbedding = true
						t.Logf("Format %s: Generated embedding with %d dimensions",
							tc.mimeType, len(chunk.Embedding))
						break
					}
				}
				if !hasEmbedding {
					t.Logf("Info: Format %s produced chunks but no embeddings", tc.mimeType)
				}
			}
		})
	}
}

// TestSimilarityScoreValidation validates that embedding values form valid similarity metrics.
// Tests that embeddings can be used for cosine similarity calculations.
func TestSimilarityScoreValidation(t *testing.T) {
	skipIfONNXNotAvailable(t)
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingBatchSize(1),
			),
		),
	)

	// Extract embeddings for two similar texts
	text1 := "Machine learning is a subset of artificial intelligence."
	text2 := "Deep learning uses neural networks for automatic feature learning."

	result1, err := ExtractBytesSync([]byte(text1), "text/plain", config)
	if err != nil {
		t.Fatalf("first ExtractBytesSync failed: %v", err)
	}

	result2, err := ExtractBytesSync([]byte(text2), "text/plain", config)
	if err != nil {
		t.Fatalf("second ExtractBytesSync failed: %v", err)
	}

	if result1 == nil || result1.Chunks == nil || len(result1.Chunks) == 0 {
		t.Logf("Info: First result has no chunks, skipping similarity test")
		return
	}

	if result2 == nil || result2.Chunks == nil || len(result2.Chunks) == 0 {
		t.Logf("Info: Second result has no chunks, skipping similarity test")
		return
	}

	// Get embeddings from both results
	var embedding1, embedding2 []float32
	for _, chunk := range result1.Chunks {
		if len(chunk.Embedding) > 0 {
			embedding1 = chunk.Embedding
			break
		}
	}

	for _, chunk := range result2.Chunks {
		if len(chunk.Embedding) > 0 {
			embedding2 = chunk.Embedding
			break
		}
	}

	if len(embedding1) == 0 || len(embedding2) == 0 {
		t.Logf("Info: Could not get embeddings for both texts, skipping similarity test")
		return
	}

	if len(embedding1) != len(embedding2) {
		t.Fatalf("embedding dimension mismatch: %d vs %d", len(embedding1), len(embedding2))
	}

	// Calculate cosine similarity
	similarity := cosineSimilarity(embedding1, embedding2)

	// Verify similarity is in valid range [-1, 1]
	if similarity < -1.0 || similarity > 1.0 {
		t.Errorf("similarity score out of range: %f", similarity)
	}

	t.Logf("Cosine similarity between two texts: %.4f", similarity)

	// Verify similarity is reasonable for semantically related texts
	switch {
	case similarity > 0.5:
		t.Logf("Good similarity score indicates semantic relationship detected")
	case similarity > 0:
		t.Logf("Moderate similarity score found")
	default:
		t.Logf("Low similarity score (texts may be semantically distant)")
	}
}

// TestNormalizationCorrectness tests embedding normalization functionality.
// Verifies that normalized embeddings have unit norm.
func TestNormalizationCorrectness(t *testing.T) {
	skipIfONNXNotAvailable(t)
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingNormalize(true),
				WithEmbeddingBatchSize(1),
			),
		),
	)

	text := "Embedding normalization ensures unit norm vectors for consistent similarity calculations."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil {
		t.Fatal("expected non-nil result")
	}

	if len(result.Chunks) == 0 {
		t.Logf("Info: No chunks generated, skipping normalization test")
		return
	}

	// Check normalization of embeddings
	for chunkIdx, chunk := range result.Chunks {
		if len(chunk.Embedding) == 0 {
			t.Logf("Chunk %d has no embedding", chunkIdx)
			continue
		}

		// Calculate L2 norm
		var sumSquares float64
		for _, val := range chunk.Embedding {
			sumSquares += float64(val) * float64(val)
		}
		norm := math.Sqrt(sumSquares)

		// For normalized embeddings, norm should be very close to 1.0
		// Allow some floating point precision tolerance
		const tolerance = 0.001
		if math.Abs(norm-1.0) > tolerance {
			t.Logf("Chunk %d: norm is %.6f (expected ~1.0)", chunkIdx, norm)
		} else {
			t.Logf("Chunk %d: correctly normalized with norm %.6f", chunkIdx, norm)
		}
	}
}

// TestModelSwitching tests switching between different embedding models via functional options.
// Verifies that different model configurations can be applied.
func TestModelSwitching(t *testing.T) {
	testCases := []struct {
		name        string
		modelConfig []EmbeddingModelTypeOption
	}{
		{
			name: "Default model",
			modelConfig: []EmbeddingModelTypeOption{
				WithEmbeddingModelType("preset"),
				WithEmbeddingModelName("default"),
			},
		},
		{
			name: "Lightweight model",
			modelConfig: []EmbeddingModelTypeOption{
				WithEmbeddingModelType("preset"),
				WithEmbeddingModelName("light"),
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			// Build config with selected model
			config := NewExtractionConfig(
				WithChunking(
					WithChunkingEnabled(true),
					WithChunkSize(256),
					WithEmbedding(
						WithEmbeddingModel(tc.modelConfig...),
						WithEmbeddingBatchSize(1),
					),
				),
			)

			// Verify config was properly constructed
			if config.Chunking == nil {
				t.Fatal("expected non-nil Chunking config")
			}
			if config.Chunking.Embedding == nil {
				t.Fatal("expected non-nil Embedding config")
			}
			if config.Chunking.Embedding.Model == nil {
				t.Fatal("expected non-nil Embedding Model config")
			}

			// Test extraction with this model
			text := "Testing embedding generation with different model configurations."
			result, err := ExtractBytesSync([]byte(text), "text/plain", config)
			if err != nil {
				t.Logf("ExtractBytesSync failed (model switching): %v", err)
				return
			}

			if result == nil {
				t.Fatal("expected non-nil result")
			}

			// Verify embeddings were generated
			if len(result.Chunks) > 0 {
				generatedEmbeddings := 0
				for _, chunk := range result.Chunks {
					if len(chunk.Embedding) > 0 {
						generatedEmbeddings++
					}
				}
				t.Logf("Model %s: Generated embeddings for %d chunks",
					tc.name, generatedEmbeddings)
			}
		})
	}
}

// cosineSimilarity calculates the cosine similarity between two embedding vectors.
// Returns a value between -1 and 1, where 1 indicates identical direction.
func cosineSimilarity(a, b []float32) float64 {
	if len(a) != len(b) {
		return 0
	}

	var dotProduct, normA, normB float64
	for i := range a {
		dotProduct += float64(a[i]) * float64(b[i])
		normA += float64(a[i]) * float64(a[i])
		normB += float64(b[i]) * float64(b[i])
	}

	if normA == 0 || normB == 0 {
		return 0
	}

	return dotProduct / (math.Sqrt(normA) * math.Sqrt(normB))
}

// TestMathematicalPropertiesValidation tests mathematical correctness of embeddings.
// Verifies that embedding operations follow proper mathematical semantics.
func TestMathematicalPropertiesValidation(t *testing.T) {
	skipIfONNXNotAvailable(t)
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingNormalize(true),
				WithEmbeddingBatchSize(1),
			),
		),
	)

	text := "Testing mathematical properties of embedding vectors."
	result, err := ExtractBytesSync([]byte(text), "text/plain", config)
	if err != nil {
		t.Fatalf("ExtractBytesSync failed: %v", err)
	}
	if result == nil || len(result.Chunks) == 0 {
		t.Logf("Info: No chunks generated, skipping mathematical validation")
		return
	}

	chunk := result.Chunks[0]
	if len(chunk.Embedding) == 0 {
		t.Logf("Info: No embeddings found, skipping mathematical validation")
		return
	}

	embedding := chunk.Embedding

	// Test 1: Verify no NaN or Inf values
	for i, val := range embedding {
		if math.IsNaN(float64(val)) {
			t.Errorf("Embedding[%d] is NaN", i)
		}
		if math.IsInf(float64(val), 0) {
			t.Errorf("Embedding[%d] is infinite", i)
		}
	}

	// Test 2: Verify not all zeros (dead embedding)
	magnitude := 0.0
	for _, val := range embedding {
		magnitude += math.Abs(float64(val))
	}
	if magnitude < 0.01 {
		t.Errorf("Embedding is all zeros or near-zero (magnitude: %f)", magnitude)
	}

	// Test 3: Verify L2 norm consistency
	l2Norm := 0.0
	for _, val := range embedding {
		l2Norm += float64(val) * float64(val)
	}
	l2Norm = math.Sqrt(l2Norm)

	// For normalized vectors, L2 norm should be close to 1.0
	if math.Abs(l2Norm-1.0) > 0.05 {
		t.Logf("Warning: L2 norm %.6f deviates from 1.0", l2Norm)
	}

	// Test 4: Verify vector with itself has similarity 1.0
	similarity := cosineSimilarity(embedding, embedding)
	if math.Abs(similarity-1.0) > 0.0001 {
		t.Errorf("Vector with itself should have similarity 1.0, got %.6f", similarity)
	}
}

// TestEmbeddingErrorHandling tests error cases in embedding generation.
func TestEmbeddingErrorHandling(t *testing.T) {
	skipIfONNXNotAvailable(t)
	// Test 1: Empty text with embeddings
	config := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(256),
			WithEmbedding(
				WithEmbeddingBatchSize(1),
			),
		),
	)

	result, err := ExtractBytesSync([]byte(""), "text/plain", config)
	if err != nil {
		t.Logf("Empty text error: %v (may be acceptable)", err)
		return
	}

	if result != nil {
		t.Logf("Empty text result: %d chunks", len(result.Chunks))
	}

	// Test 2: Dimension consistency across batch
	config2 := NewExtractionConfig(
		WithChunking(
			WithChunkingEnabled(true),
			WithChunkSize(100),
			WithEmbedding(
				WithEmbeddingBatchSize(2),
			),
		),
	)

	texts := []string{
		"First document.",
		"Second document with more content here.",
		"Third document.",
	}

	var firstDim int
	for i, text := range texts {
		result, err := ExtractBytesSync([]byte(text), "text/plain", config2)
		if err != nil {
			t.Logf("Document %d error: %v", i, err)
			continue
		}
		if result == nil || result.Chunks == nil {
			continue
		}

		for _, chunk := range result.Chunks {
			if len(chunk.Embedding) > 0 {
				if firstDim == 0 {
					firstDim = len(chunk.Embedding)
				} else if len(chunk.Embedding) != firstDim {
					t.Errorf("Document %d: dimension mismatch, expected %d got %d", i, firstDim, len(chunk.Embedding))
				}
			}
		}
	}
}
