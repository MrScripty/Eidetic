# src-tauri/src/projections

## Purpose

This directory contains Tauri projection adapters that expose backend read
models to the desktop UI.

## Contents

| File | Description |
| ---- | ----------- |
| `mod.rs` | Projection module exports and registration surface. |
| `timeline.rs` | Timeline projection adapters. |
| `bible.rs` | Story-bible projection adapters. |
| `semantic.rs` | Semantic proposal projection adapters. |
| `affect.rs` | Affect projection adapters. |
| `context.rs` | Context stack and influence projection adapters. |
| `story_script.rs` | Story and script projection adapters. |

## Problem

The UI needs read models that reflect backend state without learning SQLite,
history replay, or domain projection rules.

## Constraints

- Projection adapters must stay read-only.
- Backend services own projection construction and envelope versioning.
- Payload shape changes must be synchronized with TypeScript contract files.

## Decision

Keep desktop projection adapters separate from mutating commands and delegate to
backend projection services.

## Alternatives Rejected

- Rebuilding projections in Svelte: rejected because backend history and
  validation are authoritative.
- Combining commands and projections in one adapter module: rejected because
  read and write paths have different invariants.

## Invariants

- Projection adapters do not mutate project state.
- Projection envelopes preserve backend-owned version and revision semantics.
- Missing project/session state returns explicit errors.

## Revisit Triggers

- Projection schemas are generated from Rust types.
- Frontend caching moves to a different event model.
- Another host needs shared projection adapters.

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

- Frontend callers receive backend-owned projection DTOs and should not infer
  persistence details from adapter internals.
- Errors must distinguish missing state from rejected projection requests.
- Compatibility-sensitive shape changes require synchronized UI updates.

## Structured Producer Contract

- Projection payloads are JSON-compatible Rust structures consumed by Svelte
  stores and renderer command bridges.
- Envelope versioning must remain meaningful for cache invalidation.
