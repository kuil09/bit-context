#!/usr/bin/env bash
# Exercise the end-to-end state flow against one release binary.

set -euo pipefail

BITCTX="${1:?Usage: smoke_release.sh /path/to/bitctx}"
[[ -x "$BITCTX" ]] || {
    echo "Binary is not executable: $BITCTX" >&2
    exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXPECTED_VERSION="$(awk -F '"' '$1 == "version = " {print $2; exit}' "$ROOT_DIR/bitctx-cli/Cargo.toml")"
[[ -n "$EXPECTED_VERSION" ]] || {
    echo "Could not read the expected version from Cargo.toml" >&2
    exit 1
}

TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT
DATA_DIR="$TEMP_DIR/data"
SCHEMA="$TEMP_DIR/schema.json"
SESSION="release-smoke"

cat >"$SCHEMA" <<'EOF'
{
  "version": 1,
  "bits": {
    "0": {"name": "auth", "desc": "Authentication verified"},
    "1": {"name": "permission", "desc": "Permission verified"},
    "2": {"name": "quota", "desc": "Quota verified"}
  },
  "masks": {
    "required": {"bits": [2, 0, 1], "desc": "Release smoke gate"}
  }
}
EOF

[[ "$("$BITCTX" --version)" == "bitctx $EXPECTED_VERSION" ]]
"$BITCTX" --data-dir "$DATA_DIR" init --session "$SESSION" --schema "$SCHEMA" >/dev/null

INITIAL="$("$BITCTX" --data-dir "$DATA_DIR" eval --session "$SESSION" --mask required --format json)"
python3 -c 'import json,sys; value=json.loads(sys.argv[1]); assert value["pass"] is False; assert value["missing"] == [2,0,1]' "$INITIAL"

TEXT_OUTPUT="$("$BITCTX" --data-dir "$DATA_DIR" eval --session "$SESSION" --mask required --format text --show missing)"
grep -q '^RESULT: X$' <<<"$TEXT_OUTPUT"
grep -q '^  X bit 2: quota (Quota verified)$' <<<"$TEXT_OUTPUT"

"$BITCTX" --data-dir "$DATA_DIR" set --session "$SESSION" \
    --bit quota,auth,permission --value true,true,true >/dev/null

FINAL="$("$BITCTX" --data-dir "$DATA_DIR" eval --session "$SESSION" --mask required --format json)"
python3 -c 'import json,sys; value=json.loads(sys.argv[1]); assert value["pass"] is True; assert value["missing_conditions"] == []' "$FINAL"

"$BITCTX" --data-dir "$DATA_DIR" explain --session "$SESSION" --mask required --lang en \
    | grep -q 'All conditions satisfied'
"$BITCTX" --data-dir "$DATA_DIR" dump --session "$SESSION" --format json \
    | python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["bits"] == 7'
"$BITCTX" --data-dir "$DATA_DIR" reset --session "$SESSION" --force >/dev/null
[[ ! -e "$DATA_DIR/$SESSION" ]]

echo "Release smoke test passed"
