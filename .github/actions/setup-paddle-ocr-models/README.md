# Setup PaddleOCR Models Cache

GitHub Action to download and cache PaddleOCR ONNX models for CI testing and development.

## Overview

This action manages the setup of PaddleOCR PP-OCRv4 ONNX models used by the `kreuzberg-paddle-ocr` crate for optical character recognition testing. It:

- Downloads three model types (detection, classification, recognition) from Hugging Face
- Caches models per OS and CPU architecture (Linux x86_64, Linux ARM64, macOS, Windows)
- Provides environment variables for downstream use
- Outputs cache hit status and available model information
- Gracefully handles download failures (continues with available models)

## Models

The action downloads pre-converted ONNX format models from the `Kreuzberg/paddleocr-onnx-models` Hugging Face repository:

| Model Type | File | Size | Purpose |
|-----------|------|------|---------|
| Detection (det) | `en_PP-OCRv4_det_infer.onnx` | ~4.5 MB | Text location detection |
| Classification (cls) | `ch_ppocr_mobile_v2.0_cls_infer.onnx` | ~1.5 MB | Text orientation classification |
| Recognition (rec) | `en_PP-OCRv4_rec_infer.onnx` | ~10 MB | Text character recognition |

**Total cache size: ~16 MB per OS/architecture combination**

## Usage

### Basic Usage

```yaml
- uses: ./.github/actions/setup-paddle-ocr-models
```

### With Custom Cache Suffix

```yaml
- uses: ./.github/actions/setup-paddle-ocr-models
  with:
    cache-key-suffix: my-paddle-ocr-v4
```

### Disable Caching

For cross-architecture builds where caching doesn't help:

```yaml
- uses: ./.github/actions/setup-paddle-ocr-models
  with:
    cache-enabled: false
```

### Download Specific Models Only

```yaml
- uses: ./.github/actions/setup-paddle-ocr-models
  with:
    models: "det,rec"  # Skip classification model
```

## Inputs

| Name | Description | Required | Default |
|------|-------------|----------|---------|
| `cache-enabled` | Enable model caching (set false for cross-arch builds) | No | `true` |
| `models` | Comma-separated list of models to setup (det,cls,rec or subset) | No | `det,cls,rec` |
| `cache-key-suffix` | Suffix for cache key to differentiate model sets | No | `paddle-ocr-v4-onnx` |

## Outputs

| Name | Description |
|------|-------------|
| `cache-hit` | Whether models were restored from cache (true/false) |
| `cache-dir` | Path to the PaddleOCR model cache directory |
| `models-available` | Comma-separated list of available models after setup |

## Outputs as Environment Variables

The action automatically exports:

- `PADDLE_OCR_MODEL_CACHE`: Absolute path to model cache directory

## Cache Strategy

Models are cached using GitHub Actions cache with the following key structure:

```
paddle-ocr-v4-onnx-{OS}-{ARCHITECTURE}-v1
```

Cache restoration order (restore-keys):
1. Exact match: `paddle-ocr-v4-onnx-{OS}-{ARCHITECTURE}-v1`
2. OS-Architecture: `paddle-ocr-v4-onnx-{OS}-{ARCHITECTURE}-`
3. OS only: `paddle-ocr-v4-onnx-{OS}-`
4. Any: `paddle-ocr-v4-onnx-`

## Example: CI Rust Workflow Integration

```yaml
jobs:
  paddle-ocr-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: ./.github/actions/setup-paddle-ocr-models
        id: paddle-models

      - name: Run PaddleOCR tests
        run: cargo test --package kreuzberg-paddle-ocr
        env:
          PADDLE_OCR_MODEL_CACHE: ${{ steps.paddle-models.outputs.cache-dir }}

      - name: Report cache status
        if: always()
        run: |
          echo "Cache hit: ${{ steps.paddle-models.outputs.cache-hit }}"
          echo "Available models: ${{ steps.paddle-models.outputs.models-available }}"
```

## Error Handling

The action uses `continue-on-error: true` for individual model downloads. This means:

- If a model download fails, it logs a warning but continues
- The action reports which models are actually available in the output
- Downstream tests can check `models-available` to know what's available
- If all models fail, tests can fall back to alternative behavior

## Download Sources

Models are downloaded from:

```
https://huggingface.co/Kreuzberg/paddleocr-onnx-models/resolve/main/
```

If this repository becomes unavailable, the action will fail gracefully. Alternative sources can be configured by modifying the `MODEL_URL` environment variables in the action.

## Troubleshooting

### Models not being cached

1. Check that `cache-enabled` is not set to `false`
2. Verify GitHub Actions cache is not full (max 10 GB per repository)
3. Check runner OS and architecture match cache keys
4. View cache in repository settings (Settings → Actions → Caches)

### Download timeouts

If downloads timeout:
- Increase the 300-second timeout in the action steps
- Check Hugging Face API availability
- Try reducing the number of models (`models: "det,rec"`)

### Verifying models are present

Check that all expected models exist in the correct directory structure:

```bash
ls -lh ~/.cache/kreuzberg/paddle-ocr/
```

Expected output:
```
drwxr-xr-x det/
drwxr-xr-x cls/
drwxr-xr-x rec/

ls -lh ~/.cache/kreuzberg/paddle-ocr/det/
-rw-r--r-- model.onnx (4-5 MB)

ls -lh ~/.cache/kreuzberg/paddle-ocr/cls/
-rw-r--r-- model.onnx (1-2 MB)

ls -lh ~/.cache/kreuzberg/paddle-ocr/rec/
-rw-r--r-- model.onnx (9-11 MB)
```

The directory structure must match what `ModelManager` expects in `model_manager.rs`.

## Performance Impact

- **First run (no cache)**: ~20-30 seconds (download time depends on network)
- **Cached run**: <1 second (cache restore)
- **Cache size**: ~16 MB per OS/architecture
- **Network bandwidth**: ~16 MB download on cache miss

## Related Actions

- `.github/actions/setup-tesseract-cache` - Similar caching for Tesseract models
- `.github/actions/cache-hf-fastembed` - Hugging Face model caching for fastembed
- `.github/actions/setup-onnx-runtime` - ONNX Runtime setup for inference

## See Also

- [PaddleOCR Documentation](https://github.com/PaddlePaddle/PaddleOCR)
- [kreuzberg-paddle-ocr crate](../../crates/kreuzberg-paddle-ocr)
- [ModelManager source](../../crates/kreuzberg/src/paddle_ocr/model_manager.rs)
