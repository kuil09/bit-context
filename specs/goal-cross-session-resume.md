---
id: goal-cross-session-resume
type: goal-and-requirements
status: implemented
title: "Cross-session decision-state resume"
tags: [cli, context, resume, codex-skill]
---

## Goal

Let a fresh conversation or agent restore the ordered decision state of a known
local session without replaying completed work or requesting the previous
transcript.

## CLI contract

```text
bitctx [--data-dir PATH] resume --session ID [--mask NAME] [--format json|text]
```

- `resume` is read-only and takes the same shared session lock as `eval`.
- Mask selection precedence is explicit `--mask`, schema `default_mask`, then
  the schema's only mask.
- Multiple masks without either an explicit or default selection fail and list
  the available names. The command never guesses.
- JSON returns session ID, schema hash, selected mask, pass state, ordered
  missing indices/labels/conditions, update time, and
  `freshness: "unverified"`.
- Text output reports the same status and only missing conditions; it does not
  replay satisfied checkpoints.
- An unknown session, mask, invalid default mask, or schema hash mismatch fails
  without creating or mutating state.

## Schema contract

`default_mask` is an optional top-level schema field. When present it must name
an existing mask and participates in the schema hash. When absent, canonical
hashes for existing schemas remain unchanged.

## Skill contract

When a known session ID is provided in a new conversation, agent, or fresh
context, the skill runs `resume` before requesting prior history or
reconstructing completed work. It processes only `missing_conditions` and
treats `freshness: "unverified"` as a warning that external evidence may need
invalidation or revalidation.

## Boundary

Resume restores stored decision state only. It does not restore source text,
proofs, nuanced reasoning, task-to-session discovery, external evidence
freshness, or state from a different machine/data directory.
