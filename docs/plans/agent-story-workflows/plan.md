# Agent-assisted story workflows

**Status:** Active
**Current phase:** Authored Bible fact propagation is merged in main
`302851dbb5bf67cda922b4d79f70623e444891dc`; scene-order continuity is qualified
at frozen predecessor `441c2a29af6199b53b92f718df43d0ca916b1ce5`.
Immediate canonical creation/generation is implemented at
`9e8bd1c9d51250ac1c517945269278b7fe7e3d61`: 117 hosted core / 406 server tests,
405 frontend tests, and actual native create/select/notes/manual anchor/generate
without reopening. The delayed native response refuses persistence and the failure
capture retains exact human text, but final banner/history qualification is pending.
**Next gate:** Hosted run 37365296113 on qualification
`776d16c6e1039641c00beba9dade31465c9f28fe` failed before acquiring a runner during
GitHub's active runner-assignment incident (zero steps/artifacts). Retry that hosted
job after service recovery. Keep source/evidence frozen and inspect its actual UI
result before classifying a locator or error-publication defect. Parent owns
review/PR/merge and Library delivery. See the canonical generation qualification
report for exact source, tests, screenshots and preserved failures.
Timed facts, relationships and new/unconsumed Bible facts remain separate follow-ups.
Project-switch recovery is deferred; embeddings are optional later work.

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

## Scene order: complete continuity window

The independent follow-up starts from frozen Bible head
`af603e7682417fd49dbaac02e75142fd1b9d0d61`. Existing two-scene placement and
consumed-input review already work. A source reproduction verified a different
gap: moving unseen E from 9000 to 3000 changes B's actual selected window, and
moving B from 4000 to 8500 removes A/E, without any old consumed-source revision
change or review cause. Human B text remained unchanged in both cases.

- Capture the complete ordered selected segment window alongside actual inputs
  in existing generation command/proposal history. An absent legacy receipt
  stays absent; do not claim all historical inputs were complete windows.
- Derive review for entering/leaving members or changed external order/relative
  position. Ignore the output segment's own intentional replacement and ordinary
  target shifts that leave external selection unchanged. Preserve manual blocks,
  drafts, factual-time policy and existing source revision guards.
- Preview from the fresh window and accept only through existing targeted review.
  Refresh its existing dependencies atomically. The latest main-document segment
  event is a conservative append-only selection epoch in the review binding,
  refusing unseen move-in/move-out ABA even when visible inputs return unchanged.
  Unrelated segment edits may require a new preview; they do not alone create
  Needs review. No new schema, vectors or parallel state.
- Qualify exact entering/displaced-neighbor and target-relocation reads, native
  public-service publication, synthetic provider preview and explicit acceptance.
  Native screenshots must distinguish public-service setup from GUI actions.
  Existing timeline drag bounds and semantic story extraction remain outside
  this bounded continuity receipt; presentation placement never infers story time.

## Immediate canonical scene generation: bounded successor

Frozen predecessor: 441c2a29af6199b53b92f718df43d0ca916b1ce5. New branch:
feat/canonical-scene-generation. Cause and criteria were reported before code changes.
Canonical create writes SQLite without inserting into state.project. Admission
loads canonical SQLite, but successful_generation_metadata reads the stale mirror,
then returns before screenplay persistence when the new node is absent. It also
marks HasContent before the screenplay transaction. Save/reopen masks this ownership
split and is not an acceptable application fix.

- The existing create-child backend/transport lacks an editor action, while
  timeline double-click sends a parentless Scene rejected by core. Expose Add Scene
  in the selected Sequence editor using that validated API; backend derives parent,
  level and placement. Guard delayed acknowledgement selection by existing editor
  session/selection and mounted lifetime. No separate placement inference.
- Create a scene and generate immediately through public services and the real
  selected-node GUI without reopening. Completion reads canonical metadata and
  commits status with the generated screenplay, never inventing a mirror refresh.
- Capture target custody using existing timeline/script revisions at admission.
  Recheck existence, placement, notes, locks and output revision inside the writer
  transaction. Refuse delayed responses after target edit/delete/retime/lock or
  an intervening human screenplay write, including ABA; retain manual text,
  unrelated authored blocks and canonical placement. Stale upstream inputs remain
  captured provenance and produce review through existing mechanisms.
- Reuse existing command/history, generation dependencies and session guards.
  Preserve legacy replay; no new database/schema, credentials or dependencies.
  Project switching recovery remains deferred, rather than widening this slice.
