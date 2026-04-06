#!/bin/bash
echo ""
echo "  ========================================"
echo "    CrustBrowser Installer"
echo "  ========================================"
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust is not installed."
    echo "  Install it from: https://www.rust-lang.org/tools/install"
    exit 1
fi

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "[ERROR] Node.js is not installed."
    echo "  Install it from: https://nodejs.org/"
    exit 1
fi

echo "[1/3] Building Rust binary..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "[ERROR] Rust build failed."
    exit 1
fi

echo "[2/3] Installing CLI dependencies..."
cd CLI
npm install
cd ..

echo "[3/3] Setup complete!"
echo ""
echo "  To run CrustBrowser:"
echo "    node CLI/index.js"
echo ""
