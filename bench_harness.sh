#!/usr/bin/env bash
# Real-world comparison: Harness workflow WITH vs WITHOUT bitctx

set -euo pipefail

echo "=========================================="
echo "Harness Task: 'Deploy to production'"
echo "=========================================="
echo ""

cat <<'EOF'
┌─────────────────────────────────────────────────────────────────────┐
│  SCENARIO: Deploy to production requires 12 conditions             │
│  1. User authenticated          7. Feature flag enabled            │
│  2. Has admin permission        8. Region allowed                  │
│  3. Resource exists             9. IP whitelisted                  │
│  4. Quota not exceeded          10. Device trusted                 │
│  5. Rate limit OK               11. MFA verified                   │
│  6. Subscription active         12. Terms accepted                 │
└─────────────────────────────────────────────────────────────────────┘
EOF

echo ""
echo "WITHOUT bitctx: LLM does ALL the reasoning"
echo ""

cat <<'PROMPT'
=== FULL PROMPT SENT TO LLM ===

You are a deployment authorization system. Check ALL conditions below
and respond with "APPROVED" or "REJECTED: <reason>".

CONDITIONS TO VERIFY (all must be true):
1. user_authenticated: User has valid session (check auth service)
2. has_admin_permission: User has admin role (check RBAC)
3. resource_exists: Target service exists in registry
4. quota_ok: Deployment quota not exceeded (check quota service)
5. rate_limit_ok: Not rate limited (check rate limiter)
6. subscription_active: Org subscription is active (check billing)
7. feature_enabled: "deploy-prod" feature flag is ON (check flags)
8. region_allowed: Deployment region is permitted (check policy)
9. ip_whitelisted: Request IP is whitelisted (check network policy)
10. device_trusted: Device is registered and trusted (check device registry)
11. mfa_verified: User completed MFA in last 24h (check auth logs)
12. terms_accepted: User accepted latest deployment terms (check DB)

RESPONSE FORMAT:
- If ALL true: "APPROVED"
- If ANY false: "REJECTED: <comma-separated list of failed conditions>"

Current context:
- user_id: "user-12345"
- service: "payment-api"
- region: "us-east-1"
- ip: "10.0.0.42"
- device_id: "dev-abc123"

[LLM MUST call 12+ tools/APIs or hallucinate...]
PROMPT

echo ""
echo "--- Metrics ---"
echo "Prompt tokens: ~450"
echo "Expected response tokens: ~30"
echo "Total per request: ~480 tokens"
echo "LLM must: Read 12 conditions, call 12+ tools, reason about all"
echo "Latency: 2-10 seconds (tool calls + reasoning)"
echo "Cost (GPT-4o): ~\$0.0012/request"

echo ""
echo "WITH bitctx: Harness checks bits, LLM gets DECISION only"
echo ""

cat <<'HARNESS'
=== HARNESS CODE (Python) ===

# 1. Harness evaluates conditions DETERMINISTICALLY (no LLM)
results = {
    "user_authenticated": check_auth(user_id),
    "has_admin_permission": check_rbac(user_id, "admin"),
    "resource_exists": check_registry(service),
    "quota_ok": check_quota(org_id),
    "rate_limit_ok": check_ratelimit(ip),
    "subscription_active": check_billing(org_id),
    "feature_enabled": check_flag("deploy-prod"),
    "region_allowed": check_policy(region),
    "ip_whitelisted": check_network(ip),
    "device_trusted": check_device(device_id),
    "mfa_verified": check_mfa(user_id),
    "terms_accepted": check_terms(user_id),
}

# 2. Set bits in bitctx (instant, local)
bitctx.set_multi(list(results.keys()), list(results.values()))

# 3. Single bitwise eval
eval_result = bitctx.eval("deploy_prod_mask")

# 4. Minimal prompt to LLM
if eval_result["pass"]:
    prompt = "Deployment APPROVED. Proceed with production deploy."
else:
    failed = bitctx.explain("deploy_prod_mask")
    prompt = f"Deployment REJECTED: {failed}. Inform user."

# 5. LLM only generates user-facing message
llm_response = llm(prompt)
HARNESS

echo ""
echo "--- Metrics ---"
echo "Prompt tokens: ~40 (just the decision + brief reason)"
echo "Response tokens: ~50"
echo "Total per request: ~90 tokens"
echo "LLM does: Generate human message ONLY (no reasoning, no tools)"
echo "Latency: 200-500ms (single LLM call, no tools)"
echo "Cost (GPT-4o): ~\$0.0002/request"

echo ""
echo "COMPARISON SUMMARY"
echo ""

printf "%-30s | %-15s | %-15s\n" "Metric" "Without bitctx" "With bitctx"
printf "%-30s-+-%-15s-+-%-15s\n" "------------------------------" "---------------" "---------------"
printf "%-30s | %-15s | %-15s\n" "Prompt tokens" "~450" "~40"
printf "%-30s | %-15s | %-15s\n" "Response tokens" "~30" "~50"
printf "%-30s | %-15s | %-15s\n" "Total tokens/request" "~480" "~90"
printf "%-30s | %-15s | %-15s\n" "Token reduction" "baseline" "**81% less**"
printf "%-30s | %-15s | %-15s\n" "LLM tool calls" "12+" "**0**"
printf "%-30s | %-15s | %-15s\n" "LLM reasoning depth" "Complex (12 cond)" "None"
printf "%-30s | %-15s | %-15s\n" "Latency" "2-10 sec" "0.2-0.5 sec"
printf "%-30s | %-15s | %-15s\n" "Cost/request (GPT-4o)" "~\$0.0012" "~\$0.0002"
printf "%-30s | %-15s | %-15s\n" "Cost/1000 deploys" "~\$1.20" "~\$0.20"
printf "%-30s | %-15s | %-15s\n" "Deterministic logic" "No (LLM may err)" "**Yes**"
printf "%-30s | %-15s | %-15s\n" "Audit trail" "Prompt only" "Bits + decision"
printf "%-30s | %-15s | %-15s\n" "Adding new condition" "Rewrite prompt" "Add bit to schema"

echo ""
echo "KEY INSIGHT: The harness ALREADY has code to check each condition."
echo "bitctx just COLLECTS the boolean results and makes the FINAL"
echo "decision via bitwise AND. The LLM never sees the conditions."