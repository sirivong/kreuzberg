# OCR Configuration

Kreuzberg provides comprehensive options for configuring Optical Character Recognition (OCR) to optimize text extraction from images and scanned documents.

## OCR Configuration

All extraction functions in Kreuzberg accept an `ExtractionConfig` object that can contain OCR configuration:

### Language Configuration

The `language` parameter in a `TesseractConfig` object specifies which language model Tesseract should use for OCR:

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig

# Extract text from a German document
result = await extract_file("german_document.pdf", config=ExtractionConfig(ocr_config=TesseractConfig(language="deu")))
```

#### Supported Language Codes

| Language            | Code      | Language           | Code      |
| ------------------- | --------- | ------------------ | --------- |
| English             | `eng`     | German             | `deu`     |
| French              | `fra`     | Spanish            | `spa`     |
| Italian             | `ita`     | Japanese           | `jpn`     |
| Korean              | `kor`     | Simplified Chinese | `chi_sim` |
| Traditional Chinese | `chi_tra` | Russian            | `rus`     |
| Arabic              | `ara`     | Hindi              | `hin`     |

#### Multi-Language Support

You can specify multiple languages by joining codes with a plus sign:

```python
# Document contains both English and German text
result = await extract_file("multilingual.pdf", config=ExtractionConfig(ocr_config=TesseractConfig(language="eng+deu")))
```

!!! note

    The order of languages affects processing time and accuracy. The first language is treated as the primary language.

#### Language Installation

For Tesseract to recognize languages other than English, you need to install the corresponding language data:

- **Ubuntu/Debian**: `sudo apt-get install tesseract-ocr-<lang-code>`
- **macOS**: `brew install tesseract-lang` (installs all languages)
- **Windows**: Download language data from [GitHub](https://github.com/tesseract-ocr/tessdata)

### Page Segmentation Mode (PSM)

The `psm` parameter in a `TesseractConfig` object controls how Tesseract analyzes the layout of the page:

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig, PSMMode

# Extract text from a document with a simple layout
result = await extract_file("document.pdf", config=ExtractionConfig(ocr_config=TesseractConfig(psm=PSMMode.SINGLE_BLOCK)))
```

#### Available PSM Modes

| Mode                 | Enum Value                | Description                                              | Best For                                       |
| -------------------- | ------------------------- | -------------------------------------------------------- | ---------------------------------------------- |
| Automatic            | `PSMMode.AUTO`            | Automatic page segmentation with orientation detection   | General purpose (default)                      |
| Single Block         | `PSMMode.SINGLE_BLOCK`    | Treat the image as a single text block                   | Simple layouts, preserving paragraph structure |
| Single Line          | `PSMMode.SINGLE_LINE`     | Treat the image as a single text line                    | Receipts, labels, single-line text             |
| Single Word          | `PSMMode.SINGLE_WORD`     | Treat the image as a single word                         | Word recognition tasks                         |
| Single Character     | `PSMMode.SINGLE_CHAR`     | Treat the image as a single character                    | Character recognition tasks                    |
| Sparse Text          | `PSMMode.SPARSE_TEXT`     | Find as much text as possible without assuming structure | Forms, tables, scattered text                  |
| Sparse Text with OSD | `PSMMode.SPARSE_TEXT_OSD` | Like SPARSE_TEXT with orientation detection              | Complex layouts with varying text orientation  |

### Forcing OCR

By default, Kreuzberg will only use OCR for images and scanned PDFs. For searchable PDFs, it will extract text directly. You can override this behavior with the `force_ocr` parameter in the `ExtractionConfig` object:

```python
from kreuzberg import extract_file, ExtractionConfig

# Force OCR even for searchable PDFs
result = await extract_file("searchable.pdf", config=ExtractionConfig(force_ocr=True))
```

This is useful when:

- The PDF contains both searchable text and images with text
- The embedded text in the PDF has encoding or extraction issues
- You want consistent processing across all documents

## OCR Engine Selection

Kreuzberg supports multiple OCR engines:

### Tesseract (Default)

Tesseract is the default OCR engine and requires no additional installation beyond the system dependency.

### EasyOCR (Optional)

To use EasyOCR:

1. Install with the extra: `pip install "kreuzberg[easyocr]"`
1. Use the `ocr_backend` parameter in the `ExtractionConfig` object:

```python
from kreuzberg import extract_file, ExtractionConfig, EasyOCRConfig

result = await extract_file(
    "document.jpg",
    config=ExtractionConfig(
        ocr_backend="easyocr", ocr_config=EasyOCRConfig(language_list=["en"])  # EasyOCR uses different language codes
    ),
)
```

### PaddleOCR (Optional)

To use PaddleOCR:

1. Install with the extra: `pip install "kreuzberg[paddleocr]"`
1. Use the `ocr_backend` parameter in the `ExtractionConfig` object:

```python
from kreuzberg import extract_file, ExtractionConfig, PaddleOCRConfig

result = await extract_file(
    "document.jpg",
    config=ExtractionConfig(
        ocr_backend="paddleocr", ocr_config=PaddleOCRConfig(language="en")  # PaddleOCR uses different language codes
    ),
)
```

!!! note

    For PaddleOCR, the supported language codes are different: `ch` (Chinese), `en` (English), `french`, `german`, `japan`, and `korean`.

## Performance Optimization

OCR performance and parallel processing can be controlled through process handlers and extraction hooks which are configured in the `ExtractionConfig` object. The default configuration handles performance optimization automatically.

This is useful for:

- Limiting resource usage on systems with limited memory
- Optimizing performance on systems with many CPU cores
- Balancing OCR tasks with other application workloads

## Best Practices

- **Language Selection**: Always specify the correct language for your documents to improve OCR accuracy
- **PSM Mode Selection**: Choose the appropriate PSM mode based on your document layout:
    - Use `PSM.SINGLE_BLOCK` for documents with simple layouts
    - Use `PSM.SPARSE_TEXT` for forms or documents with tables
    - Use `PSM.SINGLE_LINE` for receipts or labels
- **Image Quality**: For best results, ensure images are:
    - High resolution (at least 300 DPI)
    - Well-lit with good contrast
    - Not skewed or rotated
- **Performance**: For batch processing, adjust `max_processes` based on your system's capabilities