- Test actual canonical selection and immediate creation with a deliberately
  stale mirror, delayed synthetic HTTP responses, refusal/no partial history,
  and successful ordinary streaming. Qualify UI create/select/notes/generate
  through native input, with Bible/timeline/screenplay visible, source-bound
  unaltered prototype evidence and explicitly synthetic model responses.

## Manual screenplay authoring: first bounded memory slice

The owner prioritizes manual screenplay edits, timeline-driven story changes and
propagation into agent memory. Recovery/project switching work is parked. This
slice starts from exact accepted integration `9f44a974bc7830e721237b3ccb295095c1bed5f6`
(tree `ea1c975c59c33535304cf226df99f008ce1900bc`).

- **Authoring path:** Edit an existing screenplay block in Script, explicitly
  save its exact text, and keep a refused draft available for correction or
  discard/reload. Commands contain document/block IDs, expected block write event
  and text; the backend preserves existing kind, ordering and segment metadata.
- **Canonical write:** Sparse block/span revisions retain the old/new author
  text. The write transaction rechecks current block identity and protected spans.
  Stale/ABA and lock failures do not leave command/history or partial text writes;
  replay does not overwrite an intervening edit.
- **Memory read:** Preview and generation share exact main-document screenplay
  reads for target/intersecting and two adjacent segments on each side. Context
  carries block and segment IDs plus separate write events. Presentation ranges
  choose continuity evidence; they do not infer fictional validity. Canonical
  authored text replaces unversioned node-script/recap evidence in this path.
- **Projection propagation:** Command responses and script events refresh the
  screenplay/history and invalidate the selected prompt preview. Earlier same-node
  preview responses cannot replace newer requests.
- **Acceptance:** Two linked scenes demonstrate type/save, exact text on reopen,
  old/new history, fresh script projection and fresh prompt evidence without a
  model. Essential stale/ABA, locked refusal and actual consumer regressions apply.
- **Deferred:** Creation of new blocks/documents, inferred semantic extraction,
  world-update acceptance, broader dependency impact, timeline semantics,
  embeddings/evaluator changes and recovery are separate follow-ups.

Design source: Puma, *Knowledge Graphs and Agentic Memory: Evidence Time Retrieval
and Scale*, research edition 2 October 2026, chapters 4–6, 11 and the Eidetic case
in chapter 13. The manuscript is a separate Library research deliverable; its
Eidetic audit targets `ab5d75ce138c1b8e25cb5effc8f8800c0c6f9841`. The accepted
source already includes the later fictional-time resolver. Reuse canonical
SQLite/history/projections; neither a graph database nor embedding relevance
replaces authored evidence or acceptance policy.

## Manual creation: bounded follow-up

On `feat/screenplay-story-memory`, close the empty-screenplay writing gap from
verified merged main `27680cb52c2b8bdfcdc478bb7f35801aa2d8e072`. Existing manual
editing, generation lineage, explicit review and range-to-script placement are
preserved. Earlier milestone sections retain their historical qualification.

- **Authoring:** Select a timeline clip, choose Write screenplay, type exact text
  and block kind, then save. Create the main document and its source-bound segment
  only when absent; subsequent commands append independent blocks. Existing text,
  locks, segment status and placement fields remain unchanged. Appends advance
  the segment dependency revision with sparse new-block membership history.
- **Authority:** SQLite owns source admission, deterministic command-derived
  block/span identities and append order. Recheck current placement/order under
  the writer lock; refusals roll back history and text. Capture project ownership
  before waiting and retain its gate through completion and publication.
- **Drafts:** Capture context/document and a stable request identity at begin;
  selection changes do not retarget pending text. An uncertain acknowledgement
  retains the exact submitted payload/ID in the active project-session owner
  through Script/Graph/Split navigation and Script consumer replacement, including
  delayed acknowledgement failure while Script is absent. Returning Script exposes
  exact retry; actual project activation replaces the draft owner and excludes
  old responses/callers. Editing, discard,
  restart and placement refresh stay disabled while that result is unknown.
  The definite native pre-recording placement refusal unlocks editing and
  explicit current-placement refresh for the original source. Same-node move or
  resize refreshes selected-node evidence from the canonical timeline range;
  recovery waits for agreement and retains explicit read-error recovery.
- **Memory:** Reuse canonical screenplay reads and revision invalidation. New text
  survives reopen, appears in neighboring prompt context and follows ordinary
  source range changes without changing authored text revisions. A generated
  descendant that consumed this segment exposes Needs review after an append.
  Its normal preview captures all new members; explicit acceptance refreshes
  actual lineage, and later edits to those members trigger review again. No model
  call is required to create canonical text or derive the review cause.
