# bit-context

> **Bit-memory context store for AI harness skills** — Replace verbose LLM reasoning with deterministic bitwise operations.

## The Problem

AI harnesses often ask LLMs to evaluate dozens of boolean conditions:
- "Check if user is authenticated, has permission, quota OK, rate limit OK, resource exists..."
- LLM must read all conditions, call tools for each, reason about AND/OR logic
- **Slow (seconds), expensive (tokens), non-deterministic (hallucination risk)**

## The Solution

**bitctx** moves condition evaluation out of the LLM:

```
┌─────────────┐     Deterministic      ┌──────────────┐     Minimal      ┌─────┐
│  Harness    │ ────── boolean ──────► │   bitctx     │ ──── pass/fail ──► │ LLM │
│  (Python)   │   checks (code)        │  (bitwise)   │   + failed bits   │     │
└─────────────┘                        └──────────────┘                  └─────┘
        │                                      │                              │
        │  check_auth()                        │  0b1011 & 0b1111             │  "Approved"
        │  check_rbac()                        │  = 0b1011 (pass)             │
        │  check_quota()                       │                              │
        ▼                                      ▼                              ▼
   Code decides                          Bitwise AND                      Generates
   each condition                        O(1) eval                        human text
```

**Result**: 80%+ token reduction, 50x latency improvement, deterministic decisions.

---

## Architecture

| Component | Role |
|-----------|------|
| **bitctx CLI** | Rust binary: bit memory + mask eval + natural language decode |
| **Schema** | JSON: bit index ↔ name/description, named masks (AND combinations) |
| **Storage** | `~/.bitctx/<session>.json` (file-based, atomic writes, file locking) |
| **Skill Wrapper** | `skills/bit-context/bitctx_skill.sh` for harness integration |

---

## Quick Start

```bash
# Build
cd bitctx-cli && cargo build --release

# Define schema (schema.json)
{
  "version": 1,
  "bits": {
    "0": {"name": "user_authenticated", "desc": "User authenticated"},
    "1": {"name": "has_permission", "desc": "Has required permission"},
    "2": {"name": "quota_ok", "desc": "Quota not exceeded"}
  },
  "masks": {
    "required": {"bits": [0, 1, 2], "desc": "All required conditions"}
  }
}

# Initialize session
bitctx init --session deploy-123 --schema schema.json

# Harness sets bits after checking conditions
bitctx set --session deploy-123 --bit user_authenticated,has_permission --value true,true

# Instant bitwise evaluation
bitctx eval --session deploy-123 --mask required --format json
# {"pass":false,"missing":[2],"missing_labels":["quota_ok"]}

# Natural language explanation (only on failure)
bitctx explain --session deploy-123 --mask required --lang en
# "Conditions not satisfied: quota_ok"
```

---

## Harness Integration (Python)

```python
import subprocess, json, os

os.environ["BITCTX_SESSION"] = "task-123"
BITCTX = "/usr/local/bin/bitctx"

def bitctx_init(schema_path):
    subprocess.run([BITCTX, "init", "--session", "task-123", "--schema", schema_path], check=True)

def bitctx_set(bits: dict):
    names = ",".join(bits.keys())
    vals = ",".join("true" if v else "false" for v in bits.values())
    subprocess.run([BITCTX, "set", "--session", "task-123", "--bit", names, "--value", vals], check=True)

def bitctx_eval(mask: str) -> dict:
    result = subprocess.run([BITCTX, "eval", "--session", "task-123", "--mask", mask, "--format", "json"],
                           capture_output=True, text=True, check=True)
    return json.loads(result.stdout)

def bitctx_explain(mask: str) -> str:
    result = subprocess.run([BITCTX, "explain", "--session", "task-123", "--mask", mask, "--lang", "en"],
                           capture_output=True, text=True, check=True)
    return result.stdout.strip()

# Usage
bitctx_init("schema.json")
bitctx_set({"user_authenticated": True, "has_permission": True, "quota_ok": False})

result = bitctx_eval("required")
if result["pass"]:
    prompt = "Deployment approved."
else:
    prompt = f"Deployment rejected: {bitctx_explain('required')}"
```

---

## Performance

| Metric | Without bitctx | With bitctx |
|--------|----------------|-------------|
| Prompt tokens | ~450 | ~40 |
| LLM tool calls | 12+ | 0 |
| Latency | 2-10 sec | 0.2-0.5 sec |
| Deterministic | ❌ | ✅ |

See [bench_harness.sh](bench_harness.sh) for detailed comparison.

---

## Project Structure

```
bit-mania/
├── specs/
│   └── goal-bitctx-cli.md       # Goal & requirements spec
├── bitctx-cli/                  # Rust CLI
│   ├── src/
│   │   ├── models/              # Schema, Session
│   │   ├── storage/             # JSON I/O, file locking
│   │   └── commands/            # init, set, eval, explain, dump, reset
│   └── Cargo.toml
├── skills/bit-context/          # Harness skill wrapper
│   ├── bitctx_skill.sh
│   ├── example_schema.json
│   └── README.md
├── bench_perf.sh                # Microbenchmark
├── bench_harness.sh             # Realistic harness comparison
└── README.ko.md                 # Korean version
```

---

## Roadmap

- [ ] v2: Daemon mode (Unix socket) for sub-millisecond eval
- [ ] v2: Library embedding (Rust crate / Python bindings)
- [ ] v2: Arbitrary bit width (bitvec)
- [ ] Schema migration tool
- [ ] TTL/auto-expiry for bits

---

## License

MIT