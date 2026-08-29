---
name: bit-context
description: Use when a task needs a deterministic persistent bitctx gate for explicit boolean prerequisites, including verified values and evidence gaps that must remain unsatisfied, named mask evaluation, or ordered missing-condition reporting. Do not use to discover facts, decide policy, or replace ordinary reasoning when no persistent gate state is needed.
---

# Bit Context

Use `bitctx` as a deterministic store and evaluator for verified boolean conditions. Treat it as a gate calculator, not as a source of facts or authority.

## Preconditions

1. Run `command -v bitctx` before any workflow.
2. If the command is missing, stop and report that `bitctx` must be installed. Do not install it automatically.
3. Choose an explicit session ID matching `[A-Za-z0-9][A-Za-z0-9._-]{0,127}`. Never rely on an implicit or shared default session.
4. Use `--data-dir <PATH>` or `BITCTX_DATA_DIR` only when the task requires an isolated or non-default store.

## Safety Boundary

- Obtain each condition value from an authoritative tool, API, policy engine, test, or user-provided fact before setting it.
- Never invent, infer, or optimistically set an unverified condition. An unset zero bit means only that the gate is unsatisfied; it does not prove a verified negative.
- Interpret `pass: true` only as “the stored bits satisfy this schema mask.” It does not prove identity, permission, policy approval, safety, payment, legal compliance, or authorization outside `bitctx`.
- Do not put secrets in bit names, descriptions, session IDs, or state. The store is local plaintext JSON.
- Use `init --force` and `reset --force` only when replacing or deleting the named session is explicitly intended.

## Workflow

1. Define a schema that maps stable bit indices from 0 through 63 to unique condition names and defines named AND masks.
2. Initialize a new session:

   ```bash
   bitctx init --session "$SESSION_ID" --schema schema.json
   ```

3. Run the real checks outside `bitctx`. Set only the results that were actually observed:

   ```bash
   bitctx set --session "$SESSION_ID" \
     --bit user_authenticated,has_permission \
     --value true,true
   ```

4. Evaluate a named mask as JSON:

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

## Error Handling

- If a session is not initialized, initialize it from the intended schema; do not use `set` to create it implicitly.
- If the schema hash does not match, stop. Reconcile the schema and session instead of forcing values into stale state.
- If condition evidence is missing, leave the bit unsatisfied and report the evidence gap.
- If a command fails, preserve the exact session ID, data directory, command, and error in the report.

The compatibility wrapper at `bitctx_skill.sh` accepts the same workflow through `BITCTX_SESSION`, but direct CLI calls are preferred for transparent arguments and error reporting.
