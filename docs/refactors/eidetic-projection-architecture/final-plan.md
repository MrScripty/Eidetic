# Eidetic Projection Architecture Refactor Plan

## Objective

Replace the current mixed ownership model with a backend-owned, projection-driven architecture for timeline context, script generation, story bible worldbuilding, semantic history, and Bevy rendering.

The implementation must leave one canonical source of truth: backend-owned SQLite state plus transactional command/event/revision records. Svelte and Bevy consume projections and submit commands. They do not own persistent project state.

## Source Planning Inputs

This execution plan consolidates the discovery notes in:

- `plans/story-tracks.md`
- `plans/story-bible-worldbuilding.md`
- `plans/script-generation-model.md`
- `plans/timeline-rendering-bevy.md`
- `plans/story-bible-3d-graph-view.md`
- `plans/architecture-blast-radius.md`

## Standards Reviewed

Implementation must comply with:

- `CODING-STANDARDS.md`
- `ARCHITECTURE-PATTERNS.md`
- `PLAN-STANDARDS.md`
- `TESTING-STANDARDS.md`
- `DOCUMENTATION-STANDARDS.md`
- `FRONTEND-STANDARDS.md`
- `ACCESSIBILITY-STANDARDS.md`
- `CONCURRENCY-STANDARDS.md`
- `INTEROP-STANDARDS.md`
- `SECURITY-STANDARDS.md`
- `DEPENDENCY-STANDARDS.md`
- `TOOLING-STANDARDS.md`
- `CROSS-PLATFORM-STANDARDS.md`
- `languages/rust/RUST-API-STANDARDS.md`
- `languages/rust/RUST-ASYNC-STANDARDS.md`
- `languages/rust/RUST-SECURITY-STANDARDS.md`
- `languages/rust/RUST-INTEROP-STANDARDS.md`
- `languages/rust/RUST-DEPENDENCY-STANDARDS.md`
- `languages/rust/RUST-TOOLING-STANDARDS.md`
- `languages/rust/RUST-CROSS-PLATFORM-STANDARDS.md`

## Hard Constraints

- No backwards compatibility with current project data structures is required.
- Old canonical paths must be removed when replacements land.
- Backend remains the only source of truth for persistent data and business decisions.
- No optimistic UI updates for backend-owned state.
- Frontend and Bevy may cache projections only as versioned read models that can be discarded and rebuilt.
- Bevy replaces the DOM/SVG timeline renderer; the DOM/SVG timeline must not remain as a runtime fallback.
- Bevy and Svelte can own transient UI/render state only.
- AI changes must be proposals until accepted through the same command/event path as user actions.
- SQLite remains the canonical local-first database; large assets live as validated project-relative files with SQLite metadata.
- JSON is not canonical storage for queryable bible facts, semantic claims, dependencies, script spans, locks, or history.

## Assumptions

- Eidetic remains local-first and can use a per-project SQLite database as the durable backend store.
- Existing data can be discarded or migrated manually later; compatibility migration is not part of this refactor.
- The current Svelte UI remains the application shell around editor panels, forms, inspectors, and accessibility command alternatives.
- Bevy can be introduced as a leaf renderer without forcing Bevy dependencies into core domain crates.
- AI behavior can be routed through proposal objects before state is committed.

## Dependencies

Internal dependencies:

- Current Rust workspace crates under `crates/`.
- Svelte frontend under `ui/`.
- Existing per-project persistence, AI routes, Y.Doc integration, export, and timeline/story/script modules.
- Existing documentation and ADR structure under `docs/`.

External dependencies:

- SQLite for local durable state.
- Bevy for the replacement timeline renderer and bible graph renderer.
- Existing AI backend dependencies.
- Y.Doc only if retained as active editing transport/cache.

## Affected Contracts And Artifacts

Structured contracts affected:

- HTTP command payloads and projection responses.
- WebSocket event envelopes.
- Bevy bridge payloads.
- Y.Doc bridge payloads, if retained.
- AI request/response/proposal DTOs.
- Export projections.
- TypeScript/Rust shared DTOs or generated schemas.

Persisted artifacts affected:

- Project SQLite schema.
- Event history and object revision rows.
- Bible graph, script document, semantic claim, dependency, and asset tables.
- Projection cache tables or fixtures.
- Schema fixtures, command fixtures, projection fixtures, and sample events.

## Worktree Hygiene

Before implementation begins:

