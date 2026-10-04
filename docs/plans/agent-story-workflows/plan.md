# Agent-assisted story workflows

**Status:** Active
**Current phase:** M4a shared timeline write-conflict boundary; M1/M2a merged.
**Next gate:** Qualify and independently review shared timeline custody before
agent write exposure. Pumas runtime/revision integration remains open.

**Admission:** Continue the user-requested Eidetic story-workflow development on
`feat/agent-story-workflows`; baseline `ab5d75ce138c1b8e25cb5effc8f8800c0c6f9841`.

## Objective and acceptance

Writers can develop a feature-length screenplay by conversing with Eidetic's
agent, manually editing contextual timeline clips and the bible, or combining
both. The agent uses the graph as structured memory. The optional 3D graph is
an inspect/edit surface, not a prerequisite for writing. Existing editing,
SQLite projects, projections, and native renderer capabilities remain supported.

Acceptance claims (all pending unless the ledger records scoped evidence):

1. Both manual and agent operations create, move, cut, position, resize, and
   delete clips through the same validated commands, transactions and history.
   Stale proposals cannot overwrite intervening human changes.
2. Canonical facts supplied to generation are deterministically resolved for
   explicit story time before prompting. Narrative placement, fictional time,
   and edit history are distinct. Flashbacks do not silently use screen time.
3. Pumas supplies verified chat/runtime and embedding capabilities. Retrieval
   candidates retain source revision, model identity and dimension; stale or
   incompatible vectors cannot become canonical facts or silently participate
   in similarity ranking.
4. The existing Eidetic-owned agent harness supports bounded, inspectable
   conversational runs. Provider, malformed-call, tool and cancellation outcomes
   are visible; completion is not inferred from a partially failed execution.
5. A multi-act, feature-length manual/agent screenplay walkthrough demonstrates
   continuity and separate screenplay output, recovery and undo, repeated and
   interrupted interactions, with graph visualization closed as well as open.

## Binding decisions and ownership

- SQLite commands/events/revisions remain authoritative. No graph database
  migration or duplicate agent-owned story store.
- Core contracts own meaning and invariants; backend services own validation,
  persistence and projection; Tauri owns lifecycle/transport; Svelte and Bevy
  consume projections and issue commands.
- Reuse the existing harness, tool manifest, graph tools and Pumas endpoint
  resolver. Extend explicit domain operations rather than exposing raw SQL or
  asking the model to reconstruct repository-internal mechanics.
- Read/propose/apply are separate capabilities. A proposal or returned next
  operation is not authorization. Command replay and exact source revision
  checks belong to the backend, not prompts.
- Timed fact resolution must not treat every snapshot as simultaneously true.
  Existing `at_ms` snapshots require an explicit compatibility decision when
  independent world time is introduced. Do not guess that mapping.
- Embeddings are derived retrieval indexes, never canon or a truth/confidence
  score. Manual authoring remains usable without a running model.
- Agent-interface design follows Coding-Standards' engine-owned validation,
  compact authoritative results, explicit unresolved states and bound workflow
  context guidance. The application owns mechanics; the model supplies intent.

## Milestones and gates

| Milestone | State | Write set | Gate |
| --- | --- | --- | --- |
| M1: Reliable existing harness lifecycle | Accepted (bounded backend scope) | server harness and focused tests; this plan/index | Provider/contract/tool failures persisted; rejected calls never execute; prior successful calls preserved |
| M2: Explicit temporal fact projection | Active (M2a) | core time contracts; snapshot/query commands; context/prompt/tests | Future facts excluded; same-time conflicts explicit; sparse facts inherited; flashback and edit-history distinctions tested |
| M3: Pumas embeddings and inference | Planned | adapters/configuration; derived index; provider integration/tests | Actual pinned Pumas APIs; model/dimension/revision invalidation; unavailable model and malformed outputs fail visibly |
| M4: Shared manual/agent timeline editing | Active (M4a) | timeline command contracts/services; agent tools; Tauri/UI bindings/tests | Create/move/cut/resize/delete parity, containment/lock validation, replay, undo, stale proposals |
| M5: Conversational authoring and acceptance | Planned | agent service/UI; native interaction gaps; documentation | Multi-act screenplay walkthrough and complete applicable repository verification |

## Composed-design review

Applicable: this objective composes durable memory, inference, editing and UI.

1. **Concerns and dimensions:** Writers/agents author clips and facts on demand;
   the backend validates and commits them in local SQLite. Runtime adapters
   perform bounded inference when requested. Renderers display projections.
   Story time determines fictional truth; edit time orders durable changes.
