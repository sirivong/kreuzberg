# API Reference

This section provides detailed documentation for the Kreuzberg public API.

## Core Components

- [Extraction Functions](extraction-functions.md) - Functions for extracting text from files and bytes
- [Types](types.md) - Data structures for extraction results and configuration
- [OCR Configuration](ocr-configuration.md) - Configuration options for OCR engines
- [Exceptions](exceptions.md) - Error types and handling

## Public API

All components documented in this section are exported directly from the `kreuzberg` package and can be imported as follows:

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig  # etc.
```

## API Overview

Kreuzberg's API is organized around a few key concepts:

1. **Extraction Functions**: Main entry points for extracting text from documents
1. **Configuration Objects**: Control extraction behavior, OCR settings, and more
1. **Result Objects**: Contain extracted text, metadata, and format information
1. **OCR Backends**: Pluggable OCR engines with specific configuration options

## Core Function Patterns

All extraction functions follow these patterns:

### Async API

```python
from kreuzberg import extract_file, ExtractionConfig

# Basic usage
result = await extract_file("document.pdf")

# With configuration
result = await extract_file("document.pdf", config=ExtractionConfig(force_ocr=True))
```

### Sync API

```python
from kreuzberg import extract_file_sync, ExtractionConfig

# Basic usage
result = extract_file_sync("document.pdf")

# With configuration
result = extract_file_sync("document.pdf", config=ExtractionConfig(force_ocr=True))
```
