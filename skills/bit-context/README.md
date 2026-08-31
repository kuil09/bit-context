# Bit Context Codex skill

This directory is an installable Codex skill for deterministic evaluation of boolean conditions that were already verified outside `bitctx`. It can also resume long-running work from stable checkpoints without replaying completed procedures.

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
bitctx resume --session "$SESSION_ID" --format json
```

Only set values backed by real evidence. A passing result means only that the stored values satisfy the selected mask; it is not external authorization.

## Resume without replay

Reuse the same explicit session when a task continues. Evaluate the target mask first, treat unchanged true bits as settled checkpoints, and work only on missing or newly invalidated conditions:

```bash
bitctx eval --session "$SESSION_ID" --mask required --format json

# A new observation invalidated one previous checkpoint.
bitctx set --session "$SESSION_ID" --bit quota_ok --value false
bitctx eval --session "$SESSION_ID" --mask required --format json
```

Do not encode the whole conversation as bits. Keep source material and nuanced reasoning outside `bitctx`; use bits only for stable, decision-relevant checkpoints. On continuation, report newly checked or changed conditions and the ordered `missing_conditions` instead of recapping completed work.

In a new conversation, agent, or fresh context, a known session ID can restore the stored decision state without replaying the transcript:

```bash
bitctx resume --session "$SESSION_ID" --format json
```

`resume` selects `default_mask`, or the only schema mask. With multiple masks and no default, pass `--mask` explicitly. Its `freshness` field is always `unverified`: the command restores the checkpoint but cannot prove that external evidence is still current.

Always inspect the JSON `pass` field. A successful `eval` or `resume` process exit means that evaluation ran, not that the selected mask passed.

For a human-readable overview, use `--format text`. It always renders bit positions 0 through 63 as an 8×8 matrix: `O` is satisfied, `X` is unsatisfied, and `·` is outside the selected mask. `X` does not prove a verified negative. Add `--show all`, `--show satisfied`, or `--show missing` for ordered details.

## Compatibility wrapper

The wrapper requires an explicit session for every command except help:

```bash
export BITCTX_SESSION=task-123
./bitctx_skill.sh init example_schema.json
./bitctx_skill.sh eval required json
./bitctx_skill.sh resume
./bitctx_skill.sh eval required text missing
./bitctx_skill.sh init example_schema.json --force
./bitctx_skill.sh reset --force
```

Optional variables:

- `BITCTX_BIN`: executable path, default `bitctx`
- `BITCTX_DATA_DIR`: isolated data directory, default `~/.bitctx`

Session state is plaintext JSON. Do not store secrets in it.