- **Qualification:** Execute actual SQLite source modules, core contracts and
  frontend logic/rendering fixtures here. Native AppState and graphical desktop
  execution remain distinct gates; skipped-ORT compilation is compile evidence.
- **Deferred:** Automatic semantic extraction into the bible, live-model quality,
  broader authoring operations and project switching remain separate work.

## Manual edit continuity and source navigation: bounded follow-ups

- Existing-block edit drafts belong to the active project session, keyed by
  document/block. Script/Graph/Split removal, selection changes and projection
  refresh retain exact text and captured revisions. Uncertain saves reconcile
  immutable payload/IDs through existing revision/replay/lock authority.
- Known definite native stale/locked refusals retain editable drafts. Explicit
  discard/reload clears only after a successful canonical read. Drafts are
  transient; application restart and project-switch recovery remain deferred.
- Execute late-acknowledgement/navigation and exact-text frontend fixtures, plus
  actual SQLite reconciliation/context/downstream-review evidence. Distinguish
  these from native GUI and live-model evidence. Freeze this milestone, then
  continue the independent navigation follow-up without awaiting review.
- Source navigation displays existing source clip identity/name/range from the
  canonical timeline projection and selects that clip through existing editor
  state. Missing or unbound sources remain explicit. Navigation must retain live
  drafts and must not infer fictional time, rewrite placement or issue writes.

## Retained-draft comparison: bounded authoring follow-up

- A refused or still-open edit can read and display current canonical saved text
  beside its exact retained draft. Reading never writes text or changes its base
  revision. Preserve the comparison across normal workspace navigation.
- Explicit Continue draft from this version retains exact draft text and advances
  only its expected revision to the read snapshot. Current projection must agree
  with that snapshot; changed/ABA versions require a fresh comparison. A later
  intervening write or lock still refuses through existing backend validation.
- Uncertain saves keep immutable payload/ID reconciliation and cannot compare,
  discard or continue another version. Failed/missing reads keep the draft and
  admit no unread version. Reuse canonical command/projection and session owners;
  no backend authority change, automatic merge, save or proposal acceptance.
- Qualify actual frontend read/controller/store/SSR flows with fixture native
  transport, plus actual SQLite stale/read/continuation, exact memory/downstream
  review, idempotent replay and late lock regression evidence. Parent separately
  qualifies native GUI and full server execution. Project switching is deferred.

## Screenplay generation lineage and Needs review: bounded descendant

Continue on a separate descendant of frozen manual-authoring `9fe7a4a5`.
The parent has independently accepted that first feature's source/native-service
behavior (113 core, 314 server, 338 frontend and desktop compile/smoke), with
graphical typing/save/reopen explicitly unqualified due the extracted runtime's
hardcoded WebKitNetworkProcess path. This second milestone still requires
independent qualification; the GitHub publication hold remains unchanged.

- Persist successful output and its actual captured screenplay revision lineage
  atomically in existing SQLite/history/semantic dependency storage. Each input
  binds separate block-text and segment-placement revisions to the output event.
  Validate historical evidence even when source edits/deletion happen during model
  execution; never rebind to a later source revision at output commit.
- Failed/empty streams create no output or lineage. Replay signs the complete
  captured context, does not duplicate rows and does not replace later outputs.
  Additive generation/binding tables preserve older unbound dependencies. Known
  empty inputs and unavailable history remain distinct.
- Derive Needs review for the latest successful generation by comparing bound
  consumed revisions with current inputs. Include changed/deleted causes and
  historical source excerpts. Preserve prior lineage after explicit regeneration;
  a refreshed binding supersedes its old impact. Keep own prior-draft inputs for
  audit while ignoring their intentional replacement as external impact.
- Render review causes beside exact screenplay text. Preserve authored content,
  locks, canonical segment status and explicit proposal acceptance. No automatic
  cascade, world-fact inference, acceptance/dismissal, graph-database migration,
  timeline semantic change or project-switch work belongs to this milestone.
- Required fixtures: A-to-B and unrelated C; failed generation; source edits
  during generation; replay; deleted-source explainability; refreshed binding.
  Verify actual SQLite source modules here and native integration independently.
  Skipped-ORT compilation cannot qualify server runtime behavior.


## Targeted review proposal: bounded follow-up

