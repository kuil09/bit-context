---
id: goal-bitctx-cli
type: goal-and-requirements
status: implemented
title: "bitctx v0.3: Deterministic Boolean Context Store"
tags: [cli, rust, bitwise, context, codex-skill]
---

## Goal

Provide a local CLI and Codex skill that can persist boolean results already verified by a harness and evaluate named AND gates deterministically. The result is a compact decision artifact; it is not a fact-discovery, authentication, policy, or authorization system.

## Target constraints

- `init`, `set`, `eval`, `resume`, `explain`, `dump`, and `reset` operate through an explicit safe session ID.
- `init` creates a schema and zeroed state that is immediately evaluable.
- A named mask returns an ordered pass/fail result and structured missing conditions.
- `resume` restores the selected stored decision state without returning settled checkpoints as continuation work.
- Text evaluation renders a fixed 8×8 matrix covering bit positions 0 through 63.
- Text details can be filtered to all, satisfied, or missing conditions while preserving mask order.
- Concurrent writers to one session do not lose successful updates.
- The repository ships an installable Codex skill and four release binaries for Linux/macOS on x86-64/ARM64.

## Preservation constraints

- Valid v0.1 `schema.json` and `session.json` files remain readable.
- The schema hash algorithm remains compatible with v0.1.
- `missing` and `missing_labels` remain in the JSON output.
- JSON remains the default evaluation format and keeps its existing structure.
- External checks remain authoritative; `bitctx` never manufactures condition values or grants permission.
- State remains recoverable JSON, with explicit documentation that it is plaintext.

## Interface

```text
bitctx [--data-dir PATH] init    --session ID --schema FILE [--force]
bitctx [--data-dir PATH] set     --session ID --bit NAMES --value VALUES
bitctx [--data-dir PATH] eval    --session ID --mask NAME [--format json|text] [--show all|satisfied|missing]
bitctx [--data-dir PATH] resume  --session ID [--mask NAME] [--format json|text]
bitctx [--data-dir PATH] explain --session ID --mask NAME [--lang ko|en]
bitctx [--data-dir PATH] dump    --session ID [--format json|text]
bitctx [--data-dir PATH] reset   --session ID [--force]
```

Data directory precedence is CLI `--data-dir`, `BITCTX_DATA_DIR`, then `~/.bitctx`.

In text evaluation, bit 0 is the top-left matrix cell and bit 63 is the bottom-right. `O` denotes a selected satisfied condition, `X` denotes a selected unsatisfied condition, and `·` denotes a position outside the mask. Unsatisfied does not assert a verified negative. `--show` is valid only with text output.

Session IDs match `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` and must be one normal path component.

## Explain contract

```text
bitctx [--data-dir PATH] explain --session ID --mask NAME [--lang ko|en]
```

- Read-only; takes a shared session lock.
- Prints a human-readable list of missing conditions for the selected mask only.
- No JSON output; output is Korean (default) or English per `--lang`.
- If the mask passes, prints a single "all satisfied" line.
- If the mask fails, prints the mask description followed by each missing condition as "- <name>: <description>" in mask definition order.
- Unknown session, mask, or schema hash mismatch fails without creating or mutating state.

## Dump contract

```text
bitctx [--data-dir PATH] dump --session ID [--format json|text]
```

- Read-only; takes a shared session lock.
- JSON output structure:
  ```json
  {
    "session_id": "...",
    "schema_hash": "...",
    "bits": 0,
    "bit_states": [
      {"index": 0, "name": "...", "value": false, "desc": "..."}
    ],
    "created_at": "...",
    "updated_at": "..."
  }
  ```
  - `bits` is the raw u64 bitfield.
  - `bit_states` lists all defined bits in index order with current boolean value.
- Text output prints the same fields plus a visual `●`/`○` status column for each bit.
- Unknown session or schema hash mismatch fails without creating or mutating state.

## Skill package contract

The release includes `bit-context-skill.zip` containing the independently installable skill directory:

```
bit-context/
├── SKILL.md
├── agents/openai.yaml
├── bitctx_skill.sh
├── example_schema.json
├── README.md
└── README.ko.md
```

- `SKILL.md` — frontmatter `name: bit-context` with validated description; body covers preconditions, safety boundary, resume-first workflow, cross-session resume, initialize-once, changes/invalidation, result handling, error handling.
- `agents/openai.yaml` — Codex metadata: `display_name: "Bit Context"`, `short_description` (25–64 chars), `default_prompt` referencing `$bit-context`, `allow_implicit_invocation: true`.
- `bitctx_skill.sh` — compatibility wrapper requiring `BITCTX_SESSION`; subcommands `init`, `set`, `set-multi`, `eval`, `resume`, `explain`, `dump`, `reset`; validates binary existence, session presence, argument counts.
- `example_schema.json` — valid v1 schema with `default_mask`, 5 bits, 3 masks; used in skill documentation.
- READMEs — installation and usage in English and Korean.

## Benchmark contracts

- `bench_perf.sh` — measures local CLI process startup, locking, read, and JSON evaluation latency over N iterations (default 100). Reports mean microseconds per call. Includes an illustrative (non-measured) LLM comparison with explicit assumptions. Runs against an isolated temporary data directory.
- `bench_harness.sh` — prints a design comparison between a natural-language LLM gate and a bitctx boolean gate. All token counts, latencies, and cost figures are illustrative assumptions, not measurements.

## Storage contract

```text
<data-dir>/
├── .locks/<session>.lock
└── <session>/
    ├── schema.json
    └── session.json
```

- Writers use an exclusive per-session lock; readers use a shared lock.
- Lock files live outside the session directory so force initialization and reset cannot unlink the active lock.
- Lock files may intentionally persist after reset to keep the synchronization inode stable across concurrent processes.
- Writes use a same-directory temporary file, flush, sync, and rename.
- Before recursive deletion, the implementation revalidates that the target is a real directory directly below the data root.
- Unix directories use mode `0700`; state and lock files use mode `0600`.

## Schema contract

- Schema version is `1`.
- Bit indices range from 0 through 63.
- Bit names are unique and non-empty and contain no comma, leading/trailing whitespace, or control character.
- JSON bit indices, mask names, and mask bit entries cannot be duplicated.
- Each mask is non-empty and references only defined bits.
- An optional `default_mask` names an existing mask and is included in the schema hash.
- Description strings accept Unicode.
- Missing indices, names, and structured conditions preserve mask definition order.

## Boundary constraints

- Rust edition 2024 with MSRV 1.85.
- Release support is Linux and macOS on x86-64 and ARM64.
- Windows, daemon mode, network synchronization, schema migration, arbitrary bit width, and automatic evidence collection are out of scope for v0.3.
- The Codex skill checks for `bitctx` but does not install it automatically.
- The wrapper requires `BITCTX_SESSION` and has no shared default session.

## Evidence conditions

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --locked`, including CLI flow, path attacks, schema mismatch, v0.1 compatibility, mask ordering, and concurrent updates
- ShellCheck plus fake-binary wrapper and fail-closed installer tests
- Codex skill frontmatter and `agents/openai.yaml` validation
- Tag workflow quality gate, all four builds, mandatory SHA-256 files, and a verified `bit-context-skill.zip`
- Native release-asset end-to-end flow before the release is considered complete

Performance scripts report local CLI measurements separately from illustrative token and remote-model assumptions. No token, latency, or cost figure is a product guarantee.