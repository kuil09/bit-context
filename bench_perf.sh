#!/usr/bin/env bash
# Performance comparison: bitctx vs LLM prompt approach

set -euo pipefail

BITCTX="${BITCTX:-/Users/gun9/Developer/bit-mania/bitctx-cli/target/release/bitctx}"
SESSION="perf-test-$(date +%s)"
SCHEMA="/tmp/perf_schema.json"
ITERATIONS="${ITERATIONS:-100}"

cat > "$SCHEMA" <<'EOF'
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
    "required": {"bits": [0,1,3,5,6,7,10,11,13], "desc": "Full access requirements"},
    "read_only": {"bits": [0,2,7], "desc": "Read access requirements"},
    "admin": {"bits": [0,1,15], "desc": "Admin access requirements"}
  }
}
EOF

echo "=== bitctx Performance Benchmark ==="
echo "Iterations: $ITERATIONS"
echo "Conditions: 16 bits, 3 masks (largest: 9 conditions)"
echo ""

# Setup
$BITCTX init --session "$SESSION" --schema "$SCHEMA" >/dev/null
$BITCTX set --session "$SESSION" --bit "0,1,2,3,5,6,7,10,11,13" --value "true,true,true,true,true,true,true,true,true,true" >/dev/null

# Warmup
for i in {1..10}; do
    $BITCTX eval --session "$SESSION" --mask required --format json >/dev/null
done

# Benchmark bitctx eval using /usr/bin/time or perl for microseconds
echo "--- bitctx eval (bitwise) ---"
start_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time*1000')
for i in $(seq 1 $ITERATIONS); do
    $BITCTX eval --session "$SESSION" --mask required --format json >/dev/null
done
end_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time*1000')
bitctx_ms=$((end_ms - start_ms))
bitctx_per_call=$((bitctx_ms * 1000 / ITERATIONS))

echo "Total: ${bitctx_ms}ms for $ITERATIONS calls"
echo "Per call: ${bitctx_per_call}μs (includes process spawn)"

# Measure cold start separately
echo ""
echo "--- Cold start (first call) ---"
$BITCTX reset --session "$SESSION" --force >/dev/null
$BITCTX init --session "$SESSION" --schema "$SCHEMA" >/dev/null
$BITCTX set --session "$SESSION" --bit "0,1,2,3,5,6,7,10,11,13" --value "true,true,true,true,true,true,true,true,true,true" >/dev/null
start_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time*1000')
$BITCTX eval --session "$SESSION" --mask required --format json >/dev/null
end_ms=$(perl -MTime::HiRes=time -e 'printf "%.0f\n", time*1000')
cold_ms=$((end_ms - start_ms))
echo "Cold start: ${cold_ms}ms"

# Token estimation for LLM approach
echo ""
echo "=== LLM Prompt Approach (Estimated) ==="

# Typical prompt for 9 conditions
prompt_tokens=150  # System + user prompt with condition descriptions
response_tokens=50  # Expected response
total_tokens=$((prompt_tokens + response_tokens))

# Typical LLM API latency (varies by provider)
llm_latency_ms=800  # Conservative average for small prompt

echo "Prompt tokens (9 conditions listed): ~${prompt_tokens}"
echo "Response tokens: ~${response_tokens}"
echo "Total tokens per eval: ~${total_tokens}"
echo "Estimated API latency: ~${llm_latency_ms}ms"

# Comparison
echo ""
echo "=== Comparison ==="
speedup=$((llm_latency_ms * 1000 / bitctx_per_call))
token_savings=$((total_tokens * ITERATIONS))

echo "bitctx eval:     ${bitctx_per_call}μs (${bitctx_ms}ms total)"
echo "LLM API (est):   ${llm_latency_ms}ms per call (${llm_latency_ms}000μs)"
echo "Speedup:         ~${speedup}x faster"
echo ""
echo "Token savings for $ITERATIONS evals: ~${token_savings} tokens"
echo "At \$0.0015/1K tokens (GPT-4o-mini): ~\$${token_savings/1000000}"

# Cleanup
$BITCTX reset --session "$SESSION" --force >/dev/null
rm "$SCHEMA"

echo ""
echo "Note: LLM latency varies (200ms-3s). bitctx is local, deterministic, no network."