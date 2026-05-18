# crates/server/src

## Purpose
This directory contains the local Axum host for Eidetic: route registration, persistence, AI backend integration, realtime synchronization, and static asset serving over the domain model in `eidetic-core`.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `main.rs` | Server bootstrap, routing, CORS policy, and static hosting. |
| `routes/` | HTTP handlers for project, timeline, story, AI, export, and reference workflows. |
| `persistence.rs` | SQLite project persistence and project listing. |
| `history_store.rs` | SQLite command, event, object revision, and field delta persistence for projection-owned state. |
| `history_store_tests.rs` | Focused history-store transaction, idempotency, and round-trip tests. |
| `revision_projection.rs` | Read-side field projection rebuilds from object revision history. |
| `revision_projection_tests.rs` | Focused projection rebuild tests over persisted history rows. |
| `ydoc.rs` | Yjs/Yrs document coordination and persistence serialization. |
| `ai_backends/` | Provider adapters for local and remote text generation backends. |
| `diffusion/` | Diffusion-model process management and Python bridge types. |

## Problem
The application needs a local host that exposes core behavior through a browser-friendly API while remaining compatible with local persistence and streaming updates.

## Constraints
- Server routes are the host-facing contract for the frontend.
- Persistence and Yjs state must remain compatible with saved projects.
- `persistence.rs` and `ydoc.rs` are above decomposition thresholds tracked in `ADR-001`.

## Decision
Keep transport, persistence, and realtime coordination in the server crate while documenting the planned splits for the oversized persistence and CRDT modules.

## Alternatives Rejected
- Embedding persistence and host logic in `eidetic-core`: rejected because it would make the core crate host-specific.
- Splitting persistence and Yjs modules during the standards pass: rejected because contract correctness took priority over structural churn.

## Invariants
- Host-facing route semantics are centralized under `routes/`.
- Saved project compatibility is preserved across persistence changes.
- Realtime document state and structural project state stay synchronized on load/save boundaries.

## Revisit Triggers
- Another persistence backend or transport host is introduced.
- `persistence.rs` or `ydoc.rs` gains another unrelated concern.

## Dependencies
**Internal:** `eidetic-core`, `routes/`, `ai_backends/`, `diffusion/`, `history_store.rs`, `revision_projection.rs`.
**External:** `axum`, `tower-http`, `tokio`, `rusqlite`, `yrs`, `pyo3`, `reqwest`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized server modules.

## Usage Examples
```rust
use crate::routes;
```

## API Consumer Contract
- The frontend is the primary consumer of the HTTP and WebSocket surfaces rooted here.
- Route handlers must return explicit HTTP status semantics and preserve the JSON field names consumed by `ui/src/lib/api.ts`.
- Realtime event ordering must remain compatible with the websocket client’s single-owner lifecycle.

## Structured Producer Contract
- This directory produces persisted SQLite project data, Y.Doc blobs, and JSON payloads consumed by the UI.
- Schema or payload changes must preserve compatibility with saved projects or ship with synchronized migrations and client updates.
