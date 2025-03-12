# Extraction Functions

Kreuzberg provides both asynchronous and synchronous functions for text extraction from files and byte data. All functions accept an optional `ExtractionConfig` parameter for configuring the extraction process, including OCR options.

## Asynchronous Functions

These functions return awaitable coroutines that must be awaited or run in an asyncio event loop.

### extract_file

Extract text from a file path:

::: kreuzberg.extract_file

### extract_bytes

Extract text from raw bytes:

::: kreuzberg.extract_bytes

### batch_extract_file

Process multiple files concurrently:

::: kreuzberg.batch_extract_file

### batch_extract_bytes

Process multiple byte contents concurrently:

::: kreuzberg.batch_extract_bytes

## Synchronous Functions

These functions block until extraction is complete and are suitable for non-async contexts.

### extract_file_sync

Synchronous version of extract_file:

::: kreuzberg.extract_file_sync

### extract_bytes_sync

Synchronous version of extract_bytes:

::: kreuzberg.extract_bytes_sync

### batch_extract_file_sync

Synchronous version of batch_extract_file:

::: kreuzberg.batch_extract_file_sync

### batch_extract_bytes_sync

Synchronous version of batch_extract_bytes:

::: kreuzberg.batch_extract_bytes_sync

## Configuration

All extraction functions accept an optional `config` parameter of type `ExtractionConfig`. This object allows you to:

- Control OCR behavior with `force_ocr` and `ocr_backend`
- Provide engine-specific OCR configuration via `ocr_config`
- Add validation and post-processing hooks

See the [Types](./types.md) reference for details on the available configuration options.

### Examples

#### Basic Usage

```python
from kreuzberg import extract_file, ExtractionConfig

# Simple extraction with default configuration
result = await extract_file("document.pdf")

# Extraction with custom configuration
result = await extract_file("document.pdf", config=ExtractionConfig(force_ocr=True))
```

#### OCR Configuration

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig, PSMMode

# Configure Tesseract OCR with specific language and page segmentation mode
result = await extract_file(
    "document.pdf",
    config=ExtractionConfig(force_ocr=True, ocr_config=TesseractConfig(language="eng+deu", psm=PSMMode.SINGLE_BLOCK)),
)
```

#### Alternative OCR Engines

```python
from kreuzberg import extract_file, ExtractionConfig, EasyOCRConfig, PaddleOCRConfig

# Use EasyOCR backend
result = await extract_file(
    "document.jpg", config=ExtractionConfig(ocr_backend="easyocr", ocr_config=EasyOCRConfig(language_list=["en", "de"]))
)

# Use PaddleOCR backend
result = await extract_file(
    "chinese_document.jpg", config=ExtractionConfig(ocr_backend="paddleocr", ocr_config=PaddleOCRConfig(language="ch"))
)
```

#### Batch Processing

```python
from kreuzberg import batch_extract_file, ExtractionConfig

# Process multiple files with the same configuration
file_paths = ["document1.pdf", "document2.docx", "image.jpg"]
config = ExtractionConfig(force_ocr=True)
results = await batch_extract_file(file_paths, config=config)
```

#### Synchronous API

```python
from kreuzberg import extract_file_sync, ExtractionConfig, TesseractConfig

# Synchronous extraction with configuration
result = extract_file_sync("document.pdf", config=ExtractionConfig(ocr_config=TesseractConfig(language="eng")))
```
