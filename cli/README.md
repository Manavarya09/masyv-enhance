# @masyv/enhance

AI-powered image enhancement, upscaling, and vectorization for developers.

## Features

- **Smart Pipeline** — auto-detects image type (photo, logo, text, illustration) and routes to the optimal processing pipeline
- **AI Upscaling** — Real-ESRGAN super-resolution via ONNX Runtime (2x, 4x, 8x)
- **SVG Vectorization** — raster-to-vector conversion for logos and icons
- **Multi-format Export** — PNG, JPEG, WebP, SVG
- **Claude Plugin** — callable as an MCP tool

## Install

```bash
npm install -g @masyv/enhance
```

## Usage

```bash
# Smart enhance (auto-detect image type)
masyv enhance photo.jpg --smart

# AI upscale 4x
masyv enhance photo.jpg --mode upscale --scale 4

# Vectorize a logo to SVG
masyv enhance logo.png --mode vectorize --format svg

# Batch process a folder
masyv enhance ./images/ --batch --format webp

# High-quality JPEG export
masyv enhance photo.png --format jpeg --quality 95
```

## CLI Options

```
masyv enhance <input> [options]

Options:
  --smart              Auto-detect image type and route to best pipeline
  --mode <mode>        Processing mode: smart, upscale, vectorize, enhance
  --scale <n>          Upscale factor: 2, 4, 8 (default: 4)
  --format <fmt>       Output format: png, jpeg, webp, svg (default: png)
  -o, --output <path>  Output file path
  --quality <n>        JPEG quality 1-100 (default: 90)
  --batch              Process all images in a directory
  --model-dir <path>   Path to ONNX model directory
  --verbose            Enable debug logging
```

## Smart Pipeline

The engine automatically classifies your image and applies the best processing:

| Detected Type | Pipeline |
|--------------|----------|
| Photo | Denoise → AI Upscale → Sharpen |
| Logo | Threshold + Cleanup → Vectorize / Upscale |
| Text | Contrast Boost → Edge Sharpen |
| Illustration | Gentle Denoise → AI Upscale |

## AI Models

For AI-powered upscaling, download the Real-ESRGAN model (~67MB):

```bash
# From the project repo
./scripts/download-models.sh

# Or manually
curl -L -o models/realesrgan_x4plus.onnx \
  https://huggingface.co/qualcomm/Real-ESRGAN-x4plus/resolve/main/Real-ESRGAN-x4plus.onnx
```

Without the model, the engine gracefully falls back to high-quality Lanczos3 interpolation.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core Engine | Rust |
| AI Inference | ONNX Runtime |
| Upscaling | Real-ESRGAN x4plus |
| Vectorization | VTracer |
| Image Processing | image + imageproc |
| CLI | clap (Rust) + commander (Node) |

## Links

- [GitHub](https://github.com/Manavarya09/masyv-enhance)
- [Issues](https://github.com/Manavarya09/masyv-enhance/issues)

## License

MIT
