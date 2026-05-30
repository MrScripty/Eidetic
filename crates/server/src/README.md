# crates/server/src

## Purpose
This directory contains Eidetic's backend runtime: persistence, command and
projection services, AI backend integration, and realtime coordination over the
domain model in `eidetic-core`.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `lib.rs` | Backend runtime module root consumed by binaries, tests, and future desktop bindings. |
| `backend_task.rs` | Backend task supervisor for explicit desktop lifecycle ownership. |
| `sqlite.rs` | Shared SQLite connection setup for write-capable project database access. |
| `persistence.rs` | SQLite project persistence and project listing. |
| `project_service.rs` | Host-neutral project create, load, save, update, and list behavior consumed by Tauri commands. |
| `ai_service.rs` | Host-neutral AI status, config, context-preview, and child-plan generation behavior consumed by Tauri commands. |
| `ai_generation_service.rs` | Host-neutral streaming script generation and batch generation orchestration consumed by Tauri commands. |
| `ai_generation_runtime.rs` | Supervised AI generation runtime for streaming, status persistence, script block writes, and recap generation. |
| `affect_service.rs` | Host-neutral affect command/projection behavior over backend-owned affect storage. |
| `model_service.rs` | Host-neutral Pumas model-list behavior consumed by Tauri commands. |
| `model_endpoint_resolver.rs` | Backend-owned llama.cpp OpenAI endpoint policy and Pumas runtime-profile resolution for live provider workflows. |
| `agent_structured_tool_provider.rs` | Provider-independent structured JSON tool loop for text-only model providers. |
| `agent_premise_workflow.rs` | First premise graph-context workflow slice over backend graph reads, reviewable proposals, and harness history. |
| `export_service.rs` | Host-neutral PDF export behavior consumed by Tauri commands. |
| `reference_service.rs` | Host-neutral reference document list/upload/delete behavior consumed by Tauri commands. |
| `affect_store.rs` | SQLite affect value, dependency, and proposal persistence with revision-history writes. |
| `command_service.rs` | Host-neutral command handlers consumed by Tauri command adapters. |
| `projection_service.rs` | Host-neutral projection readers consumed by Tauri command adapters. |
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
desktop shell while remaining compatible with local persistence and streaming
updates.

## Constraints
- Backend services are the durable source of truth for command and projection
  behavior.
- Production desktop transport uses Tauri commands and events instead of a
  local HTTP/WebSocket listener.
- Persistence and Yjs state must remain compatible with saved projects.
- `persistence.rs` and `ydoc.rs` are above decomposition thresholds tracked in `ADR-001`.

## Decision
Keep backend services, persistence, and realtime coordination in the server
crate. Milestone 7 removed the standalone listener, static host, WebSocket host,
Axum route adapters, and route tests after the desktop frontend moved to Tauri
commands/events and service-level tests covered backend behavior.

### Complection Review

`affect_store.rs` is dense but currently coherent: schema setup, command
recording, row mapping, revision generation, target encoding, and proposal
status transitions all protect the same SQLite transaction invariant. A reader
changing affect persistence must reason about the persisted row shape and the
history revisions together.

The next useful boundary is not a line-count split. If affect targets or
proposal lifecycle rules grow independently, extract target/endpoint encoding
or proposal status transitions behind a small store-local contract. Until then,
splitting SQL fragments, row mapping, and revisions into separate files would
increase coupling by hiding the transaction invariant.

## Alternatives Rejected
- Embedding persistence and host logic in `eidetic-core`: rejected because it would make the core crate host-specific.
- Splitting persistence and Yjs modules during the standards pass: rejected because contract correctness took priority over structural churn.
- Embedding Axum in the Tauri desktop runtime: rejected by Milestone 7 because
  production desktop transport uses Tauri command/event contracts, not a
  loopback HTTP/WebSocket server.
- Splitting `affect_store.rs` solely by size: rejected because the current
  coupling is a transaction/revision invariant rather than unrelated ownership.

## Invariants
- New backend behavior must be added behind service APIs before being exposed
  through Tauri adapters.
- Saved project compatibility is preserved across persistence changes.
- Realtime document state and structural project state stay synchronized on load/save boundaries.

## Revisit Triggers
- Another persistence backend is introduced.
- A desktop command needs behavior that is not yet backed by a service-level
  command/projection API.
- `persistence.rs` or `ydoc.rs` gains another unrelated concern.

## Dependencies
**Internal:** `eidetic-core`, `project_service.rs`, `command_service.rs`, `projection_service.rs`, `ai_backends/`, `sqlite.rs`, `history_store.rs`, `bible_graph_schema.rs`, `bible_graph_store.rs`, `bible_graph_field_store.rs`, `bible_graph_edge_store.rs`, `bible_graph_command.rs`, `object_field_command.rs`, `revision_projection.rs`.
**External:** `tokio`, `rusqlite`, `yrs`, `reqwest`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized server modules.

## Usage Examples
```rust
use eidetic_server::project_service;

let result = project_service::list_projects();
assert!(result.is_ok());
```

## API Consumer Contract
- Tauri command/event adapters consume backend services directly instead of
  calling Axum handlers.
- Realtime event ordering must remain compatible with the desktop event
  client's single-owner lifecycle.

## Structured Producer Contract
- This directory produces persisted SQLite project data, Y.Doc blobs, and JSON payloads consumed by the UI.
- Refactor-era schema and payload changes are allowed to break old project data only when the projection architecture plan explicitly owns that deletion.
