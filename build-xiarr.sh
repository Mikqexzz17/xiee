#!/bin/bash
# build-xiarr.sh — Buduje XIARR (Xiee Browser) dla Linux
# Wymaga: Node.js, npm, Rust, cargo-tauri

set -e
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$SCRIPT_DIR/xiarr"

echo "[XIARR] Sprawdzam zaleznosci..."

# Node.js
if ! command -v node &> /dev/null; then
    echo "[XIARR] Instaluje Node.js..."
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
fi

# cargo-tauri
if ! command -v cargo-tauri &> /dev/null; then
    echo "[XIARR] Instaluje Tauri CLI..."
    source ~/.cargo/env
    cargo install tauri-cli --version "^2" --locked
fi

echo "[XIARR] Instaluje zaleznosci npm..."
npm install

echo "[XIARR] Buduje XIARR..."
source ~/.cargo/env
cargo tauri build --bundles none

echo "[XIARR] Gotowe!"
echo "[XIARR] Binarny: target/release/xiarr"
