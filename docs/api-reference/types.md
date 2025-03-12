# Types

Kreuzberg uses several data structures to represent extraction results, configuration options, and metadata.

## ExtractionResult

The result of a file extraction, containing the extracted text, MIME type, and metadata:

::: kreuzberg.ExtractionResult

## ExtractionConfig

Configuration options for extraction functions:

::: kreuzberg.ExtractionConfig

## OCR Configuration

### TesseractConfig

::: kreuzberg.TesseractConfig

### EasyOCRConfig

::: kreuzberg.EasyOCRConfig

### PaddleOCRConfig

::: kreuzberg.PaddleOCRConfig

## PSMMode (Page Segmentation Mode)

::: kreuzberg.PSMMode

## Metadata

A TypedDict that contains optional metadata fields extracted from documents:

::: kreuzberg.Metadata