- Inspect git status.
- Do not start source-code implementation while unrelated implementation files are dirty unless the user explicitly allows that state.
- Markdown discovery notes may remain dirty during planning.
- Commit each verified logical slice before starting the next implementation slice.
- Keep lockfile, generated contract, schema, and fixture changes owned by one serial integration step unless a specific worker plan says otherwise.

## Implementation Progress Log

Completed slices:

- `docs(refactor): add projection architecture plan` added this consolidated plan, the codebase impact review, and supporting planning notes.
- `fix(build): correct pumas library path` corrected the workspace path to the local Pumas library crate so Cargo can resolve workspace metadata.
- `fix(build): update pumas dependency lockfile` refreshed `Cargo.lock` for the corrected local Pumas library path.
- `feat(core): add projection contract primitives` added host-agnostic command, event, revision, typed value, and projection envelope contracts.
- `feat(server): add transactional history store` added SQLite command/event/revision persistence with typed field-delta columns, command idempotency, and rollback tests.
- `feat(server): rebuild field projections from revisions` added a read-side projection adapter that rebuilds an object's current field state from persisted revision history.
- `feat(server): apply object field commands through history` added a validated command handler that writes field updates through history storage and returns rebuilt projections.
- `feat(server): expose object field command route` added `POST /api/commands/object-field` as a command-in/projection-out HTTP boundary over the history command path.
- `feat(server): return versioned object field projections` wrapped object-field command route projections in `ProjectionEnvelope` with projection versions and the latest change event ID.
- `feat(ui): add object field command helper` added a typed frontend command helper that submits object-field commands and returns versioned backend projections without mutating UI stores.
- `refactor(server): centralize sqlite write setup` moved write-capable SQLite connection pragmas into one server helper used by structural persistence and command routes.
- `feat(server): expose object field projection route` added `GET /api/projections/object-field` for focused versioned projection reads from persisted history state.
- `feat(ui): add object field projection helper` added a read-only frontend helper for focused object-field projection fetches without mutating stores.
- `refactor(server): share projection route helpers` centralized active project path lookup and history error mapping across command and projection routes.
- `feat(ui): add object field projection store` added a focused projection cache/action layer that reads and writes through backend projection APIs without mutating broad bible entity state optimistically.
- `feat(core): add bible graph contracts` added canonical story-bible root nodes, typed graph node/part/field/edge contracts, and a node-detail projection shape for persistence and UI slices to share.
- `feat(server): add bible graph node commands` added a backend-only create/read vertical slice for canonical bible graph nodes, including typed SQLite rows, transactional history/current-state writes, idempotent command handling, and versioned node-detail projections.
- `feat(ui): add bible graph node api helpers` added typed frontend command/projection helpers for the backend bible graph node create/read slice without mutating legacy entity stores.
- `feat(ui): add bible graph node projection store` added a focused UI cache/action layer for backend-owned bible graph node projections and create-command responses without mutating legacy entity state.
- `feat(server): expose bible graph node list projection` added a versioned read projection for persisted bible graph node lists with stable ordering and revision-history versioning.
- `feat(ui): add bible graph node list helper` added a typed frontend projection helper for the backend-owned bible graph node list read model.
- `feat(ui): cache bible graph node list projections` extended the bible graph projection store with a backend-owned node-list cache and create-command invalidation.
- `feat(server): add canonical bible root command` added an explicit command path for persisting missing system-owned canonical bible root nodes through history and typed graph rows.
- `feat(ui): add canonical bible root command helper` added a typed frontend command helper for initializing backend-owned canonical bible root nodes.
- `feat(ui): cache canonical bible root command projections` routed canonical-root command responses into the bible graph node-list projection cache.
- `feat(server): add bible graph field command` added typed part/field current-state rows, a transactional graph-field command, and populated node-detail projections versioned from node and field revision history.
- `feat(ui): add bible graph field command helper` added typed frontend command and projection-store helpers for backend-owned bible graph field updates without invalidating node-list projections or mutating legacy entity state.
- `feat(ui): render bible tab from graph projections` moved the story-bible navigator/list surface onto backend-owned bible graph node-list projections, with graph-node selection kept separate from legacy entity detail selection.
- `feat(server): validate bible graph parent nodes` enforced parent/child graph invariants for node creation before history writes, including command and route coverage for missing-parent rejection.
- `feat(ui): show bible graph node details` added a read-only projection-backed graph-node detail panel for selected bible graph nodes without routing through legacy entity detail state.
- `fix(ui): ensure bible graph category roots` tightened graph-node creation so the Bible navigator asks the backend to ensure canonical roots when the specific category root is missing.
- `refactor(ui): split bible graph tab controls` extracted category/root mapping, filter controls, and add controls from `StoryBibleTab.svelte` to resolve the frontend component decomposition issue before schema editor work.
- `refactor(server): split bible graph field storage` extracted graph schema setup and part/field storage from `bible_graph_store.rs`, bringing graph store modules back under the decomposition threshold before edge/snapshot work.
- `feat(server): persist bible graph edges` added a typed graph edge command route, relational edge current-state storage, endpoint validation, and incoming/outgoing edge loading in node-detail projections.
- `feat(ui): show bible graph node edges` rendered incoming and outgoing bible graph edges from node-detail projections without reading legacy entity relationship state.
- `feat(ui): add bible graph edge command helper` added typed frontend command/store helpers for backend-owned edge writes, caching only the returned source-node projection and invalidating stale target-node detail projections.
- `test(ui): split bible graph projection store tests` separated projection read/cache tests from command cache-write tests to keep graph projection store coverage under decomposition thresholds before schema editor work.
- `feat(server): project bible graph schema defaults` added core-owned built-in graph schema defaults, projected expected empty parts/fields for known schemas without persisting empty rows, overlaid persisted field values, and validated known-schema field commands.
- `feat(server): expose bible graph schema projection` added a versioned backend projection route for built-in graph schema defaults so UI/schema-editor work can consume backend-owned schema read models instead of duplicating defaults.
- `feat(ui): add bible graph schema projection helper` added a focused frontend schema projection contract file and read helper for backend-owned bible graph schema projections without expanding the oversized shared `types.ts`.
- `feat(ui): cache bible graph schema projections` added a focused Svelte projection cache for backend-owned graph schema read models with pending/error handling and no optimistic persistent state.
- `feat(ui): edit bible graph projection fields` extracted graph field rendering into a focused component and routed field saves through backend-owned graph field commands with local draft state only.
- `feat(ui): add bible graph edge creation form` added a projection-backed edge creation form in graph node detail that submits backend edge commands and waits for returned projections instead of inserting local edges.
- `feat(ui): gate graph creation by schema projections` loaded backend-owned graph schema projections in the bible tab and disabled node creation for categories whose schema is not present in the backend projection.
- `refactor(ui): centralize bible graph category adapter` consolidated bible graph category lists, short labels, node names, and schema projection resolution so UI category presentation stays separate from backend-owned schema availability.
- `refactor(server): share bible graph field value codec` extracted the SQLite `FieldValue` codec from graph field storage into a reusable backend module for upcoming node-scoped snapshot field persistence without changing existing graph-field behavior.
- `feat(core): add bible graph snapshot contracts` added typed graph snapshot IDs, snapshot field DTOs, node-detail snapshot projections, and snapshot-field command contracts across Rust and TypeScript projection boundaries before persistence is introduced.
- `feat(server): store bible graph snapshots` added SQLite current-state tables, typed snapshot field upsert/load storage, node-detail snapshot projection loading, and node-detail versioning from bible snapshot revisions.
- `feat(server): add bible graph snapshot field command` added validated snapshot-field command application and an HTTP command route that records bible snapshot revisions and returns updated node-detail projections.
- `feat(ui): add bible graph snapshot field command helper` added frontend command and projection-store helpers for snapshot field updates, caching only returned backend node-detail projections without optimistic snapshot insertion.
- `feat(ui): show bible graph snapshots` added a read-only projection-backed snapshot list in graph node details so persisted node-scoped snapshots are visible without using legacy entity snapshot APIs.
- `feat(ui): add bible graph snapshot editor` added a local-draft snapshot creation form that submits snapshot field commands and waits for backend node-detail projections instead of inserting snapshots client-side.
- `refactor(ui): split bible graph dto types` moved bible graph command/projection DTOs out of the oversized shared frontend type file while re-exporting them for existing import compatibility.
- `feat(core): add script document contracts` added host-agnostic script document, segment, block, span, lock, patch, command, and projection contracts in Rust and TypeScript before script persistence is introduced.
- `feat(server): add script document block command` added backend-owned script document current-state tables, history-backed block command application, a script document projection route, and route coverage for command/projection behavior before replacing legacy node-owned screenplay writes.
- `feat(ui): add script document api helpers` added typed frontend helpers and focused tests for script block commands and script document projection reads without introducing local canonical script state.
- `feat(ui): cache script document projections` added a focused script document projection store that caches backend reads and command responses by document ID while preserving backend ownership of script state.
- `feat(ui): render script panel from projections` moved the script panel read path from Beat node content to the backend-owned main script document projection and refreshes that projection on `script_changed` websocket events.
- `feat(server): add script document lock command` added a transactional script lock command route that validates existing spans, records lock revisions, upserts lock rows, returns updated script document projections, and emits `script_changed`.
- `feat(ui): add script lock command helper` added typed frontend script lock command support and projection-store cache updates for accepted lock responses.
- `feat(server): export script document projections` moved PDF export off Beat node screenplay blobs and onto the backend-owned main script document projection, with tests proving export does not fall back to legacy node script content.
- `feat(core): add script block span provenance` extended script block commands with explicit span provenance so AI-generated and user-edited script writes can enter the same backend command path without corrupting provenance.
- `feat(server): persist generated script blocks` routed successful AI screenplay generation through backend-owned script document block commands with AI provenance, emits `script_changed`, and stops mirroring generated screenplay text into timeline node content/Y.Doc.
- `refactor(script): remove node screenplay write path` deleted the legacy `/nodes/{id}/script` API helper/route and removed BeatEditor controls that edited or displayed node-owned screenplay blobs, leaving script display on backend-owned script document projections.
- `feat(server): protect locked script spans` made script block commands reject updates that would remove or change locked span text before any history/current-state writes occur, with regression coverage proving failed updates leave projections and revisions unchanged.
- `refactor(ui): remove legacy extraction and consistency UI` deleted orphaned extraction review/diff components, frontend helpers, websocket handlers, and editor store state that depended on node-owned screenplay text, keeping future semantic review work off stale UI state.
- `refactor(server): remove legacy AI script mutation routes` deleted `/ai/react`, `/ai/extract`, `/ai/extract/commit`, automatic post-generation bible mutation, and their websocket events/prompt helpers so AI no longer commits legacy graph/script changes directly from node screenplay text.
- `fix(server): decouple unlock from screenplay content` stopped node unlock from recalculating status from legacy `node.content.content`; node locks now leave script-related status untouched because durable script text is backend-owned by script document projections.
- `feat(server): add timeline render projection` added a backend-owned, layout-neutral timeline render projection contract and HTTP projection route for future Bevy timeline consumption without adding Bevy dependencies to core or retaining DOM-specific render data.
- `feat(ui): add timeline render projection helper` added focused TypeScript DTOs and a typed projection API helper for the backend-owned timeline render read model without changing the existing DOM timeline runtime.
- `feat(ui): cache timeline render projections` added a discardable frontend projection cache for the backend-owned timeline render model and refreshes it from timeline mutation websocket events for future Bevy host consumption.
- `feat(ui): derive timeline render models` added a pure projection-to-render-model adapter with normalized timing and clip indexes so a future Bevy host can consume deterministic derived data without owning canonical timeline state.
- `feat(renderer): add Bevy timeline bridge crate` added an isolated Bevy ECS leaf crate that receives backend timeline render projections and emits validated selection commands without introducing Bevy dependencies into `eidetic-core` or `eidetic-server`.
- `feat(renderer): build Bevy timeline scene entities` added read-only Bevy ECS track and clip entities rebuilt from backend timeline render projections, keeping scene state derived and disposable.
- `feat(renderer): expose Bevy timeline wasm bridge` added a wasm-bindgen wrapper for browser hosts to pass backend timeline render projections into the isolated Bevy bridge and drain validated renderer commands as JS values.
- `feat(renderer): add Bevy timeline hit testing` added renderer-owned clip hit testing by track and timeline time, including a wasm bridge method that emits validated selection commands without mutating backend-owned state.
- `feat(renderer): add Bevy timeline viewport state` added transient pan and zoom viewport state derived from projection duration, exposed through the wasm bridge without persisting renderer camera state.
- `feat(server): add timeline node range command` added a backend command route for validated timeline node move/resize operations that returns the updated timeline render projection for Bevy and Svelte command consumers.
- `feat(ui): add timeline node range command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed timeline node move/resize commands.
- `feat(ui): cache timeline command projections` routed timeline node range command responses into the timeline render projection cache without optimistic local patching.
- `feat(renderer): emit timeline node range commands` added validated renderer command emission for node move/resize requests so future Bevy drag/resize interactions can flow through backend-confirmed timeline commands.
- `refactor(ui): route timeline drag resize through commands` moved the existing DOM timeline move/resize handlers onto the backend timeline node range command/projection path without optimistic local timeline mutation.
- `feat(ui): add timeline nudge shortcuts` added keyboard-accessible selected-node nudge commands that submit backend timeline node range commands instead of mutating local timeline state.
- `feat(server): add timeline split node command` added a backend command route for validated timeline node split operations that returns the updated timeline render projection.
- `feat(ui): add timeline split command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed timeline node split commands.

