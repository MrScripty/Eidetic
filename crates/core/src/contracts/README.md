# crates/core/src/contracts

## Purpose
This directory defines host-agnostic command, event, revision, and projection contracts used by the projection-architecture refactor.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | Public contract types, validated IDs, typed values, change history records, and projection envelopes. |
| `bible_graph.rs` | Canonical story-bible graph contracts, expected root nodes, typed graph parts/fields/edges, and node-detail projection shapes. |

## Problem
The new architecture needs stable types for backend-owned commands, event history, sparse object revisions, and read projections before persistence, routes, Svelte, or Bevy can implement their slices safely.

## Constraints
- Contracts must remain independent from HTTP, SQLite, Svelte, Bevy, Y.Doc, and AI backend implementations.
- Public wire shapes must be serializable and round-trip testable.
- Canonical queryable facts must remain typed instead of hidden inside arbitrary JSON.

## Decision
Start with a small core contract module that owns IDs, object kinds, field values, generic field-update commands, graph-node create/root-initialization/field/edge commands, change events, object revisions, projection envelopes, and the first story-bible graph detail/list read models. Later slices can add domain-specific command/projection payloads without changing runtime infrastructure first.

## Alternatives Rejected
- Defining contracts in server routes: rejected because route-owned contracts would couple persistence, UI, and Bevy to HTTP handlers.
- Defining contracts in TypeScript first: rejected because backend-owned state and validation must be authoritative.

## Invariants
- Contracts remain deterministic and host-agnostic.
- Long-lived boundary types have explicit serde shapes.
- Object revisions describe field-level deltas and do not require whole-object snapshots.
- Canonical bible roots are system-owned graph nodes, not enum-only branches in application logic.

## Revisit Triggers
- Contracts become public SDK or binding surface.
- A persistence slice needs additional field value types.
- Projection envelopes need cross-process delivery metadata beyond version and change event.
- Bible graph schemas require richer typed field constraints than the current field value primitives.

## Dependencies
**Internal:** None.
**External:** `serde`, `uuid`.

## Related ADRs
- `ADR-001` decomposition baseline.
- Refactor plan: `docs/refactors/eidetic-projection-architecture/final-plan.md`.

## Usage Examples
```rust
use eidetic_core::contracts::{ChangeEvent, ChangeEventKind, CommandId};

let event = ChangeEvent::new(CommandId::new(), ChangeEventKind::UserEdit, "edit script");
assert_eq!(event.summary, "edit script");
```

## API Consumer Contract
- These types are stable internal Rust contracts for command/event/projection slices.
- External API exposure must add explicit boundary validation and serialization round-trip tests in the consuming layer.
- Compatibility is not required for pre-refactor project data.

## Structured Producer Contract
- `ChangeEvent`, `ObjectRevision`, and `ObjectRevisionField` are intended to map directly to SQLite command/event/revision rows.
- `ProjectionEnvelope<T>` is a versioned read model wrapper for Svelte, Bevy, AI, and export projections.
- Field value variants define typed persistence semantics; adding variants requires persistence and wire round-trip tests.
