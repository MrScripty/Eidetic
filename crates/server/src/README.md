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
| `ai_temporal_context.rs` | Deterministic per-field fictional-time resolution before prompt construction; excludes future assertions and rejects same-time conflicts. |
| `ai_temporal_context_tests.rs` | Sparse inheritance, ordering, conflict, duplicate and cleared-value temporal regressions. |
| `ai_generation_service.rs` | Host-neutral streaming script generation and batch generation orchestration consumed by Tauri commands. |
| `ai_generation_runtime.rs` | Supervised AI generation runtime for streaming, status persistence, script block writes, and recap generation. |
| `affect_service.rs` | Host-neutral affect command/projection behavior over backend-owned affect storage. |
| `model_service.rs` | Host-neutral Pumas model-list behavior consumed by Tauri commands. |
| `model_endpoint_resolver.rs` | Backend-owned llama.cpp OpenAI endpoint policy and Pumas runtime-profile resolution for live provider workflows. |
| `agent_structured_tool_provider.rs` | Provider-independent structured JSON tool loop for text-only model providers. |
| `agent_workflow_harness.rs` | Bounded agent execution with persisted intent, terminal outcomes and cooperative cancellation. |
| `agent_workflow_harness_tests.rs` | Provider, manifest, executor, cancellation and exact-budget lifecycle regressions against SQLite history. |
| `agent_premise_workflow.rs` | First premise graph-context workflow slice over backend graph reads, reviewable proposals, and harness history. |
| `export_service.rs` | Host-neutral PDF export behavior consumed by Tauri commands. |
| `reference_service.rs` | Canonical reference list/upload/delete and source-bound asynchronous embedding publication. |
| `embeddings.rs` | Transitional HTTP embedding adapter validating returned model, input index and finite nonzero vectors. |
| `vector_store.rs` | Disposable exact-source index with publication tickets, representation-space filtering and stable ranking. |
| `vector_store_tests.rs` | Source custody, stale lifecycle, incompatible representation and numeric ranking regressions. |
| `reference_retrieval_tests.rs` | Actual RAG attach-boundary regression coverage for queued/reopened sessions and late query results. |
| `affect_store.rs` | SQLite affect value, dependency, and proposal persistence with revision-history writes. |
| `command_service.rs` | Host-neutral command handlers consumed by Tauri command adapters. |
| `projection_service.rs` | Host-neutral projection readers consumed by Tauri command adapters. |
| `timeline_command_guard.rs` | Transaction-local timeline snapshot validation shared by all timeline history writers. |
| `timeline_range_history_tests.rs` | Descendant range delta, replay ordering, no-op and atomic stale-edit regressions. |
| `timeline_command_guard_tests.rs` | Stale edit, atomic rollback, replay and refresh regressions across shared timeline commands. |
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
The M3a retrieval slice in `docs/plans/agent-story-workflows/plan.md` retains the
existing index and canonical project owner. It binds derived vectors to exact
source and configured representation identity, invalidates asynchronous work at
replacement/deletion boundaries, and treats mismatches as unavailable retrieval.
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
- Reference embeddings are disposable derived state. Exact source snapshots and
  project-index/document tickets fence async publication; project create/load and
  reference deletion invalidate pending work. Admission binds single and batch
  requests before project snapshot I/O; reopening the same path cannot rebind
  queued retrieval. Create/load publish path, project and index under the project
  guard. Source checks happen again before
  ranking under project-then-index lock order. No lock crosses model I/O.
- Only nonempty finite nonzero vectors with matching configured endpoint/model
  and dimension are ranked. Returned HTTP model identity must match the request.
  This does not attest immutable model weights; Pumas revision-aware embeddings,
  live runtime qualification and saved-reference re-indexing remain M3 work.
- Generation and preview accept an optional explicit fictional `story_time_ms`.
  Snapshot fields resolve independently at or before it. Conflicting values at
  the latest effective timestamp stop context construction. A cleared value is
  withheld as unknown until a later assertion. With no time scope, every field
  that has timed assertions is withheld, including its otherwise untimed default.
  These read-side rules do not rewrite canonical facts or edit history. Batch
  generation still has no clip-specific world-time mapping and withholds timed
  fields; a shared parent time is not silently assigned to child scenes.
  Future-only fields with no baseline retain an unknown marker before their
  first assertion. Temporal resolution applies to fields, not edges, and
  effective provenance is label/time rather than unique assertion identity.
- Agent intent is recorded before tool execution. Failed or cancelled runs have
  terminal history; failures never trigger automatic tool retries. Cooperative
  cancellation does not roll back commands already committed. If persistence
  itself fails at a terminal call, result or run write, the harness preserves the
  typed execution outcome alongside the persistence error. A cancelled execution
  stays Cancelled when its run can still be recorded; it does not become Failed
  because a call/result write failed. No failed persistence write is retried.
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

## Portable Test Resource Contracts

SQLite tests must close inspection connections before removing database files,
so cleanup verifies the same resource lifetime on Windows and Unix. Project-path
validation returns children under the canonical root when that root exists;
tests compare exact paths built from that canonical root and native components.
A missing root continues to use lexical normalization. Timestamp equality and
path escape-rejection assertions remain required.

### Shared timeline snapshot custody (M4a)

Timeline command services plan edits from a persisted project snapshot before
entering a blocking write task. All nine timeline history writers now validate
that snapshot against current SQLite nodes, node-arc membership, relationships
and total duration inside the same transaction that records the command/event/
revisions. Collection ordering is ignored; every serialized row field is compared.
A mismatch returns a conflict and rolls back history as well as current state.
The caller must reload and review the operation rather than blindly retrying.
A matching command-ID/payload replay remains idempotent even after later edits.

This deliberately conservative prerequisite rejects unrelated timeline edits
because existing writers upsert whole collections. It does not introduce a
second store, new schema, agent write authority, or automatic proposal acceptance.
It does not yet bind a long-lived proposal to an immutable revision: an
edit-and-revert can have equal current state, and command payloads still lack an
expected revision. Project-session custody across navigation, lock/subtree
semantics, agent tool/UI parity, undo, and generation/stream persistence remain
separate acceptance gates in the active story-workflow plan. The guard requires
an initialized canonical project database; it never seeds missing rows from a
possibly stale in-memory mirror.

Commands before the initial durable project save return an explicit conflict;
no fallback mirror is used to initialize current state inside a timeline edit.
The user can retry after persistence and reload.

### Complete parent-resize history (M4b)

A parent move/resize can change every descendant's range. The range command now
records the target and each actually changed descendant in one event/transaction,
including exact old/new start and end fields. Unchanged descendants and unrelated
clips get no new revision; the existing explicit no-op target record is retained.
Per-object history reads order committed events first, then within-event revisions.
Previously ordering only by per-event `sort_order` replayed a later direct child
edit before an earlier descendant edit. Caller timestamps are not a substitute
for committed event order. This repairs review/projection inputs; it does not
introduce or claim a completed undo UI, locked-subtree policy or agent write tool.

History ordering relies on the current append-only SQLite event table. Broad
save preserves history, load reopens it, and PDF export does not rebuild it.
There is no supported history import/compaction path in the audited source.
This is not a portable/global revision clock; future event-reinserting or
reordering maintenance requires an explicit ordering contract before adoption.
