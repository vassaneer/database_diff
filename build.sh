#!/bin/bash
# Build script for compiling Rust to WASM

set -e  # Exit on any error

echo "Database Schema Comparison Tool - Build Script"
echo "=============================================="

# Check if Rust is installed
if ! command -v rustc &> /dev/null
then
    echo "Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Rust version: $(rustc --version)"

# Check if Cargo is installed
if ! command -v cargo &> /dev/null
then
    echo "Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

echo "Cargo version: $(cargo --version)"

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found. Installing..."
    # Install wasm-pack
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

echo "wasm-pack version: $(wasm-pack --version)"

# Add wasm target if not already added
echo "Adding wasm32-unknown-unknown target if needed..."
rustup target add wasm32-unknown-unknown

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web

echo "Build complete!"
echo "To run the application:"
echo "1. Start a web server in this directory (e.g., python3 -m http.server 8000)"
echo "2. Open index.html in your web browser"