Discovered issues:

- Commit hooks report `Can't find lefthook in PATH`. Commits succeed, but tooling setup is incomplete and should be fixed before treating hook execution as a verified gate.
- Baseline `cargo fmt --all -- --check` reports pre-existing formatting drift in server files. Do not mix that repo-wide cleanup into feature slices; either add a dedicated formatting cleanup slice or intentionally defer it with CI expectations updated.
- `rustfmt` on `crates/server/src/main.rs` can recurse through out-of-line modules and reformat unrelated baseline-drift server files. Until the dedicated formatting cleanup lands, format/check only newly added or intentionally touched Rust files and inspect `git status` before staging.
- Adding Bevy 0.16.1 to an isolated leaf crate pulled 94 packages into `Cargo.lock`. Keep subsequent renderer features behind the leaf crate/package boundary, avoid default Bevy features unless needed, and run dependency reviews before adding window/render/asset/text features.
- Timeline node range commands currently update backend-owned in-memory project state and trigger the existing save path, but they do not yet write sparse SQLite object revisions or command idempotency records. Migrate timeline structural commands to the same revision/history storage model before claiming durable undo/redo or replay for timeline edits.
- `cargo test -p eidetic-server history_store` passes but reports pre-existing dead-code warnings in `diffusion/types.rs` and `ydoc.rs`. These warnings block a future `-D warnings` gate and need a cleanup or ownership decision before CI can enforce warning-free server builds.
- The first implementation attempt exposed the stale Pumas path and lockfile state as a build metadata blocker. The path and lockfile are now fixed, and future slices should use Cargo verification instead of relying on stale metadata.
- The command route currently opens the active SQLite project path per request because `AppState` has no backend-owned database connection owner yet. Before broad route adoption, add an explicit database lifecycle owner with the same concurrency/shutdown discipline as the rest of the backend.
- Frontend bible editing currently mutates broad `Entity` caches and whole detail objects (`EntityDetail.svelte`, `story.svelte.ts`, `api.ts`, and websocket invalidation handlers). UI command adoption must use focused projection stores and avoid optimistic local patching or treating form state as canonical.
- Resolved: `crates/server/src/bible_graph_store.rs` exceeded the 500-line decomposition threshold while owning schema setup, node state, part/field state, and projection reads. It was split into schema, node/projection, and part/field storage modules before edge/snapshot work.
- Resolved: the first script document store implementation exceeded the 500-line decomposition threshold while owning schema setup, value codecs, current-state writes, projection reads, and tests. It was split into schema, codec, store, and test modules before the script command route was committed.
- Resolved: `cargo check -p eidetic-server` reported non-test dead-code warnings in `history_store.rs` (`RevisionOperation` import and `load_command`). The replay helper is now test-only and the production import set is warning-free.
- Resolved: `ui/src/lib/components/sidebar/bible/StoryBibleTab.svelte` exceeded the 250-line component decomposition threshold after moving list/navigation to graph projections. Category/root mapping and graph-node creation controls were extracted before schema editor work.
- Partially resolved: bible graph command/projection DTOs were split out of `ui/src/lib/types.ts` into a focused module. `types.ts` remains above the preferred decomposition threshold because it still mixes legacy story, projection primitives, extraction, script, AI, and settings contracts; split those before adding script, render, semantic proposal, or Bevy bridge DTOs.
- Resolved: legacy AI extraction and consistency routes read `node.content.content` and committed bible/script side effects directly. Those routes, frontend consumers, automatic generation follow-up mutation, and emitted websocket events were removed; future semantic work must re-enter through proposal contracts.
- Resolved: `unlock_node` derived content status from legacy `node.content.content`. Unlock now leaves status unchanged because script document projections own durable screenplay text.
- Resolved: `ui/src/lib/stores/bibleGraphNodeProjection.svelte.test.ts` exceeded the 500-line decomposition threshold while covering list, detail, create, field, and edge cache behavior. Read/cache behavior and command cache-write behavior were split into separate test files before schema editor work.

