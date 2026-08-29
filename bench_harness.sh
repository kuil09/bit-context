#!/usr/bin/env bash
# Print an illustrative harness design comparison. This script does not benchmark an LLM.

set -euo pipefail

cat <<'EOF'
Illustrative harness comparison
===============================

This is a design example, not a local measurement, provider benchmark, price
quote, token guarantee, or latency guarantee.

Scenario
--------
A production deployment gate has 12 boolean conditions. The harness already
has authoritative code or tools for checking authentication, permissions,
resource existence, quota, rate limit, subscription, feature flags, region,
network policy, device trust, MFA, and accepted terms.

Natural-language gate design
----------------------------
The harness sends every condition and result to an LLM and asks it to perform
the final AND decision. Token usage and model latency scale with the prompt,
tool use, provider, model, and network.

bitctx gate design
------------------
The harness runs the same real checks, stores only their observed boolean
results, and evaluates a named mask locally:

    bitctx set --session deploy-123 \
      --bit user_authenticated,has_permission,quota_ok \
      --value true,true,false
    bitctx eval --session deploy-123 --mask deploy_prod --format json

The LLM, if one is needed, receives only a compact decision or the ordered
missing_conditions needed to write a user-facing explanation.

Illustrative assumptions
------------------------
The following figures merely make the example concrete:

    Natural-language gate prompt:  about 450 input tokens
    Compact result prompt:          about 40 input tokens
    Model response:                 about 30-50 output tokens
    Remote model latency:           about 0.2-10 seconds

Do not treat these figures as measured results. Run bench_perf.sh to measure
only the local bitctx CLI on the current machine. Measure an actual model and
harness separately when making performance or cost decisions.

Safety boundary
---------------
A passing bitctx mask means only that stored bits satisfy the mask. It does not
authenticate, grant permission, or prove that external policy was enforced.
EOF
