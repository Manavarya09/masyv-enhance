# MASYV Enhance Engine

AI-powered image enhancement, upscaling, and vectorization for developers.

## Features

- **Smart Pipeline** — auto-detects image type (photo/logo/text/illustration) and routes to the optimal processing pipeline
- **AI Upscaling** — Real-ESRGAN super-resolution via ONNX Runtime (2x, 4x, 8x)
- **SVG Vectorization** — raster-to-vector conversion using VTracer
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

## Quick Start

### Prerequisites

- Rust 1.80+ (`rustup install stable`)
- Node.js 18+ (`node --version`)

### Build

```bash
# Build everything
./scripts/build.sh

# Download AI models (optional, ~67MB)
./scripts/download-models.sh
```

### Usage

**Rust binary (direct):**

```bash
cd core
cargo run --release -- input.png --mode smart --scale 4 --format png
```

**Node CLI:**

```bash
cd cli
npx masyv enhance input.png --smart --scale 4 --format png
```

### CLI Options

```
masyv enhance <input> [options]

Options:
  --smart              Auto-detect image type and route (default)
  --mode <mode>        Processing mode: smart, upscale, vectorize, enhance
  --scale <n>          Upscale factor: 2, 4, 8 (default: 4)
  --format <fmt>       Output format: png, jpeg, webp, svg (default: png)
  -o, --output <path>  Output file path
  --quality <n>        JPEG quality 1-100 (default: 90)
  --batch              Process all images in a directory
  --model-dir <path>   Path to ONNX model directory
  --verbose            Enable debug logging
```

### Examples

```bash
# Smart enhance (auto-detect type)
npx masyv enhance photo.jpg --smart

# Upscale 4x with AI
npx masyv enhance photo.jpg --mode upscale --scale 4

# Vectorize a logo
npx masyv enhance logo.png --mode vectorize --format svg

# Batch process a folder
npx masyv enhance ./images/ --batch --format webp

# High-quality JPEG export
npx masyv enhance photo.png --format jpeg --quality 95
```

## Claude Plugin

The engine exposes an `enhance_image` tool for Claude integration. See `plugin/tool.json` for the schema.

**Tool call example:**

```json
{
  "image_path": "/path/to/input.png",
  "mode": "smart",
  "scale": 4,
  "format": "png"
}
```

## Project Structure

```
masyv-enhance/
├── core/                    Rust engine
│   ├── src/
│   │   ├── main.rs          CLI entrypoint
│   │   ├── lib.rs           Engine orchestrator
│   │   ├── pipeline/        Processing modules
│   │   │   ├── analyze.rs   Image type detection
│   │   │   ├── enhance.rs   Traditional processing
│   │   │   ├── upscale.rs   AI super-resolution
│   │   │   ├── vectorize.rs SVG conversion
│   │   │   └── export.rs    Format output
│   │   ├── models/
│   │   │   └── onnx.rs      ONNX model loader
│   │   └── utils/           Helpers
│   └── models/              ONNX files (downloaded)
├── cli/                     Node.js NPX wrapper
├── plugin/                  Claude MCP tool definition
└── scripts/                 Build and setup scripts
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
