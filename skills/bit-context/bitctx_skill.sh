#!/usr/bin/env bash
# Compatibility wrapper for the bit-context skill.

set -euo pipefail

BITCTX_BIN="${BITCTX_BIN:-bitctx}"

usage() {
    cat <<'EOF'
bit-context skill - deterministic boolean condition storage and evaluation

Environment:
  BITCTX_BIN       Path to the bitctx binary (default: bitctx)
  BITCTX_SESSION   Required session ID for every command except help
  BITCTX_DATA_DIR  Optional data directory (default: ~/.bitctx)

Commands:
  init <schema.json> [--force]       Initialize or reinitialize a session
  set <bit> <value>                  Set one bit
  set-multi <bits_csv> <values_csv>  Set multiple bits atomically
  eval <mask> [json|text] [show]     Evaluate a mask; show is all|satisfied|missing
  resume [mask] [json|text]          Resume from missing conditions; mask may be inferred
  explain <mask> [ko|en]             Explain missing conditions
  dump [json|text]                   Dump the complete session state
  reset [--force]                    Delete the session
  help                               Show this help
EOF
}

fail_usage() {
    echo "$1" >&2
    usage >&2
    exit 2
}

require_runtime() {
    if ! command -v "$BITCTX_BIN" >/dev/null 2>&1; then
        echo "bitctx is not installed or is not executable: $BITCTX_BIN" >&2
        exit 127
    fi
    if [[ -z "${BITCTX_SESSION:-}" ]]; then
        echo "BITCTX_SESSION is required; no default session is used" >&2
        exit 2
    fi
}

cmd_init() {
    [[ $# -ge 1 && $# -le 2 ]] || fail_usage "Usage: bitctx_skill.sh init <schema.json> [--force]"
    local schema="$1"
    if [[ $# -eq 2 ]]; then
        [[ "$2" == "--force" ]] || fail_usage "init accepts only --force after the schema path"
        "$BITCTX_BIN" init --session "$BITCTX_SESSION" --schema "$schema" --force
    else
        "$BITCTX_BIN" init --session "$BITCTX_SESSION" --schema "$schema"
    fi
}

cmd_set() {
    [[ $# -eq 2 ]] || fail_usage "Usage: bitctx_skill.sh set <bit> <value>"
    "$BITCTX_BIN" set --session "$BITCTX_SESSION" --bit "$1" --value "$2"
}

cmd_set_multi() {
    [[ $# -eq 2 ]] || fail_usage "Usage: bitctx_skill.sh set-multi <bits_csv> <values_csv>"
    "$BITCTX_BIN" set --session "$BITCTX_SESSION" --bit "$1" --value "$2"
}

cmd_eval() {
    [[ $# -ge 1 && $# -le 3 ]] || fail_usage "Usage: bitctx_skill.sh eval <mask> [json|text] [all|satisfied|missing]"
    if [[ $# -eq 3 ]]; then
        "$BITCTX_BIN" eval --session "$BITCTX_SESSION" --mask "$1" --format "$2" --show "$3"
    else
        "$BITCTX_BIN" eval --session "$BITCTX_SESSION" --mask "$1" --format "${2:-json}"
    fi
}

cmd_resume() {
    [[ $# -le 2 ]] || fail_usage "Usage: bitctx_skill.sh resume [mask] [json|text]"
    if [[ $# -eq 0 ]]; then
        "$BITCTX_BIN" resume --session "$BITCTX_SESSION" --format json
    elif [[ $# -eq 1 ]]; then
        "$BITCTX_BIN" resume --session "$BITCTX_SESSION" --mask "$1" --format json
    else
        [[ "$2" == "json" || "$2" == "text" ]] || fail_usage "resume format must be json or text"
        "$BITCTX_BIN" resume --session "$BITCTX_SESSION" --mask "$1" --format "$2"
    fi
}

cmd_explain() {
    [[ $# -ge 1 && $# -le 2 ]] || fail_usage "Usage: bitctx_skill.sh explain <mask> [ko|en]"
    "$BITCTX_BIN" explain --session "$BITCTX_SESSION" --mask "$1" --lang "${2:-ko}"
}

cmd_dump() {
    [[ $# -le 1 ]] || fail_usage "Usage: bitctx_skill.sh dump [json|text]"
    "$BITCTX_BIN" dump --session "$BITCTX_SESSION" --format "${1:-text}"
}

cmd_reset() {
    [[ $# -le 1 ]] || fail_usage "Usage: bitctx_skill.sh reset [--force]"
    if [[ $# -eq 1 ]]; then
        [[ "$1" == "--force" ]] || fail_usage "reset accepts only --force"
        "$BITCTX_BIN" reset --session "$BITCTX_SESSION" --force
    else
        "$BITCTX_BIN" reset --session "$BITCTX_SESSION"
    fi
}

main() {
    local command_name="${1:-help}"
    if [[ $# -gt 0 ]]; then
        shift
    fi

    case "$command_name" in
        help | --help | -h)
            usage
            return
            ;;
        init | set | set-multi | eval | resume | explain | dump | reset)
            require_runtime
            ;;
        *)
            fail_usage "Unknown command: $command_name"
            ;;
    esac

    case "$command_name" in
        init) cmd_init "$@" ;;
        set) cmd_set "$@" ;;
        set-multi) cmd_set_multi "$@" ;;
        eval) cmd_eval "$@" ;;
        resume) cmd_resume "$@" ;;
        explain) cmd_explain "$@" ;;
        dump) cmd_dump "$@" ;;
        reset) cmd_reset "$@" ;;
    esac
}

main "$@"
