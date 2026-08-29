#!/usr/bin/env bash
# Measure local bitctx CLI latency and print a separate illustrative LLM comparison.

set -euo pipefail

BITCTX="${BITCTX:-bitctx}"
ITERATIONS="${ITERATIONS:-100}"
ILLUSTRATIVE_PROMPT_TOKENS="${ILLUSTRATIVE_PROMPT_TOKENS:-150}"
ILLUSTRATIVE_RESPONSE_TOKENS="${ILLUSTRATIVE_RESPONSE_TOKENS:-50}"
ILLUSTRATIVE_LLM_LATENCY_MS="${ILLUSTRATIVE_LLM_LATENCY_MS:-800}"

command -v "$BITCTX" >/dev/null 2>&1 || {
    echo "bitctx is not installed or is not executable: $BITCTX" >&2
    exit 1
}
command -v perl >/dev/null 2>&1 || {
    echo "perl is required for high-resolution timing" >&2
    exit 1
}
[[ "$ITERATIONS" =~ ^[1-9][0-9]*$ ]] || {
    echo "ITERATIONS must be a positive integer" >&2
    exit 1
}

DATA_DIR="$(mktemp -d)"
SCHEMA="$DATA_DIR/schema-source.json"
SESSION="perf-$$"
trap 'rm -rf "$DATA_DIR"' EXIT

cat >"$SCHEMA" <<'EOF'
{
  "version": 1,
  "bits": {
    "0": {"name": "user_authenticated", "desc": "User authenticated"},
    "1": {"name": "has_permission", "desc": "Has required permission"},
    "2": {"name": "resource_exists", "desc": "Target resource exists"},
    "3": {"name": "quota_ok", "desc": "Quota not exceeded"},
    "4": {"name": "rate_limit_ok", "desc": "Rate limit OK"},
    "5": {"name": "subscription_active", "desc": "Subscription active"},
    "6": {"name": "feature_enabled", "desc": "Feature flag enabled"},
    "7": {"name": "region_allowed", "desc": "Region allowed"},
    "8": {"name": "ip_whitelisted", "desc": "IP whitelisted"},
    "9": {"name": "device_trusted", "desc": "Device trusted"},
    "10": {"name": "mfa_verified", "desc": "MFA verified"},
    "11": {"name": "terms_accepted", "desc": "Terms accepted"},
    "12": {"name": "payment_valid", "desc": "Payment method valid"},
    "13": {"name": "age_verified", "desc": "Age verified"},
    "14": {"name": "kyc_passed", "desc": "KYC passed"},
    "15": {"name": "admin_override", "desc": "Admin override granted"}
  },
  "masks": {
    "required": {"bits": [0, 1, 3, 5, 6, 7, 10, 11, 13], "desc": "Full access requirements"}
  }
}
EOF

run_bitctx() {
    "$BITCTX" --data-dir "$DATA_DIR/state" "$@"
}

now_us() {
    perl -MTime::HiRes=time -e 'printf "%.0f\n", time * 1000000'
}

run_bitctx init --session "$SESSION" --schema "$SCHEMA" >/dev/null
run_bitctx set --session "$SESSION" \
    --bit "0,1,3,5,6,7,10,11,13" \
    --value "true,true,true,true,true,true,true,true,true" >/dev/null

for _ in $(seq 1 10); do
    run_bitctx eval --session "$SESSION" --mask required --format json >/dev/null
done

START_US="$(now_us)"
for _ in $(seq 1 "$ITERATIONS"); do
    run_bitctx eval --session "$SESSION" --mask required --format json >/dev/null
done
END_US="$(now_us)"

TOTAL_US=$((END_US - START_US))
PER_CALL_US=$((TOTAL_US / ITERATIONS))
if ((PER_CALL_US < 1)); then
    PER_CALL_US=1
fi

echo "Local bitctx measurement"
echo "  Platform: $(uname -s) $(uname -m)"
echo "  Iterations: $ITERATIONS"
echo "  Total: ${TOTAL_US} us"
echo "  Mean per CLI call: ${PER_CALL_US} us (includes process startup, locking, read, and JSON output)"

TOTAL_ESTIMATED_TOKENS=$((ILLUSTRATIVE_PROMPT_TOKENS + ILLUSTRATIVE_RESPONSE_TOKENS))
ILLUSTRATIVE_SPEEDUP=$((ILLUSTRATIVE_LLM_LATENCY_MS * 1000 / PER_CALL_US))

echo
echo "Illustrative LLM comparison (not measured and not guaranteed)"
echo "  Assumed prompt tokens: $ILLUSTRATIVE_PROMPT_TOKENS"
echo "  Assumed response tokens: $ILLUSTRATIVE_RESPONSE_TOKENS"
echo "  Assumed total tokens: $TOTAL_ESTIMATED_TOKENS"
echo "  Assumed API latency: ${ILLUSTRATIVE_LLM_LATENCY_MS} ms"
echo "  Implied latency ratio under those assumptions: ${ILLUSTRATIVE_SPEEDUP}x"
echo "  Provider, model, network, prompt, and tool behavior can change these values materially."