## Concurrent Worker Policy

No parallel worker execution is assumed by this plan.

If implementation is split across workers later, add a worker-wave plan before delegation that defines:

- non-overlapping primary write sets,
- allowed adjacent write sets,
- forbidden/shared files,
- one owner for schemas, generated contracts, lockfiles, shared DTOs, fixtures, and global tooling,
- report paths,
- integration order,
- cleanup requirements.

Current worker wave:

- Local owner: schema, contracts, command handlers, route registration, shared DTOs, plan updates, and final integration commits.
- Backend explorer owner: inspect server graph persistence and route seams for the smallest canonical bible graph node vertical slice. Primary output is a report only; no file writes.
- Frontend explorer owner: inspect bible UI/component seams for a projection-driven detail/editor slice that avoids mutating `storyState.entities`. Primary output is a report only; no file writes.
- Forbidden shared files for workers: `Cargo.lock`, package lockfiles, generated bindings, migration/schema files, `ui/src/lib/types.ts`, and this plan unless explicitly assigned in a later wave.
- Integration order: local schema/contract slice first, backend command/projection slice second, frontend projection consumer slice third.
- Cleanup requirement: close or reuse workers after their reports; do not leave parallel implementation branches with overlapping write sets.

## Concurrency And Race-Risk Review

The refactor touches async state, background work, WebSocket delivery, AI generation, Y.Doc, persistence, and renderer bridges. Each implementation slice must record the owner for:

