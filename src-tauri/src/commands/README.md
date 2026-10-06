# src-tauri/src/commands

## Purpose

This directory contains Tauri command adapters that expose backend service
operations to the desktop UI.

## Contents

| File | Description |
| ---- | ----------- |
| `mod.rs` | Command module exports and registration surface. |
| `timeline.rs` | Timeline command adapters. |
| `bible.rs` | Story-bible command adapters. |
| `semantic.rs` | Semantic proposal command adapters. |
| `affect.rs` | Affect command adapters. |
| `context.rs` | Context influence command adapters. |
| `object_script_story.rs` | Object, script, and story command adapters. |

## Problem

The UI needs desktop-callable commands without duplicating backend validation,
history writes, or projection rebuild policy.

## Constraints

- Command adapters must remain thin.
- Backend services own validation, persistence, and idempotency.
- Payloads must stay serializable for Tauri invoke boundaries.

## Decision

Keep Tauri commands grouped by product domain and delegate immediately to
backend services or command services.

## Alternatives Rejected

- One monolithic command file: rejected because domain-specific command groups
  are easier to review.
- Frontend-owned validation only: rejected because commands mutate durable
  backend state.

## Invariants
- `command_script_block_create` forwards captured source placement, kind and
  exact text to the host-neutral creation service; IDs, append ordering, history
  and project-session admission remain backend-owned.
- `command_script_impact_proposal_preview` delegates proven-cause capture and
  provider drafting to the backend. It creates a pending propagation proposal;
  existing accept/reject commands exclusively own the writer's review decision.
- `command_script_block_edit` forwards text and expected block revision to the
  backend service; canonical metadata, locks and history remain backend-owned.

- A command adapter must not write persistence directly.
- Errors crossing the Tauri boundary must be deliberate and stable enough for
  frontend handling.
- New backend mutations need service-level tests before adapter exposure.

## Revisit Triggers

- Commands are generated from shared schemas.
- Error payloads become structured instead of string-based.
- Another host consumes the same adapter layer.

## Dependencies

**Internal:** `eidetic-server`, `eidetic-core`, `crate::state`.
**External:** `tauri`, `serde`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```rust
// Registered by the desktop builder and called from TypeScript through invoke.
```

## API Consumer Contract

- Frontend callers provide validated command DTOs from shared TypeScript
  contract files.
- Adapters return backend projection payloads or errors without hiding command
  rejection reasons.
- Compatibility changes require coordinated frontend API updates.

## Structured Producer Contract

- Command results are structured JSON payloads consumed by UI stores and tests.
- New fields should be additive unless the active plan explicitly owns a
  breaking contract change.

`command_bible_graph_edge_label` accepts the core label-only command envelope,
including the expected owned edge revision. It forwards to the shared Bible
command service; transaction validation and revision recording stay in the
server. The desktop boundary adds no cached graph preimage or replacement state.

Its label-only response returns `projection: null` for `already_recorded` replay,
so a deleted source need not be projected. Fresh writes still return source node
detail. The adapter forwards this optional projection without inventing one.
