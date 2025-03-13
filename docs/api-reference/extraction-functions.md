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
