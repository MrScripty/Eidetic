# crates/server/src

## Purpose
This directory contains Eidetic's backend runtime: persistence, command and
projection services, AI backend integration, and realtime coordination over the
domain model in `eidetic-core`.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `lib.rs` | Backend runtime module root consumed by binaries, tests, and future desktop bindings. |
| `backend_task.rs` | Backend task supervisor that reaps observed completions during normal admission and owns unfinished work through shutdown. |
| `sqlite.rs` | Shared SQLite connection setup for write-capable project database access. |
| `persistence.rs` | SQLite project persistence and project listing. |
| `project_service.rs` | Host-neutral project lifecycle, with outgoing document persistence and committed source snapshots for Save As. |
| `ai_service.rs` | Host-neutral AI status, config, context-preview, and child-plan generation behavior consumed by Tauri commands. |
| `ai_script_context.rs` | Canonical screenplay evidence for shared preview/generation context, carrying exact block and segment write identities. |
| `script_generation_lineage.rs` | Atomic successful-generation records and existing semantic dependencies bound to consumed screenplay revisions. |
| `script_generation_lineage_tests.rs` | A-to-B/unrelated-C, edits during generation, atomic rollback, replay, source deletion, refreshed and unavailable binding regressions. |
| `script_impact_projection.rs` | Derived Needs review causes against the latest successful generation, retaining historical source excerpts after deletion. |
| `script_impact_review.rs` | Proven-cause capture, revision-bound propagation proposal recording and atomic targeted acceptance with refreshed lineage. |
| `script_impact_review_tests.rs` | Linked-scene preview/reject/accept, stale/ABA, graph-time, lock, replay, target-scope and rollback fixtures. |
| `script_impact_review_guard_tests.rs` | Resolved-world context, explicit fictional time and late span/content regeneration lock guards. |
| `script_impact_review_service.rs` | Desktop-facing preview service using the configured provider without committing screenplay output. |
| `script_impact_review_service_tests.rs` | Native AppState preview regressions for truncated HTTP refusal and complete split-SSE proposal text. |
| `script_impact_prompt.rs` | Targeted screenplay prompt from captured canonical inputs and resolved graph context, with strict complete-stream collection. |
| `script_impact_prompt_tests.rs` | Deterministic provider boundary and partial/error/empty-output refusal fixtures. |
| `script_block_edit.rs` | Manual text-only block edits with expected-revision and transaction-local lock validation, preserving server-owned placement and metadata. |
| `script_block_edit_tests.rs` | Save/reopen/history/context, stale/ABA refusal, lock refusal and bounded continuity source regressions. |
| `manual_script_workflow_tests.rs` | Native AppState preview and shared generation-admission regression with a deliberately stale project mirror and no provider call. |
| `ai_temporal_context.rs` | Deterministic per-field fictional-time resolution before prompt construction; excludes future assertions and rejects same-time conflicts. |
| `ai_temporal_context_tests.rs` | Sparse inheritance, ordering, conflict, duplicate and cleared-value temporal regressions. |
| `ai_generation_service.rs` | Host-neutral streaming script generation and batch generation orchestration consumed by Tauri commands. |
| `ai_generation_runtime.rs` | Supervised AI generation runtime for streaming, status persistence, script block writes, and recap generation. |
| `ai_generation_stream_tests.rs` | Provider-independent stream error, successful EOF, empty output and progress regressions. |
| `ai_generation_runtime_tests.rs` | Native AppState/SQLite regressions for failed-stream lineage preservation, generation cleanup and captured-input persistence after an intervening edit. |
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
| `command_service_timeline.rs` | Timeline command services with source admission and an owned session gate through document/event/save publication. |
| `timeline_command_admission_tests.rs` | Deterministic blocked-load tests for fallback and create-child database ownership across project replacement. |
| `timeline_postcommit_custody_tests.rs` | Queued session/save-as conflicts, bounded document-channel publication, errors and seeded A/B history regressions. |
| `state_autosave_custody_tests.rs` | Autosave memento ownership across mirror/path/document serialization and persistence. |
| `projection_service.rs` | Host-neutral projection readers consumed by Tauri command adapters. |
| `timeline_command_guard.rs` | Transaction-local timeline snapshot validation shared by all timeline history writers. |
| `timeline_range_history_tests.rs` | Descendant range delta, replay ordering, WAL interleaving and interrupted-transaction rollback regressions. |
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
- A Needs review preview targets the generated output block of a proven current
  cause. Capture target text/placement events, current screenplay inputs, cause
  and resolved graph context in one read snapshot. Include a moved source even
  outside the ordinary continuity window; retain deleted-source explanation.
- The targeted prompt consumes only its captured screenplay and resolved graph
  evidence. It shares screenplay formatting and the existing temporal resolver;
  it neither includes unbound timeline prose nor treats placement as fictional
  time. The UI supplies no fictional time, so timed world facts remain unresolved;
  the typed request also supports an explicit query coordinate.
- Preview persists an existing propagation proposal plus additive typed binding
  storage, not screenplay changes. Partial failed/empty streams create no draft.
  Request replay returns existing review state without another provider call.
