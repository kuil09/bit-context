#!/usr/bin/env bash
# bitctx installer - downloads and installs the latest release

set -euo pipefail

REPO="kuil09/bit-context"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"

# Detect platform
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
    linux-x86_64)   ASSET="bitctx-x86_64-linux" ;;
    linux-aarch64|linux-arm64) ASSET="bitctx-aarch64-linux" ;;
    darwin-x86_64)  ASSET="bitctx-x86_64-macos" ;;
    darwin-arm64|darwin-aarch64) ASSET="bitctx-aarch64-macos" ;;
    *) echo "Unsupported platform: $OS-$ARCH" >&2; exit 1 ;;
esac

# Determine download URL
if [ "$VERSION" = "latest" ]; then
    API_URL="https://api.github.com/repos/$REPO/releases/latest"
    DOWNLOAD_URL=$(curl -sSL "$API_URL" | grep -o "\"browser_download_url\": \"[^\"]*$ASSET[^\"]*\"" | head -1 | cut -d'"' -f4)
    if [ -z "$DOWNLOAD_URL" ]; then
        echo "Could not find asset for $ASSET" >&2
        exit 1
    fi
else
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
fi

echo "Installing bitctx from $DOWNLOAD_URL"

# Download and install
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/bitctx"
chmod +x "$TMP_DIR/bitctx"

# Install (with sudo if needed)
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/bitctx" "$INSTALL_DIR/bitctx"
else
    sudo mv "$TMP_DIR/bitctx" "$INSTALL_DIR/bitctx"
fi

echo "bitctx installed to $INSTALL_DIR/bitctx"
bitctx --version