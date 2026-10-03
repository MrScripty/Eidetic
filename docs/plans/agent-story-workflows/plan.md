# Agent-assisted story workflows

**Status:** Active
**Current phase:** M2a field-level story-time projection accepted by independent review.
**Next gate:** Qualify the current temporal projection head with hosted CI and
CodeRabbit review; M1 and its CI repairs are merged into main.

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
| M4: Shared manual/agent timeline editing | Planned | timeline command contracts/services; agent tools; Tauri/UI bindings/tests | Create/move/cut/resize/delete parity, containment/lock validation, replay, undo, stale proposals |
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