2. **Interleavings:** Generation reads a revision-bound context and may complete
   after manual edits or navigation. Apply must revalidate current identity and
   revision. Model identity/dimension qualify derived vectors independently of
   canonical facts. Renderer visibility is not a graph-truth filter.
3. **Caller knowledge:** Callers supply explicit intent, object identity, expected
   revision and requested time scope. They do not need SQL, revision assembly,
   provider boot mechanics or graph-renderer state to perform story operations.
   Tauri composes service lifecycle and transport only.
4. **Representative changes:** New time semantics affect core contract, query
   owner and its UI projection; provider changes affect the Pumas adapter and
   configuration, not canonical storage; a new clip edit affects its domain
   command and both command-consuming surfaces, not inference policy.
5. **Dependencies:** Core IDs/contracts carry stable meaning. Pumas API versions
   and runtime availability are adapter-owned. SQLite serialization/history is
   store-owned; no consumer may reconstruct it from renderer internals.
6. **Independent failure/replacement:** Inference can fail without disabling
   manual authoring. Rendering can close without losing canonical graph state.
   Focused tests cover these boundaries; the final desktop walkthrough verifies
   the composed artifact, including recovery and concurrent edits.
7. **Deletion test:** Reuse existing harness, store and renderer boundaries.
   Removing temporal resolution would force every prompt consumer to guess
   canon; removing embedding identity checks would allow incompatible retrieval.
   No new database, process or general workflow framework is justified. Each
   future new permanent mechanism needs a concrete ownership/deletion rationale.
8. **Retained complexity:** Distinct world/narrative/edit times, concurrency and
   inference availability are inherent. Keep them in typed contracts, command
   validation and lifecycle adapters. Do not duplicate those rules in prompts,
   UI stores, or a second agent-specific story database.

## M2a scope and compatibility

- Exact write set: core AI-context contract/export/README; server context loader,
  temporal resolver/tests, prompt builder, generation/context services and README;
  Tauri AI adapter/README; UI AI API helpers/tests/README; these plan records.
- Resolve current canonical snapshots per field using explicit caller-provided
  fictional milliseconds. Existing snapshot coordinates are retained without
  migration; no implicit relationship to clip screen position is introduced.
- Requests lacking time remain supported, but potentially time-varying fields
  are now explicitly withheld. At an explicit time, untimed values provide the
  baseline before the first assertion, sparse snapshots inherit prior fields,
  and a null assertion withholds its field until a later assertion. This is a
  conservative read-side compatibility rule, not a rewrite of stored values.
- Latest equal-time conflicting assertions return a conflict before inference.
  Identical assertions coalesce deterministically; later facts can supersede an
  earlier conflict. Future and overridden values do not reach the prompt. A
  known future-only field without a baseline remains explicitly unknown before
  its first assertion, even when other fields on the same node resolve.
- This temporal slice resolves fields only. Graph edges remain untimed and
  prompts state that limitation. Effective assertion provenance currently carries
  label/time, not unique assertion identity; stronger provenance is still open.
- This slice exposes service/Tauri/client-helper input, not a persistent clip
  world-time editor or world-calendar/POV/branch model. Existing editor buttons
  still omit time and receive explicit unknowns. Batch generation remains
  unscoped rather than assigning one fictional time to every scene.
- Acceptance: core/server regressions prove prompt fact exclusion, sparse
  inheritance, conflict handling, clear semantics, and input-order independence;
  frontend IPC tests prove explicit values/zero pass through, absent input remains
  absent and invalid/lossy numeric inputs never invoke Tauri. Native desktop
  runtime and persistent per-clip mapping remain later M2/M5 acceptance.

## Constraints, blockers and re-plan triggers

- Cloud environment has limited disk; avoid full native renderer builds until
  toolchain/dependency resolution and budget are established. Focused checks do
  not substitute for the canonical launcher acceptance suite.
- Current Cargo manifest uses an absolute-layout-relative Pumas source path;
  determine reproducible dependency setup before changing dependency identity.
- Actual Pumas embedding APIs exist in the inspected Pumas source, but live
  runtime/model availability and compatibility still need verification.
- Existing user-local presentation assets are unrelated and outside the write
  set. Do not modify or publish them.
- Replan if public contracts, migration semantics, or Pumas API capabilities
  contradict a milestone; retain any unmet acceptance claim explicitly.
- Publication requires focused review/tests and the user's branch/review flow.
  A draft PR is not authority to merge.

