---
name: composition-designer
description: Modify and validate a Composition Designer project through the Composition MCP tools. Use for inspecting a TOML composition, preparing/apply atomic transactions, reading normalized models, checking L1-L7 diagnostics, or explaining why an edit was rejected.
---

# Composition Designer Agent Skill

Use the `composition-designer` MCP server. Never edit `composition/**/*.toml`, module files or
`.designer/**` directly. All project changes go through the MCP transaction tools so validation,
Revision and TOML formatting stay consistent.

## Available tools

| Tool | Use |
|---|---|
| `composition_inspect` | Project identity, modules, validity, source digest, canEdit |
| `composition_resolve` | Normalized model, including `revision.seq` and contract IDs |
| `composition_validate` | Validity plus L1–L7 diagnostics |
| `composition_source` | Read-only composition TOML |
| `composition_context` | Compact project/module/instance/diagnostic context for planning |
| `composition_impact` | References for a module or instance, or dry-run impact of a transaction |
| `composition_batch` | Up to 16 read-only queries in one call; each returns its own result |
| `composition_prepare_transaction` | Validate a transaction and inspect its Diff; no write |
| `composition_apply_transaction` | Atomically apply a transaction |

Resources: `composition://project/inspect`, `composition://project/model`,
`composition://project/diagnostics`, `composition://project/source`,
`composition://project/context`.

## Workflow

1. **Inspect** the project. If `canEdit` is false, stop and report the diagnostics.
2. **Resolve** the model. Read `model.revision.seq` / `model.revision.source_digest`.
   Read module contract IDs from `model.modules[].contract`.
3. **Context** with `composition_context` for a compact module/instance index when the project
   is large. Use `composition_impact` with `{ "instance": "..." }` or
   `{ "module": { "id": "...", "version": "..." } }` before changing or deleting an object:
   check parent/child bindings, connections, dependents and explicit providers.
   For several independent reads, call `composition_batch` with up to 16
   `{ "id", "op", "params" }` items instead of multiple round trips.
4. Build a transaction:

```json
{
  "request_id": "agent-<unique>",
  "base_revision": {
    "seq": 0,
    "source_digest": "sha256:..."
  },
  "commands": [
    {
      "type": "set_config",
      "instance": "app",
      "property": "title",
      "value": "New title"
    }
  ]
}
```

4. Call `composition_prepare_transaction` with `{ "transaction": ... }`.
   - `isError=false` and `accepted=true`: inspect `diff.summary`/`diff.changes`.
   - `isError=true`: read the diagnostics; do not retry the same payload blindly.
   - For a denied or risky edit, call `composition_impact` with
     `{ "transaction": ... }`: it runs the same prepare without writing and lists
     `changedObjects` plus diagnostics.
5. Call `composition_apply_transaction` with the same transaction.
6. Call `composition_validate`; report `valid`, error codes and source locations.
7. Optionally read `composition_source` to confirm the written TOML.

## Rules

- Use a fresh `request_id` per intent. A repeated identical request may replay; reusing an ID with a
  different payload is rejected with `edit-duplicate-request`.
- Always use the latest `revision.seq` and `source_digest`.
- Never overwrite `composition/**` with a text editor. Use `composition_apply_transaction`.
- If you receive `edit-revision-conflict`, the project changed on disk or the Revision is stale:
  re-run `composition_resolve` / reload the MCP server, then prepare a new transaction.
- `composition_source` is read-only. Source editing is not part of protocol v1.
- `composition_context` / `composition_impact` are derived from the same model and never write;
  an impact transaction target is a prepare-only preview, not an apply.
- `composition_impact` accepts exactly one of `module`, `instance`, `transaction`; never combine
  them in one call.
- `composition_batch` is read-only (inspect/resolve/validate/source/context/impact), max 16 items;
  a failing item does not abort the others. Writes stay in prepare/apply transaction calls.
- Layout (`.designer/layout.toml`) and Canvas positions are not project semantics and are not
  exposed by MCP.
- Prefer one command per transaction for clear review; multiple commands are allowed and atomic.
- Report the Diff and validation result to the human; do not claim success without `accepted=true`
  and a valid post-apply validation.

## Common failures

| Diagnostic | Meaning |
|---|---|
| `edit-invalid-command` | Bad command fields, duplicate ID, missing instance, delete root |
| `edit-revision-conflict` | Stale base revision or external file change |
| `edit-duplicate-request` | request_id reused with different payload |
| `contract-property-type-mismatch` | Config value has the wrong type |
| `structure-slot-cardinality` | Binding changes violate slot cardinality |
| `ref-instance-not-found` | Binding/connection refers to a missing instance |

## Demo

The repository contains a real MCP stdio end-to-end test:

```powershell
cargo test -p composition-mcp --test e2e -- --nocapture
```

It starts `composition-mcp --project <copy>`, resolves the project, prepares and applies a
`set_config` transaction, validates, and confirms the TOML source changed.
