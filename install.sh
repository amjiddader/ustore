#!/bin/bash
set -euo pipefail

REPO="amjiddader/ustore"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

echo ""
echo "  🏪 uStore Installer"
echo "  ─────────────────────"
echo ""

# Check architecture
ARCH=$(dpkg --print-architecture)
if [ "$ARCH" != "amd64" ]; then
    echo "  ✗ Unsupported architecture: $ARCH (only amd64 supported)"
    exit 1
fi

# Find latest .deb from GitHub releases
echo "  → Finding latest release..."
DEB_URL=$(curl -sL "https://api.github.com/repos/${REPO}/releases" \
    | grep -oP '"browser_download_url"\s*:\s*"\K[^"]*\.deb' \
    | head -1)

if [ -z "$DEB_URL" ]; then
    echo "  ✗ Could not find .deb in releases."
    exit 1
fi

DEB_FILE="$TMP_DIR/ustore.deb"
echo "  → Downloading: $(basename "$DEB_URL")"

# Use aria2c if available, otherwise curl
if command -v aria2c &>/dev/null; then
    aria2c -q --max-connection-per-server=16 --split=16 \
        --dir="$TMP_DIR" --out="ustore.deb" "$DEB_URL"
else
    curl -sL -o "$DEB_FILE" "$DEB_URL"
fi

echo "  → Installing ustore..."
sudo dpkg -i "$DEB_FILE" || true
sudo apt-get install -f -y -qq

echo ""
echo "  ✓ uStore installed successfully!"
echo ""
echo "  Get started:"
echo "    ustore update        Fetch package registry"
echo "    ustore search <app>  Search for apps"
echo "    ustore install <app> Install an app"
echo "    ustore list          List installed apps"
echo ""