## Records

- [Execution ledger](execution-ledger.md)
- [Issues](issues.md)
- Existing architecture: `docs/refactors/eidetic-projection-architecture/final-plan.md`
- Existing screenplay model: `docs/plans/script-generation-model/plan.md`

## M3a: source-bound reference retrieval

This bounded prerequisite starts from merged main `3757edf6e4050aa211cb6ac7d9f991167c2af74e`.
It does not complete M3's Pumas runtime or immutable model-revision acceptance.

- **Observed failure:** Upload embeds asynchronously, while deletion removes only
  currently present vectors. A late completion could resurrect deleted content.
  The index was also retained across project replacement, and incompatible vector
  dimensions were ranked as zero rather than excluded.
- **Ownership:** The existing disposable `VectorStore` owns index lifetime and
  document publication tickets. The project remains the canonical reference
  source. No new persistent store, framework, configuration format or dependency.
- **Source binding:** Register an exact source snapshot before embedding. A ticket
  binds project-index lifetime, document ID and indexing revision. Deletion,
  replacement and project create/load invalidate earlier tickets. Insertion and
  search compare the current canonical source's ID, name, content and type.
- **Concurrency:** Project and index locks are always acquired in that order;
  publication and deletion hold both in the same synchronous critical section.
  No guard crosses model I/O. Single/batch admission captures the index epoch
  before project snapshot I/O and carries it into queued generation. Create/load
  publish path, project and index under the project guard. An in-flight query
  cannot select from a replacement index, including reopening the same path.
  Every attach attempt first clears prior retrieval context.
- **Representation checks:** The transitional HTTP adapter requires exactly one
  indexed result for its one input and an exact returned model match. Nonempty,
  finite, nonzero vectors are validated before use. Endpoint/model/dimension
  mismatches are filtered before ranking. Float64 accumulation avoids finite
  float32 overflow/underflow; ties use stable chunk IDs.
- **Honest boundaries:** A configured model name does not attest immutable weights
  or revision. Aliased model replacements remain a Pumas M3 integration concern.
  Missing/mismatched model metadata fails closed instead of silently accepting an
  unidentified embedding. HTTP transport remains transitional; no live model,
  Pumas inference, UI status surface or restart re-indexing is claimed here.
- **Failure:** Upload remains successful if canonical storage succeeds and model
  inference fails. The task logs indexed/failed/discarded counts, rather than
  claiming embedding success after all chunks failed. Generation can proceed
  without optional retrieval; query embedding failure is explicitly logged.
- **Write set:** Server embedding adapter, vector store/tests, reference service,
  project replacement boundaries, generation admission/RAG attachment and caller
  tests, source README and plan records.
- **Verification:** Regression tests cover late completion after deletion,
  source change, index replacement, stale query scope, model/endpoint/dimension
  mismatches, malformed outputs, numeric extremes and deterministic top-k. Run
  focused core/server tests and clippy locally; hosted workspace and frontend
  checks must qualify the exact submitted head. Desktop/live-model checks remain
  unrun unless recorded separately.

Design sources: *Knowledge Graphs and Agentic Memory*, chapter 6 (candidate
retrieval and representation-space compatibility) and chapter 11 (exact
source/version binding and explicit unavailable states); Coding-Standards
`CORE-STANDARDS.md` (derived artifacts, lifecycle and authority boundaries) and
`docs/plans/agent-interface-efficiency/plan.md` (preserve exact source authority
while simplifying transport). The present slice follows those principles without
claiming to expose a completed agent retrieval workflow.


## M4a: shared timeline write-conflict boundary

- **Observed defect:** Services load a project before `spawn_blocking`; timeline
  history writers then upsert full node collections from that snapshot. An
  intervening manual edit can be overwritten by a later command even on another
  clip. A deleted row can be resurrected by an old collection.
- **Owner and write set:** Existing server timeline history writers plus one
  shared transaction-local validator, regressions, source README and this ledger.
  No transport contract, schema, dependency identity or retrieval code changes.
- **Boundary:** Compare persisted nodes, node-arc memberships, relationships and
  total duration to the planned snapshot in the write transaction. Comparison
  ignores only outer collection order. Reject mismatches as conflicts, atomically
  rolling back the command, event and revisions. Preserve exact replay behavior.
- **Alternatives:** Blind whole-snapshot overwrites violate backend ownership.
  Retrying without a refreshed/reviewed intent could apply an unwanted operation.
  A new agent-only store or writer would duplicate authority. Per-object revision
  preconditions remain the next protocol slice rather than being faked by an
  in-memory token or timestamp.