- Bound acceptance rechecks source/target revisions, cause, graph evidence,
  pending proposal contents, span locks and the source node's regeneration lock
  after acquiring the SQLite writer lock. Accepted text/span, proposal status,
  sparse history and refreshed actual-input lineage commit atomically. Preserve
  document/segment metadata, other blocks and the writer's source edit.
- Bound previews cannot be amended/retargeted through the generic update command;
  reject and request a fresh preview. Stale refusal leaves the pending proposal
  and authored text available. Rejection changes only proposal review history.
  Inferred world assertions remain separate proposals; this flow writes no facts.
- Successful generated output, generation record, revision-bound semantic
  dependencies and sparse history commit in one SQLite transaction. Stream
  failure/empty output creates no generation lineage. Validate captured historical
  text and placement without rebinding to current inputs after model I/O.
- Generation replay covers the complete input evidence and cannot overwrite a
  later output or duplicate lineage. Additive tables preserve unbound legacy edges.
- Script projection payload, review causes and version share one read snapshot.
  Only the latest successful generation's bindings define review causes; old
  bindings remain audit history. Intentional replacement of the consumer's own
  prior draft does not flag itself. Missing input history is explicitly unavailable.
- Review impact preserves authored text, locks, canonical segment status and
  explicit proposal acceptance. It neither infers semantic relationships beyond
  actual supplied screenplay context nor recursively marks unrelated segments.
- Manual script edits change only canonical block text and its user-edited span.
  Clients supply an expected block write-event identity; the transaction checks
  it and current locks before mutation. Failure rolls back command/history rows.
  Replay preserves intervening edits. Document/segment metadata stays backend-owned.
- Preview and single/batch script generation hydrate the same main-document
  screenplay context: target/intersecting segments plus up to two preceding and
  two following segments in presentation order. Every block carries its source
  segment/block IDs and their exact write events. These coordinates do not resolve
  fictional time. Legacy node script text and unbound recaps are omitted from this
  path. This slice does not infer or accept world updates or mark dependent scripts stale.
- Timeline command admission captures the database path and fallback project
  mirror under the project guard before loading persisted state. Failed loads
  cannot read a later session's mirror; derived create-child commands reuse the
  original admission instead of resolving the active database a second time.
- Participating timeline commands, project create/load/save and autosave share an
  async session gate. Timeline requests capture a session identity before waiting;
  replacement/reopen or save-as invalidates queued requests explicitly. Admitted
  work retains its gate through document enqueue, event and save publication.
  Autosave keeps mirror, path and serialized document in that gate through I/O.
- Admitted timeline/lifecycle work moves its owned gate into a supervised task;
  caller cancellation does not release it during blocking persistence or document
  publication. Waiting for admission stays cancellable. Backend shutdown aborts
  these tasks; this is not crash recovery or a guarantee for other producers.
- Normal supervisor spawn reaps finished task handles and observes their join
  results, including named panic reporting, before records accumulate across
  command history. Unfinished handles remain owned; shutdown still aborts and joins
  them. A completed tail may remain until the next spawn/count/shutdown, so record
  retention follows outstanding work and recent completions rather than total edits.
  Smoke inspection is not required to run cleanup. Passive test measurements do
  not prune the registry.
- Create/load flush the outgoing session directly before replacing its document.
  Save As flushes/reloads the source database, preserving committed timeline/arcs
  over its stale mirror, and copies that snapshot with the current document blob.
  Serialization/write/read failures prevent active-session publication. Autosave
  skips writes when serialization fails, preserving the last stored document.
  A different existing Save As destination returns a conflict to avoid combining
  its timeline/history with the source. Save As copies a current-state snapshot,
  not source command history, affect stores or external collaboration sessions.
- Y.Doc load restores into a fresh document rather than merging project lifetimes.
  Empty/fallback population starts fresh; invalid blobs preserve the current doc
  until an explicit fallback. Closed channels return errors. A document-channel
  failure after SQLite commit reports that the command is already committed;
  it does not claim SQL rollback or silently drop a full-channel write.
- This custody gate is scoped to participating timeline/lifecycle operations.
  Other producers, pre-admission frontend intent tokens, model/generation custody
  and external collaborative client resynchronization are not qualified here.
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
- A generation stream error is terminal even after progress tokens were emitted.
  Its accumulated prefix never reaches successful script persistence or completion
  events. Existing failure cleanup reports the provider error and releases the
  generating marker; prior script blocks and their revision history remain intact.
  Only successful EOF with nonempty text can reach the success path. Successful
  empty EOF remains the distinct "AI produced no output" outcome. This does not
  qualify project-switch custody, successful persistence atomicity or live models.
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

Range-history regressions also exercise two connections to a real saved WAL
project: a held reader snapshot cannot upgrade after another writer commits,
and a stale command cannot overwrite that commit. Temporary test-only triggers
interrupt descendant updates after history and ancestor writes, covering both
statement abort with transaction-drop rollback and SQLite transaction rollback.
Reopen checks retain prior history and Y.Doc bytes; failed attempts leave command
identities available for explicit resubmission. These are deterministic database
interleavings and injected SQL failures, not process-kill or power-loss tests.
