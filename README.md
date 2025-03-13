# Kreuzberg

Kreuzberg is a Python library for text extraction from documents. It provides a unified interface for extracting text from PDFs, images, office documents, and more, with both async and sync APIs.

## Why Kreuzberg?

- **Simple and Hassle-Free**: Clean API that just works, without complex configuration
- **Local Processing**: No external API calls or cloud dependencies required
- **Resource Efficient**: Lightweight processing without GPU requirements
- **Format Support**: Comprehensive support for documents, images, and text formats
- **Multiple OCR Engines**: Support for Tesseract, EasyOCR, and PaddleOCR
- **Modern Python**: Built with async/await, type hints, and a functional-first approach
- **Permissive OSS**: MIT licensed with permissively licensed dependencies

## Installation

### Basic installation

```bash
pip install kreuzberg
```

This installs Kreuzberg with basic dependencies. To use OCR features, you'll need to install the system-level Tesseract OCR engine:

```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr

# macOS
brew install tesseract

# Windows (via chocolatey)
choco install tesseract
```

### Alternative OCR engines

```bash
# Install with EasyOCR support (pure Python, no system dependencies)
pip install "kreuzberg[easyocr]"

# Install with PaddleOCR support (good for Asian languages)
pip install "kreuzberg[paddleocr]"
```

## Quick Examples

### Basic text extraction

```python
import asyncio
from kreuzberg import extract_file

async def main():
    # Extract text from a PDF
    result = await extract_file("document.pdf")
    print(result.content)

    # Extract text from an image
    result = await extract_file("scan.jpg")
    print(result.content)

    # Extract text from a Word document
    result = await extract_file("report.docx")
    print(result.content)

asyncio.run(main())
```

### OCR configuration

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig, PSMMode

async def extract_with_ocr():
    # Extract text from a German document
    result = await extract_file(
        "german_document.pdf",
        config=ExtractionConfig(
            force_ocr=True,
            ocr_config=TesseractConfig(
                language="deu", psm=PSMMode.SINGLE_BLOCK  # German language  # Treat as a single text block
            ),
        ),
    )
    print(result.content)
```

### Batch processing

```python
from kreuzberg import batch_extract_file

async def process_documents():
    file_paths = ["document1.pdf", "document2.docx", "image.jpg"]
    results = await batch_extract_file(file_paths)

    for path, result in zip(file_paths, results):
        print(f"File: {path}")
        print(f"Content: {result.content[:100]}...")  # First 100 chars
```

### Synchronous API

```python
from kreuzberg import extract_file_sync

# Extract text synchronously
result = extract_file_sync("document.pdf")
print(result.content)
```

## Documentation

For more detailed information, check out the full documentation:

- [Getting Started](https://example.com/getting-started)
- [User Guide](https://example.com/user-guide)
- [API Reference](https://example.com/api-reference)
- [Examples](https://example.com/examples)

## Supported Formats

Kreuzberg supports a wide range of document formats:

- **Documents**: PDF, DOCX, DOC, RTF, TXT, EPUB, etc.
- **Images**: JPG, PNG, TIFF, BMP, GIF, etc.
- **Spreadsheets**: XLSX, XLS, CSV, etc.
- **Presentations**: PPTX, PPT, etc.
- **Web Content**: HTML, XML, etc.

## OCR Engines

Kreuzberg supports multiple OCR engines:

- **Tesseract** (Default): Lightweight, fast startup, requires system installation
- **EasyOCR**: Good for many languages, pure Python, but downloads models on first use
- **PaddleOCR**: Excellent for Asian languages, pure Python, but downloads models on first use

For comparison and selection guidance, see the [OCR Backends](https://example.com/ocr-backends) documentation.

## Contribution

This library is open to contribution. Feel free to open issues or submit PRs. It's better to discuss issues before submitting PRs to avoid disappointment.

### Local Development

1. Clone the repo

1. Install the system dependencies

1. Install the full dependencies with `uv sync`

1. Install the pre-commit hooks with:

    ```shell
    pre-commit install && pre-commit install --hook-type commit-msg
    ```

1. Make your changes and submit a PR

## License

This library is released under the MIT license.