Frozen lineage `0b687a8168c3499cfbcc356f814958d7ee11859b` has parent-reported
independent source/native-service acceptance: 113 core, 323 server, 339 frontend,
checks/build and additional deletion-before-commit and placement-only probes.
Graphical Needs review remains unqualified under the existing WebKit blocker.
Continue separately on `feat/script-impact-review`; publication hold stays intact.

The actual missing feature is a bridge from a proven review cause to a targeted
pending propagation proposal and its explicit accept/reject flow. Existing
proposal storage/review commands are reused; older generic proposals remain
unbound. This slice adds a preview service/provider boundary, additive captured
evidence binding, transaction-local stale/lock guards and a Script review surface.

- Target the output block identified by the generation's impact. Capture current
  authored screenplay context and resolved graph evidence at explicit optional
  fictional time; reuse current SQLite, semantic lineage and temporal resolver.
  Include a proven moved source outside the default continuity window. No world
  extraction, database migration or presentation-to-fictional-time inference.
- Preview creates a proposal without changing canonical text. Strict stream
  completion refuses partial/error/empty output. A persisted request replays
  without another provider call. Prompt sources are completely represented by
  the stored binding, without unbound timeline prose or recaps.
- Explicit acceptance rechecks target/source revisions, cause, pending proposal,
  resolved graph evidence and locks inside the writer transaction. Commit only
  the selected block/span, proposal status, sparse history and actual refreshed
  lineage. Preserve source edits, unrelated C, other blocks and segment metadata.
- Rejection records review status and retains authored text and the review cause.
  Refused stale/ABA or locked acceptance leaves a pending proposal available for
  rejection. Retargeting/amending a bound preview requires a fresh preview.
- Qualify deterministic linked scenes and the existing provider boundary here;
  independently qualify native service/desktop behavior on the exact checkpoint.
  No real-model quality, native GUI, full server runtime or recovery claims arise
  from isolated module tests or skipped-ORT compilation.

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


## M4 follow-on: validate proportional resize atomically

- **Observed defects:** A contraction can accept a zero-duration descendant.
  Floating-point scaling can also change exact no-op milliseconds or panic on
  unchecked addition for a valid large range, after already changing the target.
- **Decision/write set:** Stage all descendant ranges and validate their positive
  duration before mutation. Use checked integer offset scaling with exact
  floor-to-millisecond division. Keep existing start saturation/end clamping.
  Scope is core resize, arithmetic error, regressions and documentation.
- **Gate:** Collapsed/malformed ranges reject without partial mutation or panic;
  valid large endpoints, large no-ops, multiple levels, unrelated nodes and a
  one-millisecond positive target remain correct. Core tests/Clippy must pass.
- **Limits:** No new crossing-child containment, cut-content, hierarchy, locking
  or undo policy. Server history/native gates remain a separate dot handoff.

## M4 follow-on: keyboard timeline caller continuation custody

- **Observed defect:** Delete/split clear selection after awaiting the command,
  even if another clip is selected or a reopened session reused the same ID.
  The shell's shortcut failure handler can notify an unrelated new session.
- **Decision/write set:** Advance an editor-session generation on editor reset;
  delete/split clear only the still-selected target in their captured generation.
  Keep shortcut failure notification in the keyboard adapter under that same
  session check. Scope is editor store, keyboard adapter/tests and shell binding.
- **Gate:** Deferred delete/split preserve intervening selections and same-ID
  reopened sessions; rejection retains selection; current-session failures remain
  visible and old-session failures are suppressed. Run frontend gates.
- **Limits:** Backend writes, other command callers and renderer sessions remain
  separate custody work. This adds no split-content or containment policy.

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
- **Task records:** Normal task admission reaps completed handles and observes
  their join outcomes, preserving named failure reporting. Retain unfinished
  handles for admitted completion and shutdown joining. Completed tail records
  can remain until the next spawn/count/shutdown; repeated edits must not grow
  the registry with historical commands. Validate repeated real commands using
  passive registry measurement, without smoke-counter-driven pruning.
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

## Accepted follow-up integration qualification

- Compose accepted keyboard continuation, participating server custody/task-record
  lifecycle and atomic resize lineages on merged PR6 main `a6bd4c1`. Preserve exact
  source ancestry and main-only WAL tests; keep already merged gap/cache behavior.
- Gate the composition with per-file accepted-source identities, core/frontend
  tests and strict Clippy/format/typecheck/build/traceability, plus server compilation.
  No additional Rust implementation changes or features belong to this candidate.
