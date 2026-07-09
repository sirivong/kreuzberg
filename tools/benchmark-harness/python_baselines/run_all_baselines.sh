#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

MODELS="${MODELS:-deepseek paddleocr}"
FIXTURES="${FIXTURES:-../../fixtures}"
OUTPUT_BASE="${OUTPUT_BASE:-baselines}"
DEVICE="${DEVICE:-cuda}"

mkdir -p "$OUTPUT_BASE"

echo "======================================================================"
echo "VLM-OCR Baseline Generation Runner"
echo "======================================================================"
echo "Models to run: $MODELS"
echo "Fixtures: $FIXTURES"
echo "Output base: $OUTPUT_BASE"
echo "Device: $DEVICE"
echo "======================================================================"

if [ ! -d "$FIXTURES" ]; then
  echo "ERROR: Fixtures directory not found: $FIXTURES"
  exit 1
fi

all_success=true

for model in $MODELS; do
  case "$model" in
  deepseek)
    echo ""
    echo ">>> Running DeepSeek-OCR baseline..."
    output_dir="$OUTPUT_BASE/deepseek_ocr"
    if python deepseek_ocr_baseline.py \
      --fixtures "$FIXTURES" \
      --output "$output_dir" \
      --device "$DEVICE"; then
      echo "✓ DeepSeek-OCR complete"
    else
      echo "✗ DeepSeek-OCR failed"
      all_success=false
    fi
    ;;

  paddleocr)
    echo ""
    echo ">>> Running PaddleOCR-VL baseline..."
    output_dir="$OUTPUT_BASE/paddleocr_vl"
    if python paddleocr_vl_baseline.py \
      --fixtures "$FIXTURES" \
      --output "$output_dir" \
      --device "$DEVICE"; then
      echo "✓ PaddleOCR-VL complete"
    else
      echo "✗ PaddleOCR-VL failed"
      all_success=false
    fi
    ;;

  *)
    echo "WARNING: Unknown model: $model (skipping)"
    ;;
  esac
done

echo ""
echo "======================================================================"
echo "Baseline Generation Summary"
echo "======================================================================"
if [ "$all_success" = true ]; then
  echo "✓ All baseline generation runs completed successfully"
  echo "  Check $OUTPUT_BASE/ for generated baseline files"
  echo "======================================================================"
  exit 0
else
  echo "✗ One or more baseline generation runs failed"
  echo "  Check logs above for details"
  echo "======================================================================"
  exit 1
fi
