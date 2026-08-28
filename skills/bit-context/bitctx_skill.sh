#!/usr/bin/env bash
# bit-context skill wrapper for harnees
# Usage: bitctx_skill.sh <command> [args...]

set -euo pipefail

BITCTX_BIN="${BITCTX_BIN:-bitctx}"
SESSION="${BITCTX_SESSION:-default}"

usage() {
    cat <<EOF
bit-context skill - Bit-memory context store for AI harness

Environment:
  BITCTX_BIN       Path to bitctx binary (default: bitctx)
  BITCTX_SESSION   Session ID (default: default)

Commands:
  init <schema.json>           Initialize session with schema
  set <bit> <value>            Set bit (name or index) to true/false
  set-multi <bits_csv> <vals_csv>  Set multiple bits at once
  eval <mask> [json|text]      Evaluate mask, output format
  explain <mask> [ko|en]       Explain failure in natural language
  dump [json|text]             Dump full session state
  reset [--force]              Delete session

Examples:
  bitctx_skill.sh init schema.json
  bitctx_skill.sh set user_authenticated true
  bitctx_skill.sh set-multi "user_authenticated,has_permission" "true,true"
  bitctx_skill.sh eval required json
  bitctx_skill.sh explain required ko
  bitctx_skill.sh dump text
EOF
}

cmd_init() {
    local schema="${1:-}"
    if [[ -z "$schema" ]]; then
        echo "Usage: bitctx_skill.sh init <schema.json>" >&2
        exit 1
    fi
    "$BITCTX_BIN" init --session "$SESSION" --schema "$schema"
}

cmd_set() {
    local bit="${1:-}"
    local value="${2:-}"
    if [[ -z "$bit" || -z "$value" ]]; then
        echo "Usage: bitctx_skill.sh set <bit> <value>" >&2
        exit 1
    fi
    "$BITCTX_BIN" set --session "$SESSION" --bit "$bit" --value "$value"
}

cmd_set_multi() {
    local bits="${1:-}"
    local values="${2:-}"
    if [[ -z "$bits" || -z "$values" ]]; then
        echo "Usage: bitctx_skill.sh set-multi <bits_csv> <values_csv>" >&2
        exit 1
    fi
    "$BITCTX_BIN" set --session "$SESSION" --bit "$bits" --value "$values"
}

cmd_eval() {
    local mask="${1:-}"
    local format="${2:-json}"
    if [[ -z "$mask" ]]; then
        echo "Usage: bitctx_skill.sh eval <mask> [json|text]" >&2
        exit 1
    fi
    "$BITCTX_BIN" eval --session "$SESSION" --mask "$mask" --format "$format"
}

cmd_explain() {
    local mask="${1:-}"
    local lang="${2:-ko}"
    if [[ -z "$mask" ]]; then
        echo "Usage: bitctx_skill.sh explain <mask> [ko|en]" >&2
        exit 1
    fi
    "$BITCTX_BIN" explain --session "$SESSION" --mask "$mask" --lang "$lang"
}

cmd_dump() {
    local format="${1:-text}"
    "$BITCTX_BIN" dump --session "$SESSION" --format "$format"
}

cmd_reset() {
    local force=""
    if [[ "${1:-}" == "--force" ]]; then
        force="--force"
    fi
    "$BITCTX_BIN" reset --session "$SESSION" $force
}

main() {
    local cmd="${1:-}"
    shift || true

    case "$cmd" in
        init)       cmd_init "$@" ;;
        set)        cmd_set "$@" ;;
        set-multi)  cmd_set_multi "$@" ;;
        eval)       cmd_eval "$@" ;;
        explain)    cmd_explain "$@" ;;
        dump)       cmd_dump "$@" ;;
        reset)      cmd_reset "$@" ;;
        ""|help|--help|-h) usage ;;
        *) echo "Unknown command: $cmd" >&2; usage; exit 1 ;;
    esac
}

main "$@"