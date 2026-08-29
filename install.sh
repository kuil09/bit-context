#!/usr/bin/env bash
# bitctx installer - downloads and installs the latest release with checksum verification

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
    CHECKSUM_URL=$(curl -sSL "$API_URL" | grep -o "\"browser_download_url\": \"[^\"]*$ASSET[^\"]*\.sha256\"" | head -1 | cut -d'"' -f4)
    if [ -z "$DOWNLOAD_URL" ]; then
        echo "Could not find asset for $ASSET" >&2
        exit 1
    fi
else
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
    CHECKSUM_URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET.sha256"
fi

echo "Installing bitctx from $DOWNLOAD_URL"

# Download binary and checksum
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -sSL "$DOWNLOAD_URL" -o "$TMP_DIR/bitctx"
chmod +x "$TMP_DIR/bitctx"

# Verify checksum if available
if curl -sSLf "$CHECKSUM_URL" -o "$TMP_DIR/bitctx.sha256" 2>/dev/null; then
    echo "Verifying checksum..."
    EXPECTED=$(cat "$TMP_DIR/bitctx.sha256" | awk '{print $1}')
    if command -v sha256sum >/dev/null; then
        ACTUAL=$(sha256sum "$TMP_DIR/bitctx" | awk '{print $1}')
    elif command -v shasum >/dev/null; then
        ACTUAL=$(shasum -a 256 "$TMP_DIR/bitctx" | awk '{print $1}')
    elif command -v certutil >/dev/null; then
        ACTUAL=$(certutil -hashfile "$TMP_DIR/bitctx" SHA256 | tail -1 | tr -d ' \r\n')
    else
        echo "Warning: No checksum tool available, skipping verification" >&2
        ACTUAL="$EXPECTED"
    fi
    if [ "$EXPECTED" != "$ACTUAL" ]; then
        echo "Checksum mismatch!" >&2
        echo "Expected: $EXPECTED" >&2
        echo "Actual:   $ACTUAL" >&2
        exit 1
    fi
    echo "Checksum verified: $EXPECTED"
else
    echo "Warning: No checksum file found, skipping verification" >&2
fi

# Install (with sudo if needed)
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_DIR/bitctx" "$INSTALL_DIR/bitctx"
else
    sudo mv "$TMP_DIR/bitctx" "$INSTALL_DIR/bitctx"
fi

echo "bitctx installed to $INSTALL_DIR/bitctx"
bitctx --version