- **Limits:** This is state-equality conflict detection, not immutable proposal
  revision custody; edit-and-revert is not detected. Whole-timeline validation is
  conservative until writers become narrower. Agent read/propose/accept tools,
  lock/containment parity, undo, session-change custody, native UI acceptance,
  Pumas live inference and generation/stream persistence are still open.
- **Regression intent:** All nine writers reject a stale snapshot, rejected
  writes leave no history, deletion/membership/duration changes conflict, replay
  remains idempotent, and a refreshed snapshot preserves the prior edit.

- **Initial save:** A command arriving before the first durable project save
  receives an explicit conflict and leaves no rows/history behind. It must wait
  for save and reload; this slice does not alter project creation lifecycle.


## M4b: complete descendant range revision history

- **Observed defect:** A range edit proportionally resizes descendants but records
  only the target. Adding those missing revisions also exposes that per-object
  history was sorted by per-event `sort_order` alone, which can replay an older
  descendant delta after a later direct edit.
- **Write set/owner:** Server range-history writer, existing per-object history
  reader, focused tests and these source/plan records. No public API, dependency,
  schema, geometry semantics, inference or renderer changes.
- **Decision:** Record target plus actually changed descendants in the same event
  and transaction, preserving full old/new range pairs. Sort descendant output
  by stable node identity. Read revisions by committed event insertion order,
  then within-event order; do not assume caller clocks are monotonic.
- **Alternatives:** Recording only the target loses reconstructible history.
  Using the event-local revision index as a global clock is incorrect. A second
  history store, event migration or broad timeline rewrite is unnecessary.
- **Gate:** Multi-level descendant values match persisted nodes; unrelated/no-op
  descendants remain sparse; replay is idempotent; stale commands leave no partial
  descendant revisions; parent resize followed by direct child edit replays to
  the current range even with non-monotonic caller timestamps.
- **Limits:** Native UI, complete undo application, subtree locks, containment/
  split policy, immutable proposal preconditions and agent mutation tools remain
  open. This slice stays separate from M4a publication/review.

## M4 follow-on: correct gap projection after overlapping edits

- **Observed defect:** A shorter clip inside a longer clip moves the core gap
  cursor backwards, exposing occupied time as a gap after a manual range edit.
- **Owner/write set:** Existing core `Timeline::find_gaps`, focused core and
  renderer-projection tests, source READMEs and plan records. This follows the
  existing contract that a gap contains no story node and core owns invariants.
- **Decision:** Track the furthest occupied endpoint and its node identity;
  order equal-start ranges by end then node identity for stable gap neighbors.
  Keep the existing minimum-duration filter and level isolation.
- **Gate:** Nested, overlapping and touching clips yield only unoccupied gaps;
  boundary identities survive storage reordering; a resize is reflected correctly
  in the renderer projection; core and frontend gates pass without inference.
- **Limits:** Native visual acceptance is delegated separately. This changes no
  split/containment product policy, structural locks, mutation history, agent
  authority or undo behavior and does not complete M4.

## M4 follow-on: timeline projection cache session custody

- **Observed defect:** Project activation clears projection caches, but an old
  timeline refresh or command can complete afterward. Version-only comparison
  admits the old project's higher-version projection into the new session;
  obsolete failures/finalizers also overwrite its error/pending state.
- **Owner/write set:** Timeline frontend projection store, deterministic deferred
  tests, store README and plan records. Preserve public API and command results.
- **Decision:** Each request captures the cache generation. Clear advances that
  generation; only its own requests may publish state. Within the generation,
  versions govern projection replacement, outstanding requests own pending as a
  count, and the latest-started request owns the shared error field.
- **Gate:** Deferred refresh and all command paths cannot cross clear/new-load
  boundaries on success or failure; obsolete completion cannot end current
  pending work; overlapping commands/refreshes retain version ordering and
  pending/error ownership. Run frontend tests, typecheck, lint, format and build.
- **Limits:** This does not bind backend commands to a project session, cancel
  writes, suppress caller continuations, fix other projection stores, or resolve
  native acceptance, split policy and complete M4. Dot owns runtime qualification.

## M4 follow-on: bind backend timeline command admission

- **Observed defects:** Path capture precedes asynchronous project loading, whose
  failure fallback reads the then-active mirror. Create-child derives intent from
  the admitted project but calls create-node admission again, potentially writing
  a different database when node IDs are preserved across a copied/reopened project.