- task startup and shutdown,
- cancellation signals,
- queue capacity and overflow behavior,
- retry/idempotency behavior,
- panic/error handling,
- subscription teardown,
- restart/recovery behavior.

The first persistence and event-history slice must prove durable command idempotency before propagation or renderer bridge concurrency expands.

## Scope

In scope:

- Timeline nodes as timed context chunks.
- Composable story bible graph, schemas, parts, edges, snapshots, assets, and references.
- Script document, script segments, blocks, spans, locks, provenance, and patch proposals.
- Semantic claims, dependencies, propagation proposals, change review, undo/redo, and before/after history.
- SQLite schema and repositories for canonical state, revisions, projections, and assets.
- Command, event, and projection DTOs.
- Bevy timeline viewport and Bevy bible graph view as projection consumers.
- Svelte shell, forms, inspectors, editors, and accessibility command alternatives.
- Tests, documentation, lifecycle management, validation, and dependency placement required by standards.

Out of scope:

- Compatibility migrations for old project files unless a later product decision explicitly adds them.
- Keeping the DOM/SVG timeline as a supported renderer.
- Keeping Beat node content as final screenplay storage.
- Keeping fixed `EntityCategory` / `EntityDetails` as canonical story bible storage.
- Keeping cloned `Project` snapshots as canonical undo/redo.

