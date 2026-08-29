#!/usr/bin/env bash
# Download and install a bitctx release after mandatory SHA-256 verification.

set -euo pipefail

REPOSITORY="kuil09/bit-context"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"

command -v curl >/dev/null 2>&1 || {
    echo "curl is required" >&2
    exit 1
}

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS-$ARCH" in
    linux-x86_64) ASSET="bitctx-x86_64-linux" ;;
    linux-aarch64 | linux-arm64) ASSET="bitctx-aarch64-linux" ;;
    darwin-x86_64) ASSET="bitctx-x86_64-macos" ;;
    darwin-arm64 | darwin-aarch64) ASSET="bitctx-aarch64-macos" ;;
    *)
        echo "Unsupported platform: $OS-$ARCH" >&2
        echo "bitctx v0.2 supports Linux and macOS on x86-64 and ARM64" >&2
        exit 1
        ;;
esac

if [[ "$VERSION" == "latest" ]]; then
    BASE_URL="https://github.com/$REPOSITORY/releases/latest/download"
else
    BASE_URL="https://github.com/$REPOSITORY/releases/download/$VERSION"
fi

BINARY_URL="$BASE_URL/$ASSET"
CHECKSUM_URL="$BASE_URL/$ASSET.sha256"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

echo "Downloading $BINARY_URL"
curl -fsSL "$BINARY_URL" -o "$TEMP_DIR/$ASSET"
curl -fsSL "$CHECKSUM_URL" -o "$TEMP_DIR/$ASSET.sha256"

EXPECTED="$(awk 'NR == 1 { print $1 }' "$TEMP_DIR/$ASSET.sha256")"
if [[ ! "$EXPECTED" =~ ^[[:xdigit:]]{64}$ ]]; then
    echo "Invalid checksum file: $CHECKSUM_URL" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "$TEMP_DIR/$ASSET" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "$TEMP_DIR/$ASSET" | awk '{ print $1 }')"
else
    echo "A SHA-256 tool is required (sha256sum or shasum)" >&2
    exit 1
fi

EXPECTED_NORMALIZED="$(printf '%s' "$EXPECTED" | tr '[:upper:]' '[:lower:]')"
ACTUAL_NORMALIZED="$(printf '%s' "$ACTUAL" | tr '[:upper:]' '[:lower:]')"
if [[ "$EXPECTED_NORMALIZED" != "$ACTUAL_NORMALIZED" ]]; then
    echo "Checksum mismatch for $ASSET" >&2
    echo "Expected: $EXPECTED" >&2
    echo "Actual:   $ACTUAL" >&2
    exit 1
fi

echo "Checksum verified: $ACTUAL"
if [[ -d "$INSTALL_DIR" && -w "$INSTALL_DIR" ]]; then
    install -m 0755 "$TEMP_DIR/$ASSET" "$INSTALL_DIR/bitctx"
else
    command -v sudo >/dev/null 2>&1 || {
        echo "Cannot write to $INSTALL_DIR and sudo is unavailable" >&2
        exit 1
    }
    sudo install -d -m 0755 "$INSTALL_DIR"
    sudo install -m 0755 "$TEMP_DIR/$ASSET" "$INSTALL_DIR/bitctx"
fi

echo "bitctx installed to $INSTALL_DIR/bitctx"
"$INSTALL_DIR/bitctx" --version