- **Decision/write set:** Capture path and fallback mirror under the project
  guard before I/O for all timeline writers. Forward create-child's captured
  path/project to the existing create-node executor without re-admission. Scope
  is the timeline service, deterministic admission tests and documentation.
- **Gate:** Failed-load fallback retains admitted project identity after a switch;
  a queued create-child with shared node IDs writes A while preserving B's persisted
  nodes and active database/mirror identity. This does not establish isolation of
  B's shared Y.Doc, event stream or autosave from old-command post-commit effects.
  Compile/check and strict Clippy here; actual native tests are delegated to dot.
- **Evidence limit:** A dependency-free harness executes the extracted admission
  function with deferred/failing I/O. It is function-level evidence, not server
  runtime qualification. Skipped-ORT compilation is never native test evidence.
- **Remaining custody:** This binds the source once admitted. It does not add
  expected frontend session tokens to the wire, cancel admitted writes, fence
  post-commit events/Y.Doc/autosave, or fix concurrent save-path publication.

## M4 follow-on: admitted timeline post-commit and document custody

- **Observed defects:** An admitted A command can publish document writes, events
  and save signals after B becomes active. Autosave captures mirror/path before
  awaiting document serialization, allowing mixed-session mementos. The documented
  Y.Doc replacement instead merges updates; empty/fallback loads retain old data.
- **Decision:** One async gate spans admitted timeline work and its post-commit
  effects, project create/load/save and autosave snapshots through persistence.
  Capture session identity before a timeline request waits; reopen/replacement
  and save-as renew identity, returning a conflict for obsolete queued requests.
  Restore saved documents into a fresh CRDT store, reset before fallback population,
  and reattach update observation. Invalid-load/channel errors remain explicit.
- **Transition repair:** Explicitly flush the outgoing mirror/path/document under
  that gate before create/load replaces it. For Save As, flush and recover the
  committed source snapshot before writing a new destination; retain current-state
  copy semantics without copying command history or affect stores. Reject different
  existing destinations rather than mixing database ownership. Required document
  serialization and persistence failures keep the active session in place; autosave
  serialization failure skips its write and preserves the stored blob.
- **Caller cancellation:** After admission, supervised work owns the gate through
  blocking persistence and publication even if its caller disconnects. Admission
  waits remain cancellable. Shutdown can abort work; crash recovery and unrelated
  producers remain outside this guarantee.
- **Publication:** Await bounded document sends under the gate before emitting
  timeline events/save signals. If the manager closes after SQL commit, return
  an explicit committed-publication error; SQL history remains durable/idempotent.
  This does not implement a distributed transaction or automatic rollback of
  already committed SQL when its document publication fails.
- **Write set:** Timeline services, project lifecycle, state/autosave, Y.Doc manager,
  admission signature adaptation, focused component/native tests and records.
- **Gate:** Component tests verify true document replacement, empty reset, invalid
  state preservation and live update observation. Native tests must qualify seeded
  A/B history preservation, post-commit document/event ordering with a full channel,
  queued same-path reopen/save-as rejection and snapshot ownership during autosave.
  Transition tests must include an earlier real document blob before committing
  new notes, immediate A-to-B-to-A reopen, Save As from a stale mirror, outgoing
  serialization/persistence errors and cancellation at blocked publication/flush.
- **Limits:** Source and isolated document component checks do not qualify complete
  server behavior. Native tests are delegated. Other producers and frontend intent
  tokens before backend admission remain open; no completed M4/merge claim.


### Lock semantics clarified during M4 review

`StoryNode.locked` is a content-regeneration lock, as documented by its core
contract and enforced by single/batch generation admission. Absence of structural
move/cut/delete rejection is not a violation of that contract. A future structural
edit lock or agent capability policy must be specified separately; this work must
not silently disable existing manual editing under a content lock.


### M4b ordering boundary

The current writer appends `change_events`/`object_revisions`; broad save clears
and reinserts only current-state tables and leaves history untouched. Project
load opens the same database; PDF export does not copy history. No supported
history export/import, event reinsert, VACUUM or history compaction path was found
in the source audit. A real broad-save/reopen regression preserves event row
identities/order and the final child projection. Row order is a local append-only
storage boundary, not a portable or global revision clock. Any future history
rebuild, logical export/import or maintenance that can reorder events must add
an explicit persisted sequence or preserve verified event order before using
this reader; this slice makes no guarantee for arbitrary external DB rewrites.
