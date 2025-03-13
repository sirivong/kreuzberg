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
