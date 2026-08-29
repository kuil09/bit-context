# Bit Context Codex skill

This directory is an installable Codex skill for deterministic evaluation of boolean conditions that were already verified outside `bitctx`.

[한국어 안내](README.ko.md)

## Contents

- `SKILL.md`: agent instructions and safety boundary
- `agents/openai.yaml`: Codex discovery metadata
- `bitctx_skill.sh`: optional compatibility wrapper
- `example_schema.json`: example schema

## Prerequisite

Install `bitctx` separately and confirm it is available:

```bash
command -v bitctx
bitctx --version
```

The skill and wrapper never install the binary automatically.

## Install the skill

Copy or extract this directory as `bit-context` under your Codex skills directory. Keep `SKILL.md` at the root of that directory.

## Direct CLI workflow

```bash
SESSION_ID=task-123
bitctx init --session "$SESSION_ID" --schema example_schema.json
bitctx set --session "$SESSION_ID" \
  --bit user_authenticated,has_permission \
  --value true,true
bitctx eval --session "$SESSION_ID" --mask required --format json
```

Only set values backed by real evidence. A passing result means only that the stored values satisfy the selected mask; it is not external authorization.

## Compatibility wrapper

The wrapper requires an explicit session for every command except help:

```bash
export BITCTX_SESSION=task-123
./bitctx_skill.sh init example_schema.json
./bitctx_skill.sh eval required json
./bitctx_skill.sh init example_schema.json --force
./bitctx_skill.sh reset --force
```

Optional variables:

- `BITCTX_BIN`: executable path, default `bitctx`
- `BITCTX_DATA_DIR`: isolated data directory, default `~/.bitctx`

Session state is plaintext JSON. Do not store secrets in it.