## Architecture Contract

Dependency direction:

```text
Svelte / Bevy presentation -> application commands/projections -> domain -> infrastructure adapters
```

Canonical backend-owned state:

```text
timeline context chunks
bible graph rows
script document rows
change events and object revisions
semantic claims and dependencies
assets and references
```

Projection consumers:

```text
AI context projection
script editor projection
bible detail projection
timeline render projection
bible 3D render projection
change review projection
export projection
```

## Milestone 1: Contracts And Deletion Plan

Tasks:

- Define command, event, object revision, projection, and ID/value newtypes.
- Define initial DTOs for bible graph, script document, timeline render, bible render graph, AI context, and change review.
- Decide whether Y.Doc remains as an active editor transport only.
- Mark old canonical paths for deletion: `Entity`, `EntityDetails`, Beat-owned screenplay text, node script routes, snapshot undo, DOM/SVG timeline, 2D SVG relationship graph.
- Add or update README/ADR documentation for command ownership, projection ownership, persistence ownership, and renderer ownership.
- Confirm existing lint, format, typecheck, and test baseline before implementation.

Verification:

- DTO serialization round-trip tests for public command/event/projection contracts.
- Compile-time checks for new validated types.
- Documentation review against `DOCUMENTATION-STANDARDS.md` required sections for touched `src/` directories.

## Milestone 2: Persistence, History, And Validation Foundation

Tasks:

- Add SQLite tables for graph, script, event history, object revisions, semantic dependencies, assets, and projections.
- Replace clear-and-rewrite save behavior for new canonical data with transactional command writes.
- Add centralized validators for paths, URLs, IDs, names, time ranges, field values, and command payloads.
- Add command idempotency and event revision base checks.
- Add projection rebuild from persisted state.
- Add bounded queues for propagation, projection updates, WebSocket events, Bevy bridge events, and AI work.

