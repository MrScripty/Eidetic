# crates/server/src

## Purpose
This directory contains Eidetic's backend runtime: persistence, command and
projection services, AI backend integration, realtime coordination, and legacy
Axum route adapters over the domain model in `eidetic-core`.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `lib.rs` | Backend runtime module root consumed by binaries, tests, and future desktop bindings. |
| `backend_task.rs` | Backend task supervisor for explicit desktop lifecycle ownership. |
| `routes/` | Legacy Axum route adapters for project, command, timeline, story, AI, export, and reference workflows while route tests migrate to service coverage. |
| `sqlite.rs` | Shared SQLite connection setup for write-capable project database access. |
| `persistence.rs` | SQLite project persistence and project listing. |
| `project_service.rs` | Host-neutral project create, load, save, update, and list behavior consumed by legacy routes and future Tauri commands. |
| `ai_service.rs` | Host-neutral AI status, config, context-preview, and child-plan generation behavior consumed by legacy routes and Tauri commands. |
| `ai_generation_service.rs` | Host-neutral streaming script generation and batch generation orchestration consumed by legacy routes and Tauri commands. |
| `ai_generation_runtime.rs` | Supervised AI generation runtime for streaming, status persistence, script block writes, and recap generation. |
| `model_service.rs` | Host-neutral Pumas model-list behavior consumed by legacy routes and Tauri commands. |
| `export_service.rs` | Host-neutral PDF export behavior consumed by legacy routes and Tauri commands. |
| `reference_service.rs` | Host-neutral reference document list/upload/delete behavior consumed by legacy routes and Tauri commands. |
| `command_service.rs` | Host-neutral command handlers shared by legacy routes and Tauri command adapters during transport migration. |
| `projection_service.rs` | Host-neutral projection readers shared by legacy routes and Tauri command adapters during transport migration. |
| `history_store.rs` | SQLite command, event, object revision, and field delta persistence for projection-owned state. |
| `history_store_tests.rs` | Focused history-store transaction, idempotency, and round-trip tests. |
| `bible_graph_schema.rs` | SQLite schema setup for story-bible graph node, part, and field current-state rows. |
| `bible_graph_store.rs` | Typed graph-node rows, canonical root initialization helpers, and detail/list projection reads for story-bible graph nodes. |
| `bible_graph_field_store.rs` | Typed graph part/field current-state writes and part/field projection loading. |
| `bible_graph_edge_store.rs` | Typed graph edge current-state writes and incoming/outgoing edge projection loading. |
| `bible_graph_store_tests.rs` | Focused graph persistence and projection-envelope tests. |
| `bible_graph_command.rs` | Validated story-bible graph node, canonical-root, field, and edge command handlers with transactional history writes. |
| `bible_graph_command_tests.rs` | Focused graph command tests for create, idempotency, conflicts, and validation behavior. |
| `object_field_command.rs` | Validated field update command handler over history storage and projection rebuilds. |
| `object_field_command_tests.rs` | Focused command-path tests for set, clear, duplicate, and validation behavior. |
| `revision_projection.rs` | Read-side field projection rebuilds from object revision history. |
| `revision_projection_tests.rs` | Focused projection rebuild tests over persisted history rows. |
| `ydoc.rs` | Yjs/Yrs document coordination and persistence serialization. |
| `ai_backends/` | Provider adapters for local and remote text generation backends. |

## Problem
The application needs backend-owned services that expose core behavior to the
desktop shell while remaining compatible with local persistence, streaming
updates, and temporary route-adapter tests during migration.

## Constraints
- Backend services are the durable source of truth for command and projection
  behavior.
- Axum routes are legacy test adapters only; production desktop transport uses
  Tauri commands and events instead of a local listener.
- Persistence and Yjs state must remain compatible with saved projects.
- `persistence.rs` and `ydoc.rs` are above decomposition thresholds tracked in `ADR-001`.

## Decision
Keep backend services, persistence, and realtime coordination in the server
crate. Route modules remain temporarily for behavior coverage while Milestone 7
replaces route tests with service and Tauri adapter tests; the standalone
listener, static host, and WebSocket host have been removed.

## Alternatives Rejected
- Embedding persistence and host logic in `eidetic-core`: rejected because it would make the core crate host-specific.
- Splitting persistence and Yjs modules during the standards pass: rejected because contract correctness took priority over structural churn.
- Embedding Axum in the Tauri desktop runtime: rejected by Milestone 7 because
  production desktop transport uses Tauri command/event contracts, not a
  loopback HTTP/WebSocket server.

## Invariants
- Legacy route-adapter semantics are centralized under `routes/`.
- New backend behavior must be added behind service APIs before being exposed
  through Axum or Tauri adapters.
- Saved project compatibility is preserved across persistence changes.
- Realtime document state and structural project state stay synchronized on load/save boundaries.

## Revisit Triggers
- Another persistence backend is introduced.
- Axum route behavior is not yet backed by a service-level command/projection
  API and needs to be exposed through Tauri.
- `persistence.rs` or `ydoc.rs` gains another unrelated concern.

## Dependencies
**Internal:** `eidetic-core`, `routes/`, `project_service.rs`, `command_service.rs`, `projection_service.rs`, `ai_backends/`, `sqlite.rs`, `history_store.rs`, `bible_graph_schema.rs`, `bible_graph_store.rs`, `bible_graph_field_store.rs`, `bible_graph_edge_store.rs`, `bible_graph_command.rs`, `object_field_command.rs`, `revision_projection.rs`.
**External:** `axum`, `tower`, `tokio`, `rusqlite`, `yrs`, `reqwest`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized server modules.

## API Consumer Contract
- Tauri command/event adapters consume backend services directly instead of
  calling Axum handlers.
- Remaining route handlers are temporary test adapters and must delegate to
  service APIs rather than owning behavior.
- Realtime event ordering must remain compatible with the desktop event
  client's single-owner lifecycle.

## Structured Producer Contract
- This directory produces persisted SQLite project data, Y.Doc blobs, and JSON payloads consumed by the UI.
- Refactor-era schema and payload changes are allowed to break old project data only when the projection architecture plan explicitly owns that deletion.
