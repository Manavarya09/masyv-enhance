#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
MODEL_DIR="$(dirname "$SCRIPT_DIR")/core/models"

mkdir -p "$MODEL_DIR"

MODEL_FILE="$MODEL_DIR/realesrgan_x4plus.onnx"

if [ -f "$MODEL_FILE" ]; then
    echo "Model already exists: $MODEL_FILE"
    exit 0
fi

echo "==> Downloading Real-ESRGAN x4plus ONNX model (~67MB)..."
echo "    Source: HuggingFace (qualcomm/Real-ESRGAN-x4plus)"

# Try HuggingFace first
URL="https://huggingface.co/qualcomm/Real-ESRGAN-x4plus/resolve/main/Real-ESRGAN-x4plus.onnx"

if command -v curl &>/dev/null; then
    curl -L --progress-bar -o "$MODEL_FILE" "$URL"
elif command -v wget &>/dev/null; then
    wget --show-progress -O "$MODEL_FILE" "$URL"
else
    echo "Error: curl or wget required to download models"
    exit 1
fi

if [ -f "$MODEL_FILE" ]; then
    SIZE=$(wc -c < "$MODEL_FILE" | tr -d ' ')
    echo "==> Downloaded: $MODEL_FILE ($SIZE bytes)"
else
    echo "Error: download failed"
    exit 1
fi