Verification:

- Per-field bible update writes only affected rows.
- Script patch preserves locked/user-authored spans.
- Undo reverses only the selected event's revisions.
- Redo reapplies only if base revision still matches.
- Projection rebuild from persisted database matches current-state tables.
- Duplicate command IDs are idempotent.
- Partial failure cannot leave an accepted event without required revisions.
- Asset path and URL validators reject escape paths and disallowed URL schemes.

## Milestone 3: Story Bible Graph Replacement

Tasks:

- Replace `StoryBible { entities }` with canonical graph rows and schema-driven defaults.
- Implement canonical roots as system-owned graph nodes, not special enum branches.
- Replace entity routes with graph command/projection routes.
- Replace category-specific bible UI with schema-driven node, part, edge, snapshot, asset, and schema editors.
- Move AI extraction to semantic claim proposals.
- Delete old entity snapshot and fixed detail APIs after graph equivalents exist.

Verification:

- Vertical slice: create bible graph node -> set field -> query projection -> UI/API receives updated view.
- Graph invariant tests for parent/child structure, edge validity, schema field validation, and snapshot time scoping.
- Query/index tests for field-level lookup and dependency lookup.

## Milestone 4: Script Ownership Replacement

Tasks:

- Add canonical `ScriptDocument`, `ScriptSegment`, `ScriptBlock`, `ScriptSpan`, `ScriptLock`, and `ScriptPatch` models.
- Remove final screenplay ownership from timeline nodes.
- Replace Beat-based script display with script-document editor/viewer.
- Store generation output in script segments.
- Add lock/protected-span behavior and provenance.
- Replace `/nodes/{id}/script` as a screenplay write path with script document commands.
- Make export consume `ExportProjection` from `ScriptDocument`.

Verification:

- Vertical slice: manual script edit -> semantic claim proposal -> accept bible field change -> affected segment marked stale.
- Vertical slice: accepted script patch -> locked span preserved -> export projection includes final screenplay.
- Unit tests for script patch conflict handling, protected spans, provenance, and formatting projection.

## Milestone 5: Semantic Propagation And Review

Tasks:

- Replace current consistency reaction with semantic claim/dependency impact analysis.
- Add staged proposal states for bible field changes, snapshots, script patches, and regeneration work.
- Add change review UI.
- Add undo/redo over accepted change events.
- Remove cloned-project undo as canonical undo/redo.
- Ensure AI never commits graph/script state directly.

Verification:

- Vertical slice: undo accepted bible change -> graph projection reverts -> dependent script segment status updates.
- Before/after diff tests for graph and script state.
- Proposal acceptance/rejection/edit tests.
- Restart/recovery tests for accepted and pending proposals.

## Milestone 6: Bevy Timeline Renderer

Tasks:

- Add renderer-facing timeline projection DTOs.
- Add Bevy timeline host as an isolated leaf renderer or frontend-owned package.
- Add pan, zoom, playhead, selection, hit testing, move, resize, split, arcs, relationship curves, and valence/arousal overlays through backend-confirmed commands/projections.
- Add keyboard-accessible Svelte command alternatives for critical timeline operations.
- Remove the DOM/SVG timeline renderer after Bevy covers target interactions.

Verification:

- Vertical slice: Bevy timeline render model receives projection and emits a validated command.
- Pointer, focus, keyboard, and parent-gesture conflict smoke checks.
- Projection serialization tests.
- Renderer lifecycle tests for mount/unmount subscription cleanup.
- Dependency review proving Bevy is not in `eidetic-core` and is justified/feature-gated if it adds 100+ transitive dependencies.

## Milestone 7: Bevy Bible Graph View

Tasks:

- Add `BibleRenderGraph` projection DTOs.
- Add pure adapter from bible graph to render graph.
- Add deterministic layout helpers and selection/neighborhood indexes.
- Add Bevy bible graph host as a projection consumer.
- Add Svelte filters, detail panels, and accessibility command alternatives.
- Replace the 2D SVG relationship graph after Bevy graph projections cover target interactions.

Verification:

- Vertical slice: Bevy bible graph render model receives projection and emits selection/inspect command.
- Layout helper unit tests.
- Selection/neighborhood index tests.
- Projection bounding tests for large graph neighborhoods.
- Renderer lifecycle cleanup tests.

## Cross-Cutting Implementation Requirements

Code organization:

