#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Building MASYV Enhance Engine"

# Build Rust core
echo "==> Compiling Rust core (release)..."
cd "$PROJECT_DIR/core"
cargo build --release

# Copy binary to CLI bin
echo "==> Copying binary to CLI..."
mkdir -p "$PROJECT_DIR/cli/bin"
cp "$PROJECT_DIR/core/target/release/masyv-core" "$PROJECT_DIR/cli/bin/"

# Install Node dependencies
echo "==> Installing Node dependencies..."
cd "$PROJECT_DIR/cli"
npm install

echo ""
echo "==> Build complete!"
echo "    Binary: cli/bin/masyv-core"
echo "    CLI:    npx masyv enhance <image>"
