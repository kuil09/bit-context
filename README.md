# bit-context

`bitctx` is a small Rust CLI that stores verified boolean conditions as bits and evaluates named AND masks deterministically. It is useful when a harness already knows how to check each condition and needs a compact, persistent gate result.

[한국어 문서](README.ko.md)

## Boundary

`bitctx` does not authenticate users, discover facts, evaluate policy, or grant permission. A passing mask means only that the values stored in that session satisfy the selected schema mask. The caller remains responsible for obtaining trustworthy condition values and enforcing every external authorization and policy constraint.

State is stored as plaintext JSON. Do not put secrets in session IDs, bit names, descriptions, or state.

## Install

Supported release platforms are Linux and macOS on x86-64 and ARM64. Windows is not supported in the v0.2 release series.

Install the latest signed-by-checksum release asset:

```bash
curl -fsSL https://raw.githubusercontent.com/kuil09/bit-context/main/install.sh | bash
```

Set `INSTALL_DIR` to install somewhere other than `/usr/local/bin`. The installer fails closed when the release checksum, checksum tool, asset, or download is unavailable.

Build from source with Rust 1.85 or newer:

```bash
cd bitctx-cli
cargo build --release --locked
install target/release/bitctx /usr/local/bin/bitctx
```

## Quick start

Create a schema:

```json
{
  "version": 1,
  "bits": {
    "0": {"name": "user_authenticated", "desc": "Authentication was verified"},
    "1": {"name": "has_permission", "desc": "Required permission was verified"},
    "2": {"name": "quota_ok", "desc": "Quota check passed"}
  },
  "masks": {
    "required": {"bits": [0, 1, 2], "desc": "All required conditions"}
  }
}
```

Initialize and evaluate an explicit session:

```bash
bitctx init --session deploy-123 --schema schema.json

# A new v0.2 session is immediately evaluable and starts with all bits at zero.
bitctx eval --session deploy-123 --mask required --format json

# Set only values obtained from real checks.
bitctx set --session deploy-123 \
  --bit user_authenticated,has_permission,quota_ok \
  --value true,true,false

bitctx eval --session deploy-123 --mask required --format json
```

Failure output preserves mask definition order and contains both compatibility fields and structured details:

```json
{
  "pass": false,
  "missing": [2],
  "missing_labels": ["quota_ok"],
  "missing_conditions": [
    {"index": 2, "name": "quota_ok", "desc": "Quota check passed"}
  ]
}
```

For a compact visual check, text output renders every bit position in a fixed 8×8 matrix:

```bash
bitctx eval --session deploy-123 --mask required --format text
```

```text
     0   1   2   3   4   5   6   7
00 ┌───┬───┬───┬───┬───┬───┬───┬───┐
   │ O │ O │ X │ · │ · │ · │ · │ · │
08 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
16 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
24 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
32 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
40 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
48 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
56 ├───┼───┼───┼───┼───┼───┼───┼───┤
   │ · │ · │ · │ · │ · │ · │ · │ · │
   └───┴───┴───┴───┴───┴───┴───┴───┘

RESULT: X
```

Bit 0 is the top-left cell and bit 63 is the bottom-right cell. `O` means a selected condition is satisfied, `X` means it is currently unsatisfied, and `·` means the position is outside the selected mask. An `X` is not a verified false claim; it can also represent an unset condition with no evidence yet.

Add `--show all`, `--show satisfied`, or `--show missing` to text output to list matching condition names and descriptions in mask definition order. `--show` is rejected with JSON output, whose default and structure remain unchanged.

## Commands

```text
bitctx [--data-dir PATH] init    --session ID --schema FILE [--force]
bitctx [--data-dir PATH] set     --session ID --bit NAMES --value VALUES
bitctx [--data-dir PATH] eval    --session ID --mask NAME [--format json|text] [--show all|satisfied|missing]
bitctx [--data-dir PATH] explain --session ID --mask NAME [--lang ko|en]
bitctx [--data-dir PATH] dump    --session ID [--format json|text]
bitctx [--data-dir PATH] reset   --session ID [--force]
```

The data directory is selected in this order:

1. `--data-dir <PATH>`
2. `BITCTX_DATA_DIR`
3. `~/.bitctx`

Each session ID must match `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` and be one normal path component. Path separators, absolute paths, `.`, `..`, control characters, and longer IDs are rejected.

## Storage and concurrency

The default layout is:

```text
~/.bitctx/
├── .locks/
│   └── deploy-123.lock
└── deploy-123/
    ├── schema.json
    └── session.json
```

- `init`, `set`, and `reset` take an exclusive per-session lock.
- `eval`, `explain`, and `dump` take a shared per-session lock.
- Lock files live outside deletable session directories.
- A small lock file may remain after `reset`; keeping its inode stable prevents lock-unlink races when another process still references the session ID.
- State writes use a same-directory temporary file, flush and sync it, then rename it atomically.
- On Unix, data directories use mode `0700`; state and lock files use mode `0600`.
- `set` fails when the session was not initialized.
- `init --force` reinitializes the schema and zeroes every bit while holding the lock.

Schema validation rejects duplicate JSON indices, duplicate bit names, invalid names, unknown mask references, empty masks, and duplicate bits within a mask. Descriptions may contain Unicode.

## v0.2 migration

Valid v0.1 `schema.json` and `session.json` files remain readable and retain the same schema hash algorithm. Notable behavior changes are:

- `init` now creates both files with a zeroed session, so `eval` works immediately.
- The compatibility wrapper no longer uses `BITCTX_SESSION=default`; set an explicit session.
- `set` no longer creates missing sessions.
- Unsafe session IDs are rejected.
- `eval` adds `missing_conditions` without removing `missing` or `missing_labels`.

## Codex skill

The release includes `bit-context-skill.zip`, containing `skills/bit-context/SKILL.md`, `agents/openai.yaml`, the compatibility wrapper, and an example schema. Extract the `bit-context` directory into your Codex skills directory, then restart or refresh skill discovery.

The skill checks that `bitctx` is installed but never installs it automatically. It only sets condition values backed by observed evidence and never treats a passing mask as external authorization. When a known task continues, it evaluates the existing session first, treats unchanged true bits as settled checkpoints, and reports only new work, changed bits, and remaining conditions.

The wrapper is optional:

```bash
export BITCTX_SESSION=deploy-123
skills/bit-context/bitctx_skill.sh eval required json
skills/bit-context/bitctx_skill.sh eval required text missing
```

## Performance evidence

`bench_perf.sh` measures local CLI process and evaluation time on the machine where it is run. `bench_harness.sh` shows an illustrative harness comparison; its token, model latency, and cost figures are examples, not guarantees or measurements of the local CLI.

Run a local measurement with an explicit binary and isolated data directory:

```bash
BITCTX=bitctx ITERATIONS=100 ./bench_perf.sh
```

## Development

```bash
cd bitctx-cli
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --locked

cd ..
bash tests/test_wrapper.sh
shellcheck install.sh bench_perf.sh bench_harness.sh \
  skills/bit-context/bitctx_skill.sh tests/test_installer.sh \
  tests/smoke_release.sh tests/test_wrapper.sh
```

## License

MIT
