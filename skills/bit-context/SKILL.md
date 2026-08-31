---
name: bit-context
description: Use when a task needs deterministic persistent boolean gates or long-running work should resume from verified checkpoints without replaying completed steps. Track externally verified values, evaluate named masks, and report only missing or changed conditions. Do not use to discover facts, decide policy, or compress nuanced reasoning into booleans.
---

# Bit Context

Use `bitctx` as a deterministic store and evaluator for verified boolean conditions. Treat it as a compact checkpoint index and gate calculator, not as a transcript, source of facts, or authority.

## Preconditions

1. Run `command -v bitctx` before the first workflow in an environment. Do not repeat this check on every continuation unless the environment changed.
2. If the command is missing, stop and report that `bitctx` must be installed. Do not install it automatically.
3. Choose an explicit stable session ID matching `[A-Za-z0-9][A-Za-z0-9._-]{0,127}` and reuse it for the same task. Never rely on an implicit or shared default session.
4. Use `--data-dir <PATH>` or `BITCTX_DATA_DIR` only when the task requires an isolated or non-default store.

## Safety Boundary

- Obtain each condition value from an authoritative tool, API, policy engine, test, or user-provided fact before setting it.
- Never invent, infer, or optimistically set an unverified condition. An unset zero bit means only that the gate is unsatisfied; it does not prove a verified negative.
- Treat a true bit as a settled checkpoint only while its underlying input and evidence remain unchanged. New conflicting evidence invalidates it.
- Interpret `pass: true` only as “the stored bits satisfy this schema mask.” It does not prove identity, permission, policy approval, safety, payment, legal compliance, or authorization outside `bitctx`.
- Keep proofs, source text, and other nuanced context outside `bitctx`; bits only index the decision-relevant state.
- Do not put secrets in bit names, descriptions, session IDs, or state. The store is local plaintext JSON.
- Use `init --force` and `reset --force` only when replacing or deleting the named session is explicitly intended.

## Resume First

When continuing or reviewing an existing task:

1. Reuse the known session ID, data directory, schema, and target mask. Do not create a new session or guess an identifier.
2. Evaluate the target mask before replaying prior work:

   ```bash
   bitctx eval --session "$SESSION_ID" --mask required --format json
   ```

3. Unless the user requests a full audit or new evidence invalidates them, treat true bits as settled checkpoints. Do not re-run, reconsider, or narrate their completed procedures.
4. Work only on conditions listed in `missing_conditions` or conditions explicitly affected by new input.
5. Set only the changed, newly verified results, then evaluate the mask again.
6. Report the delta: checks performed now, bits changed now, and conditions still missing. Do not recap settled checkpoints unless asked.

## Cross-Session Resume

When another conversation, agent, or fresh context receives a known session ID:

1. Reuse the known data directory. Before asking for the previous transcript or reconstructing completed work, resume the stored decision state:

   ```bash
   bitctx resume --session "$SESSION_ID" --format json
   ```

2. Let `resume` use the schema's `default_mask` or its only mask. If multiple masks are ambiguous, use the task's known mask explicitly; never guess:

   ```bash
   bitctx resume --session "$SESSION_ID" --mask required --format json
   ```

3. Parse `pass`; command success only means the stored state was read and evaluated. Treat `missing_conditions` as the ordered continuation scope. Do not replay or narrate conditions omitted from that list unless the user requests an audit or new evidence affects them.
4. Treat `freshness: "unverified"` literally. Resume restores stored decision state; it does not prove that external evidence is still current. Apply the Changes and Invalidation rules before relying on affected bits.
5. If the expected session or task-to-session mapping is unavailable, stop instead of initializing a replacement.

## Initialize Once

1. Define a schema that maps stable bit indices from 0 through 63 to unique condition names and defines named AND masks.
2. Initialize one explicit session for the task:

   ```bash
   bitctx init --session "$SESSION_ID" --schema schema.json
   ```

3. Run the real checks outside `bitctx`. Set only the results that were actually observed:

   ```bash
   bitctx set --session "$SESSION_ID" \
     --bit user_authenticated,has_permission \
     --value true,true
   ```

4. Evaluate a named mask as JSON and use this session on later continuations:

   ```bash
   bitctx eval --session "$SESSION_ID" --mask required --format json
   ```

5. Read `missing`, `missing_labels`, and `missing_conditions` in their schema mask order. On failure, use the structured conditions to explain what remains unsatisfied.
6. Before acting on a passing result, separately confirm any external authorization or policy constraint the action requires.
7. Inspect or remove state only when needed:

   ```bash
   bitctx dump --session "$SESSION_ID" --format json
   bitctx reset --session "$SESSION_ID" --force
   ```

## Changes and Invalidation

- When source data, requirements, configuration, or evidence changes, clear only the affected bits and any checkpoints that depend on them. Re-run those checks before setting them true again.
- Preserve unrelated true bits. Do not reset or rebuild the whole session merely because one condition changed.
- If the schema itself must change, create or reconcile the intended schema deliberately. Never bypass a schema hash mismatch with optimistic values.
- If the affected checkpoints cannot be identified safely, leave the relevant gate unsatisfied and report the ambiguity.

## Result Handling

- Parse the JSON `pass` field to decide whether a mask passed. A successful `bitctx eval` process exit only means that evaluation ran; it does not mean the mask passed.
- On a resumed task, prefer the ordered `missing_conditions` list over reconstructing progress from conversation history.
- For a human-readable overview, optionally use `--format text` and `--show missing`. Its fixed 8×8 matrix uses `O` for satisfied selected bits, `X` for unsatisfied selected bits, and `·` for positions outside the mask; `X` does not prove a verified negative.
- A passing checkpoint mask permits the workflow to continue only within the caller's own rules; it is not external authorization.

## Error Handling

- If a session is not initialized, initialize it from the intended schema; do not use `set` to create it implicitly.
- If an expected session cannot be identified, stop and report the missing task-to-session mapping instead of silently starting over.
- If the schema hash does not match, stop. Reconcile the schema and session instead of forcing values into stale state.
- If condition evidence is missing, leave the bit unsatisfied and report the evidence gap.
- If a command fails, preserve the exact session ID, data directory, command, and error in the report.

The compatibility wrapper at `bitctx_skill.sh` accepts the same workflow through `BITCTX_SESSION`, but direct CLI calls are preferred for transparent arguments and error reporting.