- Files over 500 lines require decomposition review.
- UI components over 250 lines require decomposition review.
- Modules/services with more than roughly 7 public functions or 3 responsibilities require decomposition review.
- Directories under `src/` touched by the refactor must have current READMEs.

Concurrency:

- Domain/core APIs stay synchronous unless concurrent I/O is intrinsic.
- Every `tokio::spawn` must be owned by a lifecycle manager with tracked handles, cancellation, shutdown, panic logging, and draining.
- No lock guards may be held across `.await` unless the lock type and design explicitly support it.
- Related mutable state has one owner.

Interop:

- HTTP, WebSocket, Y.Doc bridge, and Bevy bridge payloads are trust boundaries.
- Validate every boundary payload before dispatch.
- Use explicit serde wire shapes and round-trip tests.
- Unsubscribe all event/bridge subscriptions on teardown.
- Document thread requirements for Bevy bridge calls and callbacks.

Frontend and accessibility:

- Svelte remains declarative except isolated Bevy/canvas hosts.
- Non-canvas commands use semantic HTML controls.
- Icon-only controls have accessible names.
- Canvas/Bevy critical actions have keyboard-accessible Svelte command equivalents.
- Dialogs, proposal review, lock controls, undo/redo, graph selection, and timeline commands meet keyboard/focus requirements.

Security:

- Validate project, asset, import, export, and reference paths through one backend validator.
- Validate URLs by parsing and scheme allowlisting.
- Use checked arithmetic for external dimensions, lengths, and ranges.
- Bind local listeners to loopback and enforce connection limits.

Dependencies:

- Bevy and other heavy renderer dependencies stay out of `eidetic-core`.
- Dependencies are declared by the package/crate that directly uses them.
- Heavy optional features are feature-gated.
- Dependency cost is measured before adding renderer/RAG/export dependencies.

Tooling:

- Warnings are errors in CI.
- Generated contracts are deterministic and verified.
- Persisted schema fixtures, command fixtures, projection fixtures, and sample events have validation hooks.

## Baseline Verification Commands

Use the repository's actual scripts where they differ, but the implementation plan must provide equivalents for:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo check --workspace --all-features
cargo check --workspace --no-default-features
npm run lint
npm run typecheck
npm run test
```

Performance-sensitive graph queries, projection rebuilds, and renderer projection serialization require Criterion benchmarks before performance claims or regression budgets are accepted.

## Risks And Mitigations

- Dual truth between old and new canonical models: delete old ownership paths in the same milestone where replacements become active.
- Bevy becoming a second state owner: Bevy receives projections and emits commands only.
- AI committing state silently: AI outputs proposals; accepted proposals go through command handlers.
- JSON reappearing as canonical storage: enforce relational rows for indexed facts, claims, dependencies, locks, and revisions.
- Standards deferred until late: every milestone includes verification and documentation gates.
- Oversized replacement modules: decompose by domain and projection ownership before implementation expands.
- Recovery paths untested: replay/rebuild/idempotency tests are required before propagation and renderer expansion.
- Heavy dependencies leaking into core: keep renderer/RAG/export dependencies in leaf crates or feature-gated packages.

## Re-Plan Triggers

Re-plan before continuing if:

- A required contract crosses more boundaries than expected.
- A proposed implementation needs a second source of truth.
- A milestone cannot delete the old ownership path after its replacement lands.
- Y.Doc must remain canonical for any durable script state.
- Bevy integration requires dependencies in `eidetic-core`.
- A vertical slice cannot be tested through real layer boundaries.
- Persistence requires JSON for queryable canonical data.
- Runtime bridge design cannot provide deterministic shutdown and subscription cleanup.
- Accessibility command alternatives cannot cover critical canvas/Bevy actions.

## Completion Criteria

The refactor is complete when:

- Backend-owned SQLite command/event/revision state is the only persistent source of truth.
- Timeline nodes own context only, not final screenplay text.
- Script documents own the generated screenplay artifact.
- Bible graph rows own world/story/production facts.
- Accepted changes are traceable through events, object revisions, and semantic dependencies.
- Undo/redo works through event revisions after restart.
- AI graph/script changes are reviewable proposals before acceptance.
- Svelte and Bevy consume versioned projections and submit commands only.
- The DOM/SVG timeline renderer is removed.
- The 2D SVG relationship graph is replaced by the Bevy bible graph or removed as a supported graph view.
- Standards-required tests, docs, lifecycle owners, validators, and dependency reviews are present for all touched areas.
