# Exceptions

Kreuzberg defines several exception types to provide detailed error information during the extraction process.

## KreuzbergError

The base exception class for all Kreuzberg-specific errors:

::: kreuzberg.KreuzbergError

## MissingDependencyError

Raised when a required dependency is not available:

::: kreuzberg.MissingDependencyError

## OCRError

Raised when OCR processing fails:

::: kreuzberg.OCRError

## ParsingError

Raised when document parsing fails:

::: kreuzberg.ParsingError

## ValidationError

Raised when validation of extraction configuration or results fails:

::: kreuzberg.ValidationError

## Error Handling Example

```python
from kreuzberg import extract_file, ExtractionConfig
from kreuzberg import KreuzbergError, MissingDependencyError, OCRError, ParsingError, ValidationError

async def safe_extract(file_path):
    try:
        config = ExtractionConfig()
        result = await extract_file(file_path, config=config)
        return result.content
    except MissingDependencyError as e:
        print(f"Missing dependency: {e}")
        print("Install required dependencies and try again.")
    except OCRError as e:
        print(f"OCR processing failed: {e}")
    except ParsingError as e:
        print(f"Document parsing failed: {e}")
    except ValidationError as e:
        print(f"Configuration validation failed: {e}")
    except KreuzbergError as e:
        print(f"Extraction error: {e}")
    except Exception as e:
        print(f"Unexpected error: {e}")

    return None
```
