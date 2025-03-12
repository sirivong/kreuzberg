# OCR Configuration

Kreuzberg supports multiple OCR engines, each with its own configuration options. The OCR engine is selected using the `ocr_backend` parameter in the `ExtractionConfig` object, and configured with the appropriate configuration class in the `ocr_config` parameter.

## Choosing an OCR Engine

Each OCR engine has its own strengths and trade-offs:

### Tesseract (Default)

**Advantages:**

- Lightweight and CPU-optimized
- No model downloads required (faster startup)
- Mature and widely used
- Lower memory usage

**Considerations:**

- Requires system-level installation
- May have lower accuracy for some languages or complex layouts
- More configuration may be needed for optimal results

### EasyOCR

**Advantages:**

- Good accuracy across multiple languages
- No system dependencies required (pure Python)
- Simple configuration

**Considerations:**

- Larger memory footprint
- Slower first-run due to model downloads
- Heavier resource usage

### PaddleOCR

**Advantages:**

- Excellent accuracy, especially for Asian languages
- No system dependencies required
- Modern deep learning architecture

**Considerations:**

- Largest memory footprint of the three options
- Slower first-run due to model downloads
- More resource-intensive

### Disabling OCR

If you set `ocr_backend=None` in the `ExtractionConfig`:

- No OCR will be performed
- Searchable PDFs will still extract embedded text
- For images and non-searchable PDFs, an empty string will be returned for content
- Useful when you want to explicitly avoid OCR processing

## TesseractConfig

Configuration options for the Tesseract OCR engine (default):

::: kreuzberg.TesseractConfig

### PSMMode (Page Segmentation Mode)

Control how Tesseract analyzes page layout:

::: kreuzberg.PSMMode

## EasyOCRConfig

Configuration options for the EasyOCR engine:

::: kreuzberg.EasyOCRConfig

## PaddleOCRConfig

Configuration options for the PaddleOCR engine:

::: kreuzberg.PaddleOCRConfig

## Example Usage

```python
from kreuzberg import extract_file, ExtractionConfig, TesseractConfig, EasyOCRConfig, PaddleOCRConfig, PSMMode

async def main():
    # Configure Tesseract OCR with multilingual support
    tesseract_result = await extract_file(
        "document.pdf",
        config=ExtractionConfig(
            ocr_backend="tesseract",
            ocr_config=TesseractConfig(language="eng+deu", psm=PSMMode.SINGLE_BLOCK),  # English and German
        ),
    )

    # Configure EasyOCR with multiple languages
    easyocr_result = await extract_file(
        "document.jpg",
        config=ExtractionConfig(
            ocr_backend="easyocr", ocr_config=EasyOCRConfig(language_list=["en", "de"])  # English and German
        ),
    )

    # Configure PaddleOCR for Chinese text
    paddleocr_result = await extract_file(
        "chinese_document.jpg",
        config=ExtractionConfig(ocr_backend="paddleocr", ocr_config=PaddleOCRConfig(language="ch")),  # Chinese
    )
```
