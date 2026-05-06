# MASYV Enhance Engine

[![License: MIT](https://img.shields.io/github/license/Manavarya09/masyv-enhance?color=0a0a0a)](LICENSE)
[![Stars](https://img.shields.io/github/stars/Manavarya09/masyv-enhance?style=flat&color=0a0a0a)](https://github.com/Manavarya09/masyv-enhance/stargazers)
[![Issues](https://img.shields.io/github/issues/Manavarya09/masyv-enhance?color=0a0a0a)](https://github.com/Manavarya09/masyv-enhance/issues)
[![Last commit](https://img.shields.io/github/last-commit/Manavarya09/masyv-enhance?color=0a0a0a)](https://github.com/Manavarya09/masyv-enhance/commits/main)


Canva-level image enhancement for developers — via CLI, API, and Claude plugin.

[![npm](https://img.shields.io/npm/v/@masyv/enhance)](https://www.npmjs.com/package/@masyv/enhance)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Features

- **Smart Pipeline** — auto-detects image type (photo, logo, text, illustration) and routes to the optimal processing pipeline
- **AI Upscaling** — Real-ESRGAN super-resolution via ONNX Runtime (2x, 4x, 8x)
- **SVG Vectorization** — raster-to-vector conversion for logos and icons
- **Multi-format Export** — PNG, JPEG, WebP, SVG
- **CLI via NPX** — developer-friendly command line interface
- **Claude Plugin** — callable as an MCP tool from Claude

## Architecture

```
Input Image → Analyze Type → Route Pipeline → Process → Export
                  │
          ┌───────┼───────┬──────────┐
          ▼       ▼       ▼          ▼
        Photo    Logo    Text    Illustration
          │       │       │          │
      Denoise  Threshold Contrast  Denoise
      AI Upscale Vector  Sharpen   AI Upscale
      Sharpen  Optimize
```

| Detected Type | Pipeline |
|--------------|----------|
| Photo | Denoise → AI Upscale (Real-ESRGAN) → Sharpen |
| Logo | Threshold + Cleanup → Vectorize (SVG) or Upscale |
| Text | Contrast Boost → Edge Sharpen |
| Illustration | Gentle Denoise → AI Upscale |

## Quick Start

### Install from npm

```bash
npm install -g @masyv/enhance
```

### Or build from source

**Prerequisites:** Rust 1.80+, Node.js 18+

```bash
git clone https://github.com/Manavarya09/masyv-enhance.git
cd masyv-enhance

# Build everything
./scripts/build.sh

# Download AI models (optional, ~67MB)
./scripts/download-models.sh
```

## Usage

```bash
# Smart enhance — auto-detects image type
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

### Rust binary (direct)

```bash
cd core
cargo run --release -- input.png --mode smart --scale 4 --format png
```

### CLI Options

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

## AI Models

For AI-powered upscaling, download the Real-ESRGAN model (~67MB):

```bash
./scripts/download-models.sh
```

Without the model, the engine gracefully falls back to high-quality Lanczos3 interpolation.

## Claude Plugin

The engine exposes an `enhance_image` tool for Claude integration. See [`plugin/tool.json`](plugin/tool.json) for the full schema.

```json
{
  "image_path": "/path/to/input.png",
  "mode": "smart",
  "scale": 4,
  "format": "png"
}
```

Claude can call this tool, receive the processed output path, and describe the result.

## Project Structure

```
masyv-enhance/
├── core/                    Rust engine
│   ├── src/
│   │   ├── main.rs          CLI entrypoint (clap)
│   │   ├── lib.rs           Engine orchestrator
│   │   ├── pipeline/
│   │   │   ├── analyze.rs   Image type detection
│   │   │   ├── enhance.rs   Denoise, contrast, sharpen
│   │   │   ├── upscale.rs   AI super-resolution (ONNX)
│   │   │   ├── vectorize.rs Raster → SVG (VTracer)
│   │   │   └── export.rs    Multi-format output
│   │   ├── models/
│   │   │   └── onnx.rs      ONNX model loader + tiled inference
│   │   └── utils/           Image & filesystem helpers
│   └── models/              ONNX model files (downloaded)
├── cli/                     Node.js NPX wrapper
│   ├── bin/masyv.js         CLI entrypoint
│   └── src/                 Runner + logger
├── plugin/                  Claude MCP tool definition
│   └── tool.json
├── scripts/
│   ├── build.sh             Build Rust + link to CLI
│   └── download-models.sh   Fetch ONNX models
└── tests/samples/           Test images
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Core Engine | Rust |
| AI Inference | ONNX Runtime (ort) |
| Upscaling Model | Real-ESRGAN x4plus |
| Vectorization | VTracer |
| Image Processing | image + imageproc |
| Parallelism | rayon |
| CLI | clap (Rust) + commander (Node) |

## Performance

- Typical processing: 2-5 seconds for standard images
- Multi-threaded ONNX inference
- Tile-based processing for large images (avoids OOM)
- Lazy model loading (loaded once, reused)

## License

MIT
