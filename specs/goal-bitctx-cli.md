---
id: goal-bitctx-cli
type: goal-and-requirements
status: implemented
title: "bitctx v0.2: Deterministic Boolean Context Store"
tags: [cli, rust, bitwise, context, codex-skill]
---

## Goal

Provide a local CLI and Codex skill that can persist boolean results already verified by a harness and evaluate named AND gates deterministically. The result is a compact decision artifact; it is not a fact-discovery, authentication, policy, or authorization system.

## Target constraints

- `init`, `set`, `eval`, `explain`, `dump`, and `reset` operate through an explicit safe session ID.
- `init` creates a schema and zeroed state that is immediately evaluable.
- A named mask returns an ordered pass/fail result and structured missing conditions.
- Concurrent writers to one session do not lose successful updates.
- The repository ships an installable Codex skill and four release binaries for Linux/macOS on x86-64/ARM64.

## Preservation constraints

- Valid v0.1 `schema.json` and `session.json` files remain readable.
- The schema hash algorithm remains compatible with v0.1.
- `missing` and `missing_labels` remain in the JSON output.
- External checks remain authoritative; `bitctx` never manufactures condition values or grants permission.
- State remains recoverable JSON, with explicit documentation that it is plaintext.

## Interface

```text
bitctx [--data-dir PATH] init    --session ID --schema FILE [--force]
bitctx [--data-dir PATH] set     --session ID --bit NAMES --value VALUES
bitctx [--data-dir PATH] eval    --session ID --mask NAME [--format json|text]
bitctx [--data-dir PATH] explain --session ID --mask NAME [--lang ko|en]
bitctx [--data-dir PATH] dump    --session ID [--format json|text]
bitctx [--data-dir PATH] reset   --session ID [--force]
```

Data directory precedence is CLI `--data-dir`, `BITCTX_DATA_DIR`, then `~/.bitctx`.

Session IDs match `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` and must be one normal path component.

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
- Description strings accept Unicode.
- Missing indices, names, and structured conditions preserve mask definition order.

## Boundary constraints

- Rust edition 2024 with MSRV 1.85.
- Release support is Linux and macOS on x86-64 and ARM64.
- Windows, daemon mode, network synchronization, schema migration, arbitrary bit width, and automatic evidence collection are out of scope for v0.2.
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
