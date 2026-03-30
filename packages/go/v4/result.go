package kreuzberg

import (
	"encoding/json"
	"fmt"
	"unsafe"
)

/*
#include "internal/ffi/kreuzberg.h"
#include <stdlib.h>
*/
import "C"

// GetPageCount returns the total number of pages/slides/sheets in the document.
// Returns -1 if there is an error (check the error return value).
// This method provides efficient access to page count metadata without JSON parsing.
func (r *ExtractionResult) GetPageCount() (int, error) {
	if r.Metadata.Pages != nil {
		return int(r.Metadata.Pages.TotalCount), nil
	}
	return 0, nil
}

// GetChunkCount returns the number of text chunks in the extraction result.
// Returns 0 if chunking was not enabled or there are no chunks.
// Returns -1 if there is an error.
// This method provides efficient access to chunk count without JSON parsing.
func (r *ExtractionResult) GetChunkCount() (int, error) {
	if r.Chunks != nil {
		return len(r.Chunks), nil
	}
	return 0, nil
}

// GetDetectedLanguage returns the primary detected language code (e.g., "en", "de").
// Returns an empty string if no language was detected.
// This method provides efficient access to language detection without JSON parsing.
func (r *ExtractionResult) GetDetectedLanguage() (string, error) {
	if r.Metadata.Language != nil {
		return *r.Metadata.Language, nil
	}

	if len(r.DetectedLanguages) > 0 {
		return r.DetectedLanguages[0], nil
	}

	return "", nil
}

// MetadataField represents a metadata field with its value and existence status.
type MetadataField struct {
	Name   string
	Value  interface{}
	IsNull bool
}

// GetMetadataField retrieves a metadata field from the extraction result.
// Field paths use dot notation for nested fields (e.g., "language", "pdf.page_count").
// Returns the field value parsed as a Go interface{}, or an error if retrieval fails.
// If the field doesn't exist, IsNull will be true in the returned MetadataField.
func (r *ExtractionResult) GetMetadataField(fieldName string) (*MetadataField, error) {
	if fieldName == "" {
		return nil, newValidationErrorWithContext("field name cannot be empty", nil, ErrorCodeValidation, nil)
	}

	metadataJSON, err := json.Marshal(r.Metadata)
	if err != nil {
		return nil, newSerializationErrorWithContext("failed to encode metadata", err, ErrorCodeValidation, nil)
	}

	var metadataMap map[string]interface{}
	if err := json.Unmarshal(metadataJSON, &metadataMap); err != nil {
		return nil, newSerializationErrorWithContext("failed to parse metadata", err, ErrorCodeValidation, nil)
	}

	value, exists := metadataMap[fieldName]
	if !exists {
		return &MetadataField{
			Name:   fieldName,
			Value:  nil,
			IsNull: true,
		}, nil
	}

	if value == nil {
		return &MetadataField{
			Name:   fieldName,
			Value:  nil,
			IsNull: true,
		}, nil
	}

	return &MetadataField{
		Name:   fieldName,
		Value:  value,
		IsNull: false,
	}, nil
}

// ResultToJSON serializes an ExtractionResult to a JSON string.
// This is useful for passing results through FFI or storing them.
func ResultToJSON(result *ExtractionResult) (string, error) {
	if result == nil {
		return "", newValidationErrorWithContext("result cannot be nil", nil, ErrorCodeValidation, nil)
	}

	data, err := json.Marshal(result)
	if err != nil {
		return "", newSerializationErrorWithContext("failed to encode result", err, ErrorCodeValidation, nil)
	}

	return string(data), nil
}

// ResultFromJSON deserializes an ExtractionResult from a JSON string.
// This is the inverse of ResultToJSON.
func ResultFromJSON(jsonStr string) (*ExtractionResult, error) {
	if jsonStr == "" {
		return nil, newValidationErrorWithContext("JSON string cannot be empty", nil, ErrorCodeValidation, nil)
	}

	var result ExtractionResult
	if err := json.Unmarshal([]byte(jsonStr), &result); err != nil {
		return nil, newSerializationErrorWithContext("failed to decode result JSON", err, ErrorCodeValidation, nil)
	}

	return &result, nil
}

// SerializeToToon serializes an ExtractionResult to TOON wire format.
// The result is first marshaled to JSON, then converted to TOON via the FFI.
func SerializeToToon(result *ExtractionResult) (string, error) {
	if result == nil {
		return "", newValidationErrorWithContext("result cannot be nil", nil, ErrorCodeValidation, nil)
	}

	jsonStr, err := ResultToJSON(result)
	if err != nil {
		return "", err
	}

	cJSON := C.CString(jsonStr)
	defer C.free(unsafe.Pointer(cJSON))

	ffiMutex.Lock()
	defer ffiMutex.Unlock()

	cToon := C.kreuzberg_serialize_to_toon(cJSON)
	if cToon == nil {
		return "", lastError()
	}
	defer C.kreuzberg_free_string(cToon)

	return C.GoString(cToon), nil
}

// SerializeToJson serializes an ExtractionResult to pretty-printed JSON format.
// The result is first marshaled to JSON, then re-serialized with pretty-printing via the FFI.
func SerializeToJson(result *ExtractionResult) (string, error) {
	if result == nil {
		return "", newValidationErrorWithContext("result cannot be nil", nil, ErrorCodeValidation, nil)
	}

	jsonStr, err := ResultToJSON(result)
	if err != nil {
		return "", err
	}

	cJSON := C.CString(jsonStr)
	defer C.free(unsafe.Pointer(cJSON))

	ffiMutex.Lock()
	defer ffiMutex.Unlock()

	cPretty := C.kreuzberg_serialize_to_json(cJSON)
	if cPretty == nil {
		return "", lastError()
	}
	defer C.kreuzberg_free_string(cPretty)

	return C.GoString(cPretty), nil
}

// String implements fmt.Stringer for ExtractionResult, showing a summary.
func (r *ExtractionResult) String() string {
	if r == nil {
		return "<nil ExtractionResult>"
	}

	return fmt.Sprintf("ExtractionResult{MimeType: %s, ContentLen: %d, Tables: %d, Chunks: %d}",
		r.MimeType, len(r.Content), len(r.Tables), len(r.Chunks))
}
