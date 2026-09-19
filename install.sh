#!/bin/bash
# install.sh - Instaluje Xiee OS binaria do /usr/bin
# Uruchom jako root: sudo ./install.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE="$SCRIPT_DIR/target/release"
XIARR="$SCRIPT_DIR/xiarr/src-tauri/target/release/app"
ASSETS="$SCRIPT_DIR/assets"

if [ "$(id -u)" != "0" ]; then
    echo "Blad: uruchom jako root (sudo ./install.sh)"
    exit 1
fi

echo "=== Instalacja Xiee OS v0.1.0 ==="
echo ""

BINS=(xiee-login xiee-desktop xiee-shell xiac xihh-key xfm winyy init xls xcat xecho)
for bin in "${BINS[@]}"; do
    if [ -f "$RELEASE/$bin" ]; then
        cp "$RELEASE/$bin" /usr/bin/"$bin"
        chmod +x /usr/bin/"$bin"
        echo "  [OK] $bin"
    else
        echo "  [SKIP] $bin - brak, uruchom: cargo build --release"
    fi
done

if [ -f "$XIARR" ]; then
    cp "$XIARR" /usr/bin/xiarr
    chmod +x /usr/bin/xiarr
    echo "  [OK] xiarr"
else
    echo "  [SKIP] xiarr - brak, uruchom: cd xiarr && cargo tauri build --no-bundle"
fi

echo ""
mkdir -p /usr/share/xiee

for asset in wallpaper.jpg logo.jpg; do
    if [ -f "$ASSETS/$asset" ]; then
        cp "$ASSETS/$asset" /usr/share/xiee/
        echo "  [OK] $asset"
    fi
done

echo ""
echo "=== Xiee OS zainstalowany! ==="
echo "Uruchom: xiee-login"