- Parent qualifies native server/runtime behavior on the exact combined tree,
  especially resize through history/rollback and project transitions. Cloud
  skipped-ORT compilation does not fulfill that gate. Publish through the normal
  native-qualified source route; use a verified bundle when preserved-ancestry
  publication is unavailable. Keep main and merged PR6 unchanged during preparation.

## Targeted preview transport integrity repair

- Keep the targeted-review checkpoint `a372d4f` frozen. Repair its independently
  demonstrated partial-success and split-event data loss on a separate descendant.
- Both configured text adapters use a stateful SSE reader that buffers byte/line
  fragments, preserves event order and Unicode, requires completion plus clean
  HTTP EOF, and surfaces malformed/provider/transport errors. Full collection
  must propagate errors rather than returning a prefix.
- Exercise actual loopback HTTP for both adapters and the canonical preview/
  proposal boundary. Native AppState tests additionally qualify the public preview
  service: failure produces no proposal/history/event; clean split SSE produces
  the entire pending proposal with authored screenplay preserved.
- This is provider transport correctness, not model quality. Ordinary timeline
  placement work remains a separate feature; no graph backend, recovery behavior,
  model call, publication or merge belongs to this repair.

## Ordinary timeline range edits propagate screenplay placement

- Concrete gap: move/resize commands recorded timeline ranges only. Canonical
  screenplay/context retained old placement and dependent scenes never acquired
  a consumed-placement review cause; the service published only timeline refresh.
- On a separate descendant of accepted transport checkpoint `11c437f`, synchronize
  live source-bound segment ranges for the edited node and changed descendants
  inside the same writer transaction. Read segment state under the acquired
  writer lock; record exact sparse placement deltas with the timeline event.
  Preserve authored text, block revisions, span/lock state and other metadata.
- Publish the existing script event after commit. The existing handler reloads
  script/review projections and invalidates cached prompt context. Existing
  consumption bindings derive Needs review; preview/reject/accept remain explicit.
- Historical generation validation reconstructs sparse segment fields at the
  exact captured event, preserving historical evidence after intervening edits.
  No new schema/backend or automatic screenplay/world rewrite is needed.
- Deterministic two-scene fixtures cover moving A after B, changed context order,
  placement-only impact, retime/ABA, replay after authoring, descendant resize,
  atomic rollback and fresh explicit proposal acceptance. Native qualification
  must exercise the public range service's success/replay/rollback publication.
- Keep fictional time explicitly optional and independent from screen placement.
  Track reparenting/hierarchy changes and project recovery are outside this slice.

## Authored Bible fact propagation: bounded consumed-field slice

Verified PR9 base `25b860d12fe37a3538a40616cc02ab4a81370863` and merged main
`9f75d1cb9865bbea9c93bd2b1411eb78ed7eab09` have identical accepted tree
`d22b036891aed7eb66fc94059aa1b5c1d9aab85a`. Concrete missing behavior:
BibleGraphPartFields writes canonical values/history, and ai_service supplies
resolved graph fields to generation, but ai_generation_runtime persisted only
script_context; script_impact_projection ignored BibleField endpoints. The Bible
event also left screenplay impact and cached prompt context unchanged.

- Capture untimed resolved field identities, values and exact revisions with the
  Bible context in a single read snapshot. Forward through successful generation
  and validate historical evidence before committing output and existing UsesFact
  semantic dependency bindings. Never rebind late output to a newer fact.
- Derive changed/deleted causes and consumed excerpts through existing sparse
  history. Reuse the targeted pending proposal and explicit accept/reject workflow;
  refresh actual field lineage only on accepted replacement. Preserve human text,
  drafts, locks, stale/ABA refusals and unrelated screenplay.
- Qualify exact manual Bible edit, downstream review, preview preservation and
  explicit targeted acceptance with Bible, timeline and screenplay in the real
  native window. Label the production-client HTTP fixture as synthetic; no model
  quality claim. Use the existing normal hosted native build route without
  retrying/bypassing the known local ONNX403 acquisition failure.
- Bound scope: baseline fields actually supplied to generation, including node
  text stored as a field. Timed snapshots, relationships/node names, added or
  previously absent/unconsumed fields and broader semantic extraction remain
  follow-ups. Existing unbound outputs are not assigned inferred historical
  consumption; project recovery and embeddings remain deferred.
- Frozen application fe7fa590 passes 115 actual core / 389 actual server / 400
  frontend tests and real Tauri fact-edit, retained-draft, pending preview and
  explicit accept checks. Final qualification b1129893 and run 37337057557 await
  settled UI before capture. Full source, hashes and limits are in
  [the qualification report](../../reports/bible-fact-native-qualification.md).
