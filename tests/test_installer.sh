#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALLER="$ROOT_DIR/install.sh"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

FAKE_BIN_DIR="$TEMP_DIR/bin"
INSTALL_DIR="$TEMP_DIR/install"
PAYLOAD="$TEMP_DIR/payload"
CHECKSUM="$TEMP_DIR/checksum"
URL_LOG="$TEMP_DIR/urls"
mkdir -p "$FAKE_BIN_DIR" "$INSTALL_DIR"

cat >"$PAYLOAD" <<'EOF'
#!/usr/bin/env bash
echo "bitctx 0.2.0"
EOF
chmod +x "$PAYLOAD"
shasum -a 256 "$PAYLOAD" >"$CHECKSUM"

cat >"$FAKE_BIN_DIR/uname" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
    -s) echo "Darwin" ;;
    -m) echo "arm64" ;;
    *) exit 1 ;;
esac
EOF

cat >"$FAKE_BIN_DIR/curl" <<'EOF'
#!/usr/bin/env bash
output=""
url=""
while [[ $# -gt 0 ]]; do
    case "$1" in
        -o)
            output="$2"
            shift 2
            ;;
        http*)
            url="$1"
            shift
            ;;
        *)
            shift
            ;;
    esac
done
printf '%s\n' "$url" >>"$URL_LOG"
if [[ "$url" == *.sha256 ]]; then
    [[ "${FAKE_CHECKSUM_MISSING:-0}" != "1" ]] || exit 22
    if [[ "${FAKE_CHECKSUM_INVALID:-0}" == "1" ]]; then
        printf '%064d  asset\n' 0 >"$output"
    else
        cp "$CHECKSUM" "$output"
    fi
else
    cp "$PAYLOAD" "$output"
fi
EOF

chmod +x "$FAKE_BIN_DIR/uname" "$FAKE_BIN_DIR/curl"
export CHECKSUM PAYLOAD URL_LOG

TEST_PATH="$FAKE_BIN_DIR:/usr/bin:/bin:/usr/sbin:/sbin"
PATH="$TEST_PATH" INSTALL_DIR="$INSTALL_DIR" "$INSTALLER" >/dev/null
grep -q 'releases/latest/download/bitctx-aarch64-macos$' "$URL_LOG"
grep -q 'releases/latest/download/bitctx-aarch64-macos.sha256$' "$URL_LOG"
[[ "$("$INSTALL_DIR/bitctx" --version)" == "bitctx 0.2.0" ]]

rm -f "$INSTALL_DIR/bitctx" "$URL_LOG"
PATH="$TEST_PATH" INSTALL_DIR="$INSTALL_DIR" VERSION=v0.2.0 "$INSTALLER" >/dev/null
grep -q 'releases/download/v0.2.0/bitctx-aarch64-macos$' "$URL_LOG"

rm -f "$INSTALL_DIR/bitctx"
if PATH="$TEST_PATH" INSTALL_DIR="$INSTALL_DIR" FAKE_CHECKSUM_MISSING=1 "$INSTALLER" >/dev/null 2>&1; then
    echo "Installer accepted a missing checksum" >&2
    exit 1
fi
[[ ! -e "$INSTALL_DIR/bitctx" ]]

if PATH="$TEST_PATH" INSTALL_DIR="$INSTALL_DIR" FAKE_CHECKSUM_INVALID=1 "$INSTALLER" >/dev/null 2>&1; then
    echo "Installer accepted an incorrect checksum" >&2
    exit 1
fi
[[ ! -e "$INSTALL_DIR/bitctx" ]]

echo "Installer tests passed"
