#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WRAPPER="$ROOT_DIR/skills/bit-context/bitctx_skill.sh"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

FAKE_BIN="$TEMP_DIR/bitctx"
FAKE_LOG="$TEMP_DIR/args.log"
cat >"$FAKE_BIN" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$@" >"$FAKE_LOG"
EOF
chmod +x "$FAKE_BIN"
export FAKE_LOG

assert_args() {
    local expected="$1"
    local actual
    actual="$(cat "$FAKE_LOG")"
    if [[ "$actual" != "$expected" ]]; then
        echo "Argument mismatch" >&2
        echo "Expected:" >&2
        printf '%s\n' "$expected" >&2
        echo "Actual:" >&2
        printf '%s\n' "$actual" >&2
        exit 1
    fi
}

env -u BITCTX_SESSION BITCTX_BIN=/missing/bitctx "$WRAPPER" help >/dev/null

if BITCTX_BIN="$FAKE_BIN" env -u BITCTX_SESSION "$WRAPPER" dump >/dev/null 2>"$TEMP_DIR/error"; then
    echo "Wrapper accepted a missing BITCTX_SESSION" >&2
    exit 1
fi
grep -q "BITCTX_SESSION is required" "$TEMP_DIR/error"

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" init schema.json --force
assert_args $'init\n--session\nsession-1\n--schema\nschema.json\n--force'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" init schema.json
assert_args $'init\n--session\nsession-1\n--schema\nschema.json'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" set auth true
assert_args $'set\n--session\nsession-1\n--bit\nauth\n--value\ntrue'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" set-multi 'auth,permission' 'true,false'
assert_args $'set\n--session\nsession-1\n--bit\nauth,permission\n--value\ntrue,false'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" eval required text
assert_args $'eval\n--session\nsession-1\n--mask\nrequired\n--format\ntext'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" eval required text missing
assert_args $'eval\n--session\nsession-1\n--mask\nrequired\n--format\ntext\n--show\nmissing'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" resume
assert_args $'resume\n--session\nsession-1\n--format\njson'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" resume required
assert_args $'resume\n--session\nsession-1\n--mask\nrequired\n--format\njson'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" resume required text
assert_args $'resume\n--session\nsession-1\n--mask\nrequired\n--format\ntext'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" explain required en
assert_args $'explain\n--session\nsession-1\n--mask\nrequired\n--lang\nen'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" dump json
assert_args $'dump\n--session\nsession-1\n--format\njson'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" reset --force
assert_args $'reset\n--session\nsession-1\n--force'

BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" reset
assert_args $'reset\n--session\nsession-1'

if BITCTX_BIN="$FAKE_BIN" BITCTX_SESSION=session-1 "$WRAPPER" unknown >/dev/null 2>"$TEMP_DIR/error"; then
    echo "Wrapper accepted an unknown command" >&2
    exit 1
fi
grep -q "Unknown command: unknown" "$TEMP_DIR/error"

echo "Wrapper tests passed"
