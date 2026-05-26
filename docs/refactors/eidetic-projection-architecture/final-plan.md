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
- Current product direction: finish making Svelte a projection consumer while
  adding backend-owned floating Bevy renderer windows for realtime surfaces.
  The bible graph uses the shared floating renderer host first; the timeline
  reuses that host later instead of embedding Bevy into the WebView or running
  a renderer sidecar as a second application layer.

## Standards Reviewed

Implementation must comply with:

- `CODING-STANDARDS.md`
- `COMMIT-STANDARDS.md`
- `ARCHITECTURE-PATTERNS.md`
- `PLAN-STANDARDS.md`
- `TESTING-STANDARDS.md`
- `DOCUMENTATION-STANDARDS.md`
- `FRONTEND-STANDARDS.md`
- `ACCESSIBILITY-STANDARDS.md`
- `CONCURRENCY-STANDARDS.md`
- `INTEROP-STANDARDS.md`
- `LANGUAGE-BINDINGS-STANDARDS.md`
- `SECURITY-STANDARDS.md`
- `DEPENDENCY-STANDARDS.md`
- `TOOLING-STANDARDS.md`
- `CROSS-PLATFORM-STANDARDS.md`
- `LAUNCHER-STANDARDS.md`
- `RELEASE-STANDARDS.md`
- `languages/rust/RUST-API-STANDARDS.md`
- `languages/rust/RUST-ASYNC-STANDARDS.md`
- `languages/rust/RUST-SECURITY-STANDARDS.md`
- `languages/rust/RUST-INTEROP-STANDARDS.md`
- `languages/rust/RUST-DEPENDENCY-STANDARDS.md`
- `languages/rust/RUST-TOOLING-STANDARDS.md`
- `languages/rust/RUST-CROSS-PLATFORM-STANDARDS.md`

## Standards Compliance Gates

This 2026-05-22 standards pass makes the following gates non-negotiable for
Milestones 8-12 and their blast radius.

Architecture and package boundaries:

- Contracts must be defined and reviewed before implementation. DTOs that cross
  Tauri, Bevy, AI-provider, persisted-command, or generated-TypeScript
  boundaries must use explicit serde wire shapes, validated IDs/enums, runtime
  decoders, and round-trip tests.
- `eidetic-core` and contract modules may contain DTOs, validated types, pure
  adapters, and synchronous domain helpers only. They must not depend on
  SQLite, Tauri, Axum, Bevy, Svelte, model providers, filesystem watchers, or
  runtime process management.
- `eidetic-server` owns SQLite persistence, command handlers, query services,
  graph/context/affect/agent orchestration, and provider adapters behind narrow
  traits. It must not depend on Svelte components, Bevy ECS/render internals, or
  Tauri runtime types.
- `src-tauri` remains the desktop composition root. It owns command/event
  transport wiring, runtime startup/shutdown, task lifecycle, and error
  mapping; it must not contain reusable domain workflow logic or direct SQL
  mutation paths.
- Bevy graph and timeline crates are leaf renderers. They consume versioned
  projections, own only transient camera/hover/animation/simulation state, and
  emit validated command requests. They must not own persistence, AI workflows,
  durable selections, or project facts.
- App-managed floating Bevy renderer windows are the supported native visual
  surface for graph and timeline work. The desktop renderer host owns window
  lifecycle, focus/status, projection subscription, command draining, and
  teardown; renderer crates own only disposable scene state for their renderer
  kind. The Bevy renderer must not own business logic or canonical project
  state. A split-process renderer transport is not the production target and
  would require a separate standards review before being introduced.
- The floating renderer host is infrastructure owned by the desktop
  composition root. Svelte components may request launch/focus/close and render
  status projections, but they must not own renderer lifecycle state machines,
  background task handles, command queues, or projection subscriptions.
- The superseded embedded viewport/child-surface code path must be retired,
  renamed, or quarantined before more renderer behavior is added. New Milestone
  8 or 11 implementation must not extend X11-only WebView child-window
  attachment APIs as a production path.
- Any platform-specific renderer/window behavior must live behind a
  platform-strategy/factory boundary selected by the desktop composition root.
  Do not put `cfg`, OS checks, raw-window-handle branching, or Wayland/X11/
  Win32/AppKit logic in domain services, Svelte stores, graph/timeline
  projection adapters, or renderer-independent contracts.
- Svelte owns projection caches, local form drafts, focus, filters, hover,
  panel sizing, and other transient UI state only. Backend-owned data must not
  be optimistically mutated in stores or components.

Storage and canonical state:

- SQLite relational rows remain canonical for queryable bible facts, context
  influence records, semantic dependencies, affect values, script spans, locks,
  proposals, agent runs, tool calls, command events, and revisions.
- JSON may be used only for opaque payload snapshots or bounded diagnostic
  detail that is not queried, merged, partially updated, or treated as a source
  of truth.
- Large assets, reference media, URLs, and imported/exported files must be
  represented by SQLite metadata plus validated project-relative file paths or
  validated URL records.
- Every durable write must enter through an idempotent command path that records
  history before the change is exposed through projections.

Code organization and documentation:

- Do not add responsibilities to files already over standards thresholds.
  `projection_service.rs`, `AppShell.svelte`, and graph persistence modules
  must be decomposed before new graph, affect, renderer, or harness scope is
  added to them.
- Files over 500 lines, Svelte components over 250 lines, and modules/services
  with more than roughly 7 public functions or 3 distinct responsibilities need
  an explicit decomposition decision before expansion.
- Any touched source directory must have an up-to-date `README.md` or an ADR
  explaining the boundary, contracts, invariants, alternatives rejected, and
  revisit triggers.
- Host-facing or machine-consumed directories must document API consumer
  contracts and structured producer contracts, including lifecycle, ordering,
  validation, compatibility, and regeneration rules.

Concurrency, lifecycle, and recovery:

- Domain/core work stays synchronous unless concurrent I/O is part of the
  contract. Async belongs at Tauri, provider, database, filesystem, IPC, and
  other I/O boundaries.
- Every background task, renderer bridge, provider call loop, event pump, or
  agent workflow must have a single lifecycle owner with tracked handles,
  cancellation, shutdown, panic logging, and deterministic cleanup.
- Floating renderer windows must have an explicit owner with idempotent
  start/focus/close operations, tracked renderer thread/task handles, bounded
  command queues, cancellation or stop signaling, shutdown joining, panic
  reporting, and restart/reopen behavior documented before native render work
  proceeds.
- Queues that accept frontend, renderer, provider, or tool input must be
  bounded and must document whether overflow rejects, drops, or backpressures.
- No detached `tokio::spawn` patterns are allowed. No synchronous or parking
  lock guard may be held across `.await`.
- Durable multi-step operations must be transactional, idempotent, or have
  explicit compensation/replay behavior. Projections must rebuild correctly
  after restart.

Interop and security:

- Tauri commands/events, Bevy bridges, model-provider responses, agent tool
  calls, imported/exported artifacts, URLs, paths, and persisted command
  payloads are trust boundaries.
- Floating renderer requests, size/placement hints, command drains, selection
  intents, and renderer status payloads are trust boundaries. Validate incoming
  IDs/enums/dimensions before dispatch, use checked arithmetic before renderer
  allocation or hit-test math, and reject malformed renderer commands before
  any backend mutation.
- Validate once at the boundary into typed/newtyped values, reject unknown or
  malformed payloads before dispatch, then pass validated values inward.
- Provider URLs and external reference URLs must be parsed and scheme/host
  allowlisted by backend policy. File paths must use a single backend path
  validator and canonicalized project containment checks.
- LLM and agent tool requests must never accept raw SQL, raw filesystem paths,
  raw renderer payloads, or unbounded graph reads. Tool payloads must use typed
  IDs, limits, budgets, and reviewable proposal records.
- Boundary errors must preserve source causes and bounded correlation context
  without logging secrets, unbounded prompt bodies, binary assets, or large
  provider responses.

Frontend and accessibility:

- Svelte rendering stays declarative except isolated Bevy host surfaces. Any
  direct DOM/canvas/WebGL integration must be isolated and cleaned up in
  lifecycle teardown.
- Non-canvas controls use semantic HTML. Icon-only controls require accessible
  names. Dialogs and review surfaces must manage focus and Escape behavior.
- Every critical Bevy/canvas graph or timeline action must have a
  keyboard-accessible Svelte command alternative backed by the same backend
  command path.
- Floating renderer launch/focus/close controls must use semantic controls,
  accessible names, visible focus states, Escape/close behavior where
  applicable, and focus return to the invoking Svelte control after a renderer
  window is closed.
- Frontend tests must use accessible selectors where possible and include
  keyboard interaction coverage for new graph, timeline, affect, and proposal
  controls.

Dependencies, tooling, and release:

- Bevy 0.18.1 and other heavy renderer/provider dependencies must remain in
  leaf crates/packages that directly execute them. If a dependency adds 100+
  transitive dependencies, document the justification and feature-gate it where
  practical.
- Native window/render features for Bevy must be dependency-reviewed as part of
  the renderer-owning crate, with `cargo tree`/feature notes proving the cost is
  isolated from `eidetic-core`, `eidetic-server` query logic, and Svelte build
  tooling.
- Dependency ownership must match the execution boundary. Package-local
  commands must declare their own runtime/test/build dependencies instead of
  relying on root hoisting or incidental transitive crates.
- Removing Axum/WASM paths must include dependency cleanup checks so unused
  web-server or wasm bridge dependencies do not remain.
- `launcher.sh` remains the canonical entry point for run/build/test/release
  smoke flows and must keep managed state under `.launcher-state` unless the
  operator opts out.
- Atomic implementation commits must use conventional commit format and include
  code, tests, docs, schema, and fixtures for the verified slice they complete.

Verification gates:

- The first change in each cross-layer milestone must be a thin vertical slice
  with an acceptance test that crosses the real producer/consumer boundaries.
- Durable command/projection work must include replay, reload, duplicate
  command, partial failure, and idempotency coverage where applicable.
- Tauri/Bevy/TypeScript/Rust boundary changes require native-side contract
  tests, host-side smoke tests, and serde/TypeScript round-trip coverage.
- Graph query, context influence, affect overlay, and renderer projection hot
  paths need deterministic unit tests and Criterion benchmarks before
  performance claims or regression budgets are accepted.
- Required local/CI equivalents remain formatting, clippy with warnings denied,
  Rust tests including doctests, all-features/no-default-features checks where
  applicable, frontend lint/typecheck/tests, decision traceability, dependency
  review, and launcher release-smoke coverage.

## Hard Constraints

- No backwards compatibility with current project data structures is required.
- Old canonical paths must be removed when replacements land.
- Backend remains the only source of truth for persistent data and business decisions.
- No optimistic UI updates for backend-owned state.
- Frontend and Bevy may cache projections only as versioned read models that can be discarded and rebuilt.
- Frontend stores may own only transient interaction state or discardable projection caches; any store that owns durable project/story/script/bible state is a deletion target.
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
- Existing per-project persistence, AI runtime modules, Y.Doc integration, export,
  and timeline/story/script modules.
- Existing documentation and ADR structure under `docs/`.

External dependencies:

- SQLite for local durable state.
- Bevy for the replacement timeline renderer and bible graph renderer.
- Existing AI backend dependencies.
- Y.Doc only if retained as active editing transport/cache.

## Affected Contracts And Artifacts

Structured contracts affected:

- Current HTTP command payloads and projection responses, until replaced by
  Tauri command contracts.
- Current WebSocket event envelopes, until replaced by Tauri event contracts or
  bounded event-drain contracts.
- Tauri command/event payloads and generated or mirrored TypeScript bindings.
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
- `feat(ui): cache timeline split projections` routed split timeline command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route timeline split through commands` moved the existing DOM timeline split handler onto the backend timeline split command/projection path instead of the legacy split route.
- `feat(renderer): emit timeline split commands` added validated renderer command emission for node split requests so future Bevy split interactions can flow through backend-confirmed timeline commands.
- `feat(server): add timeline delete node command` added a backend command route for validated timeline node deletion that returns the updated timeline render projection.
- `feat(ui): add timeline delete command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed timeline node deletion commands.
- `feat(ui): cache timeline delete projections` routed delete timeline command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route timeline deletion through commands` moved timeline clip deletion and the delete-key shortcut onto the backend timeline delete command/projection path instead of the legacy delete route.
- `feat(renderer): emit timeline delete commands` added validated renderer command emission for node deletion requests so future Bevy delete interactions can flow through backend-confirmed timeline commands.
- `feat(server): add timeline create node command` added a backend command route for validated timeline node creation that returns the updated timeline render projection.
- `feat(ui): add timeline create command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed timeline node creation commands.
- `feat(ui): cache timeline create projections` routed create timeline command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route timeline double-click create through commands` moved the existing DOM timeline double-click create handler onto the backend timeline create command/projection path.
- `feat(renderer): emit timeline create commands` added validated renderer command emission for node creation requests so future Bevy create interactions can flow through backend-confirmed timeline commands.
- `refactor(ui): route timeline gap fill through commands` moved the existing gap-fill interaction onto the backend timeline create command/projection path instead of the legacy fill-gap route.
- `refactor(ui): remove legacy structural timeline helpers` removed unused frontend helpers for legacy create, update, delete, split, resize, and fill-gap timeline mutation routes after their UI callers moved to command/projection paths.
- `refactor(server): remove legacy structural timeline routes` removed legacy create, update, delete, split, resize, and fill-gap timeline mutation routes after replacement command/projection paths landed.
- `feat(server): add timeline apply children command` added a backend command route for replacing a node's child timeline structure and returning the updated timeline render projection without mutating legacy bible entities.
- `feat(ui): add timeline apply children command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed child timeline replacement commands.
- `feat(ui): cache timeline apply children projections` routed child timeline replacement command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route child planning through timeline commands` moved BeatEditor child-plan application onto the backend timeline apply-children command/projection path instead of the legacy apply-children route.
- `refactor(ui): remove legacy apply children helper` removed the unused frontend helper for the legacy child-plan application route after BeatEditor moved to command/projection APIs.
- `refactor(server): remove legacy apply children route` removed the legacy child-plan application route and its direct story-bible entity mutation side effects after the command/projection replacement landed.
- `feat(server): add timeline relationship commands` added backend command routes for creating and deleting timeline relationships, returning updated timeline render projections and covering invalid endpoints/unknown relationship IDs.
- `feat(ui): add timeline relationship command helpers` added focused TypeScript command DTOs and frontend API helpers for backend-confirmed relationship create/delete commands.
- `feat(ui): cache timeline relationship projections` routed relationship create/delete command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route relationship creation through commands` moved timeline connection-drag relationship creation onto backend relationship commands instead of the legacy relationship route.
- `refactor(ui): remove legacy relationship helpers` removed unused frontend helpers for the legacy timeline relationship mutation routes after UI relationship creation moved to commands.
- `refactor(server): remove legacy relationship routes` removed the legacy timeline relationship mutation routes after replacement command/projection paths landed.
- `refactor(ui): remove legacy track mutation UI` removed the track-delete context menu and unused frontend track CRUD helpers because timeline tracks are projection lanes, not user-owned canonical state.
- `refactor(server): remove legacy track routes` removed legacy timeline track CRUD routes so clients cannot delete or reshape required projected story-level lanes through direct mutation endpoints.
- `refactor(ui): remove legacy node arc helpers` removed unused frontend helpers for legacy timeline node-arc tagging routes.
- `refactor(server): remove legacy node arc routes` removed legacy timeline node-arc tagging routes, leaving the timeline route module read-only while future arc edits wait for command/projection contracts.
- `feat(server): add timeline node lock command` added a backend command route for setting timeline node lock state and returning the updated timeline render projection.
- `feat(ui): add timeline node lock command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed node lock commands.
- `feat(ui): cache timeline node lock projections` routed node lock command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route node locking through commands` moved BeatEditor lock toggles onto backend timeline node lock commands instead of legacy node lock/unlock routes.
- `refactor(script): remove legacy node lock routes` removed unused frontend lock/unlock helpers and legacy node lock/unlock routes after lock state moved to timeline commands.
- `feat(server): add timeline node notes command` added a backend command route for setting timeline node notes, mirroring notes into Y.Doc, and returning updated timeline render projections.
- `feat(ui): add timeline node notes command helper` added focused TypeScript command DTOs and a frontend API helper for backend-confirmed node notes commands.
- `feat(ui): cache timeline node notes projections` routed node notes command responses into the timeline render projection cache without optimistic local patching.
- `refactor(ui): route node notes through commands` moved BeatEditor debounced note saves onto backend timeline node notes commands instead of the legacy node notes route.
- `refactor(script): remove legacy node notes route` removed the unused frontend notes helper and legacy node notes mutation route after notes moved to timeline commands.
- `refactor(ui): remove legacy relationship graph panel` removed the old 2D SVG relationship graph panel and toolbar toggle that rendered directly from broad legacy entity state.
- `refactor(ui): remove legacy extraction websocket event` removed the stale `entity_extraction_complete` websocket contract and handler from the deleted extraction workflow.
- `refactor(ui): remove legacy entity detail panel` removed stale legacy entity selection state and unused entity detail/development timeline components after bible details moved to graph projections.
- `refactor(ui): remove unused legacy entity helpers` removed unused frontend helpers for legacy entity CRUD, snapshots, relations, node-ref add, and resolve-at-time APIs after graph projection UI replacement.
- `refactor(server): remove unused legacy entity routes` removed unused legacy entity CRUD, snapshot, relation, node-ref add, and resolve-at-time routes after graph projection replacement, leaving only active legacy entity read/unlink paths.
- `refactor(ui): remove legacy character timeline` removed the optional character progression lane, toolbar toggle, shortcut, transient store, and layout budget that rendered from broad legacy entity state instead of backend-owned projections.
- `refactor(ui): remove legacy node entity links` removed BeatEditor linked-entity chips, timeline clip entity dots, script entity highlighting, frontend entity list/unlink helpers, websocket entity refreshes, and broad `storyState.entities` ownership.
- `refactor(server): remove legacy entity link routes` removed the final legacy `/bible/entities` read and node-ref unlink routes after frontend node-entity link consumers were deleted.
- `refactor(ui): decouple graph categories from entities` moved bible graph UI category typing onto the graph category adapter instead of importing the legacy `EntityCategory` contract.
- `refactor(ui): make project bible payload opaque` removed detailed legacy entity DTOs from the frontend type surface and made `Project.bible` opaque so UI code cannot rebuild against old entity internals.
- `refactor(core): stop prompting from legacy bible entities` removed production AI prompt dependence on `Project.bible.entities` so generation waits for a graph-backed AI context projection instead of reading stale legacy bible state.
- `refactor(core): remove legacy bible prompt helpers` deleted unused legacy bible context gathering and entity prompt-text helpers after AI prompt construction stopped reading `StoryBible.entities`.
- `refactor(core): remove legacy extraction contract` deleted the orphaned AI extraction trait method and legacy extraction DTOs after extraction routes and side effects were removed.
- `refactor(server): remove legacy json bible migration` removed the old JSON character-to-entity migration path so legacy JSON loading no longer populates `StoryBible.entities`.
- `refactor(server): stop persisting legacy bible entities` removed legacy entity tables, save/load loops, and v1 entity readers from project SQLite persistence so project persistence no longer owns story bible entity state.
- `refactor(core): remove project bible field` removed `Project.bible` from shared core/server/frontend project DTOs after project persistence stopped loading or saving legacy bible entities.
- `refactor(core): remove entity-driven timeline relationship` removed the legacy timeline relationship variant that referenced bible entity IDs; bible semantics now belong to graph edges/projections.
- `refactor(ai): remove legacy bible context field` removed the always-empty legacy bible entity context from AI request DTOs and prompt formatting; future AI context should come from graph-backed projections.
- `refactor(core): remove legacy story bible module` deleted the old `story::bible` entity/snapshot/relation module after project, timeline, persistence, and AI request contracts stopped depending on it.
- `refactor(ai): remove unused context packing helpers` deleted the dead generic AI context-packing module after request DTOs stopped carrying legacy bible context, and updated AI module documentation around the remaining prompt request assembly surface.
- `refactor(server): stop snapshotting arc mutations` removed the final route-level cloned-project snapshot writes and now-unused snapshot push helper from legacy story arc create/update/delete handlers while leaving the current arc mutation behavior intact until arc command/projection contracts replace it.
- `refactor(project): remove empty snapshot undo routes` deleted the cloned-project undo/redo routes, websocket event, frontend API helpers, shortcuts, toolbar controls, and transient UI flags after snapshot producers were removed.
- `refactor(core): remove legacy node content aliases` removed legacy `generated_text` / `user_refined_text` node-content deserialization and old content-status aliases from core because compatibility with old project data is outside this refactor.
- `refactor(server): remove legacy project load migrations` removed server-side JSON project loading, JSON sibling auto-migration, v1 SQLite migration helpers, and JSON project discovery so project loading accepts only the current SQLite schema.
- `feat(story): add arc command projection bridge` added story arc command/projection contracts and a backend applicator that returns arc-list projections while still updating `Project.arcs` for the current UI bridge.
- `refactor(story): route legacy arc mutations through commands` moved the existing `/arcs` create/update/delete endpoints onto the story arc applicator so the remaining UI bridge no longer owns independent arc mutation logic.
- `feat(ui): add story arc command projection helpers` added focused frontend DTOs plus command/projection API helpers for backend-owned story arc mutations and arc-list reads without switching UI callers yet.
- `refactor(ui): route story arc UI through projections` added a story arc projection store, routed arc sidebar create/update/delete and websocket refresh through backend command/projection helpers, and removed unused frontend legacy arc mutation helpers.
- `refactor(server): remove legacy arc mutation routes` deleted the obsolete `/arcs` list/create/update/delete route surface after UI callers moved to story arc command/projection APIs, leaving only the current progression analysis route.
- `refactor(story): project arc progression analysis` replaced the final legacy `/arcs/progression` read path with a backend story arc progression projection, updated the Svelte analysis panel to consume the projection, and fixed the frontend contract from stale `beat_count` to backend `node_count`.
- `refactor(ui): hydrate arcs from projections` stopped seeding sidebar arc state from the broad project payload during project creation/load and refreshes the backend story arc projection instead.
- `refactor(ui): remove story arc bridge store` deleted the separate frontend `storyState` arc cache and moved arc sidebar/timeline consumers to the backend story arc projection cache directly.
- `refactor(ui): remove project arc dto field` removed `Project.arcs` from the frontend project DTO so arc UI cannot accidentally rebuild broad project-payload ownership instead of consuming story arc projections.
- `feat(ui): expose cached timeline render model` added a projection-store helper that derives the deterministic timeline render model from the cached backend timeline projection for future DOM/Bevy timeline consumers.
- `refactor(ui): hydrate timeline render projection` refreshes the backend timeline render projection during project create/load alongside story arc projections so future timeline consumers do not start from only the broad project timeline payload.
- `feat(story): shadow record arc command history` records story arc create/update/delete command IDs, change events, and `StoryArc` object revisions into the existing history tables while preserving `Project.arcs` as the current read model.
- `feat(timeline): shadow record node range history` records timeline node move/resize command IDs and `TimelineNode` range revisions into the existing history tables while preserving the current in-memory timeline read model.
- `feat(timeline): shadow record node lock history` records timeline node lock command IDs and `TimelineNode` lock revisions into the existing history tables while preserving the current in-memory timeline read model.
- `refactor(ui): render relationships from timeline projections` moved the DOM relationship overlay to the cached backend timeline render projection model instead of reading relationships and node geometry from the broad timeline store.
- `feat(timeline): shadow record node notes history` records timeline node notes command IDs and `TimelineNode` notes/status revisions into the existing history tables while suppressing duplicate replay Y.Doc writes.
- `feat(timeline): shadow record relationship create history` records timeline relationship create command IDs and `TimelineRelationship` revisions into the existing history tables while suppressing duplicate replay relationship insertion.
- `feat(timeline): shadow record relationship delete history` records timeline relationship delete command IDs and `TimelineRelationship` delete revisions into the existing history tables while suppressing duplicate replay side effects.
- `feat(timeline): shadow record node create history` records timeline node create command IDs and `TimelineNode` create revisions into the existing history tables while suppressing duplicate replay node creation and Y.Doc ensure side effects.
- `refactor(server): split timeline command history` extracted timeline command history-recording helpers from the timeline mutation applicator to restore module decomposition before adding node-delete and child-replacement history.
- `feat(timeline): shadow record node delete history` records timeline node delete command IDs, subtree `TimelineNode` delete revisions, and deleted relationship revisions into the existing history tables while suppressing duplicate replay deletion/Y.Doc side effects.
- `feat(timeline): shadow record node split history` makes split commands deterministic with caller-supplied replacement node IDs, records original/delete and replacement/create revisions plus child/relationship rewrites, and suppresses duplicate replay side effects.
- `feat(timeline): shadow record child replacement history` records timeline child-replacement command IDs, deleted child-subtree revisions, replacement child create revisions, and removed relationship revisions while suppressing duplicate replay node/Y.Doc side effects.
- `refactor(server): split timeline command routes` extracted timeline command route handlers into a focused route module while preserving the existing command router surface and reducing the aggregate command module below the decomposition threshold.
- `test(server): split timeline command route tests` moved timeline command route coverage and helpers into focused out-of-line test modules owned by the timeline command route module.
- `test(server): split non-timeline command route tests` moved object/story, bible graph, and script command route coverage into focused out-of-line test modules owned by the aggregate command route module.
- `feat(story): persist story arc current state` added a focused SQLite story arc current-state store, writes arc create/update/delete current rows in the same transaction as command history, routes story arc list/progression projections through SQLite-backed arcs, and split projection route tests to keep touched modules under decomposition thresholds.
- `refactor(story): stop broad saves for arc commands` removed full project save scheduling from story arc command routes because arc command persistence is now handled by transactional SQLite current-state and history writes.
- `refactor(story): validate arc commands from sqlite` moved story arc command validation and revision delta reads from the in-memory project mirror to the SQLite story arc store, and refreshes `Project.arcs` from SQLite only as a transitional compatibility mirror.
- `refactor(ui): move story arc dto ownership` moved frontend story arc, arc color/type, progression, and color helper contracts into `storyArcTypes.ts`, keeping compatibility re-exports from `types.ts` while routing arc consumers to the focused module.
- `refactor(ui): move ai runtime dto ownership` moved frontend AI status/config, diffusion status/request, and model list contracts into `aiTypes.ts`, keeping compatibility re-exports from `types.ts` while routing API, editor state, and AI config panel consumers to the focused module.
- `refactor(ui): move websocket dto ownership` moved frontend websocket server message contracts into `wsTypes.ts`, keeping a compatibility re-export from `types.ts` while routing the websocket client to the focused module.
- `refactor(ui): move project dto ownership` moved frontend project and reference document contracts into `projectTypes.ts`, keeping compatibility re-exports from `types.ts` while routing API, splash, and reference panel consumers to the focused module.
- `refactor(ui): move child planning dto ownership` moved frontend child planning contracts into `childPlanningTypes.ts`, keeping compatibility re-exports from `types.ts` while routing API and beat-plan editor consumers to the focused module.
- `refactor(ai): hydrate story arcs from sqlite` routes AI generation, child planning, batch generation, and context preview prompt construction through a cloned project whose arcs are loaded from the SQLite story arc store, so stale `Project.arcs` mirrors no longer affect AI story-arc context.
- `refactor(ui): move projection primitive dto ownership` moved command IDs, projection envelopes, object field values, and generic object-field command/projection contracts into `projectionTypes.ts`, keeping compatibility re-exports from `types.ts` while routing focused DTO modules, APIs, stores, and bible field displays to the projection primitive owner.
- `refactor(ui): move timeline dto ownership` moved timeline IDs, hierarchy models, timeline gaps, layout constants, and timeline helpers into `timelineTypes.ts`, made `storyArcTypes.ts` own `ArcId`, and routed UI/API consumers to focused modules so `types.ts` is only a compatibility barrel.
- `refactor(server): preserve sqlite story arcs on broad save` changed broad project save/autosave persistence to preserve existing SQLite story arc current-state rows before clearing legacy project tables, while only seeding from `Project.arcs` when no persisted arcs or story-arc revisions exist.
- `refactor(story): stop refreshing project arc mirror` removed the remaining story arc command-route writes back into `Project.arcs`; story arc commands now use SQLite current state for validation, persistence, and returned projections without mutating the in-memory project mirror.
- `feat(timeline): persist node range current state` added a focused timeline node current-state store and changed node-range commands to upsert SQLite `nodes` rows in the same transaction as command/event/revision history while preserving the existing in-memory projection bridge.
- `feat(timeline): persist node lock current state` extended node-lock commands to upsert SQLite `nodes.locked` current state transactionally with command/event/revision history while preserving the existing in-memory projection bridge.
- `feat(timeline): persist node notes current state` extended node-notes commands to upsert SQLite node `content_json` current state transactionally with command/event/revision history while preserving the existing Y.Doc notification and in-memory projection bridge.
- `feat(timeline): persist relationship current state` added a focused timeline relationship current-state store and changed relationship create/delete commands to upsert/delete SQLite `relationships` rows in the same transaction as command/event/revision history.
- `feat(timeline): persist created node current state` extended create-node commands to insert SQLite `nodes` current-state rows transactionally with command/event/revision history while preserving the existing in-memory projection bridge and Y.Doc node initialization event.
- `feat(timeline): persist node delete current state` extended delete-node commands to remove SQLite node, node-arc, and affected relationship current-state rows transactionally with command/event/revision history while preserving the existing in-memory projection bridge and Y.Doc node removal event.
- `feat(timeline): persist node split current state` extended split-node commands to replace the original node with split child rows, rewrite node-arc tags, and upsert affected relationships in SQLite current state transactionally with command/event/revision history.
- `feat(timeline): persist child replacement current state` extended apply-children commands to remove replaced child subtrees, insert planned child nodes, rewrite node-arc tags, and upsert remaining relationships in SQLite current state transactionally with command/event/revision history.
- `refactor(timeline): read render projections from sqlite` changed the timeline render projection route to load the active project SQLite file instead of trusting the in-memory project mirror, with regression coverage that stale mirror state is ignored.
- `refactor(timeline): read legacy timeline routes from sqlite` moved the remaining read-only legacy timeline route handlers onto the active project SQLite file so `/timeline`, child queries, and gap queries no longer trust the in-memory project mirror.
- `refactor(timeline): preserve sqlite state on broad save` changed broad project save/autosave persistence to preserve existing SQLite timeline node, node-arc, and relationship current-state rows once timeline current state or revision history exists, preventing stale mirrors from overwriting backend-owned timeline data.
- `refactor(timeline): project range responses from sqlite` added SQLite current-state loaders for timeline nodes, node arcs, and relationships, and changed node-range command responses to build their returned render projection from persisted rows instead of stale in-memory node state.
- `refactor(timeline): project node state responses from sqlite` changed node lock/notes commands to persist complete SQLite node current state and return render projections from persisted rows, with duplicate replay coverage proving stale mirror state is ignored.
- `refactor(timeline): project node structural responses from sqlite` changed create/delete node commands to persist complete SQLite node, arc-tag, and relationship current state and return render projections from persisted rows, with duplicate replay coverage for stale mirror creation/deletion state.
- `refactor(timeline): project relationship responses from sqlite` changed relationship create/delete command responses to rebuild relationship render state from SQLite while keeping node state authoritative only when node revisions exist, with duplicate replay coverage for stale relationship mirrors.
- `refactor(timeline): project split responses from sqlite` changed split-node command responses to rebuild render projections from SQLite current-state rows, with duplicate replay coverage proving stale unsplit mirror state is ignored.
- `refactor(timeline): project child responses from sqlite` changed child-replacement command responses to rebuild render projections from SQLite current-state rows, with duplicate replay coverage proving stale child mirrors are ignored.
- `refactor(ai): preview context from sqlite project` changed AI context preview to load the active SQLite project directly, so prompt previews use persisted timeline nodes and story arcs instead of the in-memory project mirror.
- `refactor(ai): build generation requests from sqlite` routed AI generation, child generation, batch child discovery, and per-child batch request construction through the same active SQLite project loader used by context preview.
- `refactor(ai): persist generation status to sqlite` added a focused timeline node content-status current-state update helper and routed AI generating/error/empty/success status writes through SQLite while preserving transitional mirror updates.
- `refactor(ai): persist scene recap to sqlite` added a focused timeline node scene-recap current-state update helper and routed scene recap context reads/writes through SQLite while preserving transitional mirror updates.
- `refactor(timeline): validate range commands from sqlite` changed node-range command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for that command route.
- `refactor(timeline): validate node state commands from sqlite` changed node lock/notes command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for those command routes.
- `refactor(timeline): validate relationship commands from sqlite` changed timeline relationship create/delete validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for those command routes.
- `refactor(timeline): validate split commands from sqlite` changed split-node command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for that command route.
- `refactor(timeline): validate delete commands from sqlite` changed delete-node command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for that command route.
- `refactor(timeline): validate create commands from sqlite` changed create-node command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for that command route.
- `refactor(timeline): validate child commands from sqlite` changed apply-children command validation/history input to load the active SQLite project instead of the in-memory mirror and stopped mutating the mirror for that command route.
- `refactor(server): remove unused async command surfaces` removed unused diffusion/Y.Doc shutdown, read-all, token-flush, write helper, and unconsumed content-change surfaces so `cargo check -p eidetic-server` is warning-free.
- `feat(server): store bible reference proposals` added shared semantic proposal contracts, SQLite-backed pending bible reference proposal rows, command/event/revision history, create/projection routes, and focused tests so AI-discovered child references can be reviewed without mutating the bible graph.
- `feat(timeline): propose bible references from child plans` preserved child-plan character, location, and prop references through apply-children commands and writes pending bible reference proposals in the same SQLite transaction as the timeline child replacement.
- `feat(server): reject bible reference proposals` added a focused proposal rejection command that records review history, updates pending proposal status transactionally, returns the proposal list projection, and avoids status-only acceptance until acceptance can apply bible graph mutations atomically.
- `feat(server): accept bible reference proposals` added an atomic proposal acceptance command that updates pending proposal status and creates the accepted bible graph node in one SQLite history transaction, returning the proposal list projection and broadcasting bible/proposal changes.
- `feat(ui): cache bible reference proposals` added typed frontend proposal DTOs, semantic proposal command/projection helpers, a backend-owned proposal projection cache, focused tests, and websocket refresh handling without optimistic local proposal mutation.
- `feat(ai): preview graph-backed bible context` added a backend-owned AI bible context projection over persisted bible graph nodes, fields, snapshots, and edges, then wired the context preview prompt to consume that projection without reading legacy bible state.
- `feat(ai): generate with graph-backed bible context` reused the AI bible context projection for single-node and batch generation requests so generated script prompts use the same backend-owned graph facts as context previews.
- `feat(ai): decompose with graph-backed bible context` reused the AI bible context projection for child-decomposition requests so AI child plans can propose characters, locations, and props from persisted graph facts.
- `feat(server): link accepted bible proposals` allowed proposal acceptance to target an existing schema-compatible bible graph node without mutating it, while preserving new-node creation for unknown target node IDs.
- `feat(server): project change review history` added a backend-owned change review projection over recorded change events, object revisions, and before/after field deltas for proposal and propagation review surfaces.
- `feat(ui): cache change review history` added typed frontend change review DTOs, a projection helper, and a discardable projection cache for backend-owned change review history without local mutation.
- `refactor(server): split history read store` moved revision summaries, object revision reads, and change review history queries out of the write-side history store so future undo/redo and dependency traversal can grow without overloading command/event recording.
- `feat(server): store semantic dependencies` added typed semantic dependency contracts, relational SQLite dependency rows, command/event/revision history, command and projection routes, and source/target field-level lookup tests for future propagation impact analysis.
- `feat(server): stage propagation proposals` added typed propagation proposal contracts and SQLite-backed pending proposal rows for bible field, bible snapshot field, script block patch, and script segment regeneration review without mutating graph or script state.
- `feat(server): reject propagation proposals` added a focused rejection command that records proposal review history, transitions pending propagation proposals to rejected, and returns the updated backend-owned proposal projection.
- `feat(server): accept bible field propagation proposals` added a focused acceptance command for pending bible-field propagation proposals, applying the staged field value and proposal status transition in one event-history transaction while leaving script and regeneration acceptance for later slices.
- `feat(server): accept script block propagation proposals` added focused acceptance for pending script-block patch proposals, reusing script block validation and locked-span checks while applying the staged text and proposal status transition in one event-history transaction.
- `refactor(server): split propagation acceptance` moved propagation proposal acceptance into a focused module so rejection/status review logic and target-acceptance logic can evolve without pushing the review module over the decomposition threshold.
- `feat(ui): cache propagation proposals` added focused TypeScript propagation proposal DTOs, command/projection API helpers, a backend-owned proposal projection cache, websocket refresh handling, and tests without merging propagation proposals into the bible-reference proposal store.
- `feat(server): accept bible snapshot propagation proposals` tightened snapshot-field propagation targets to include part and field identity, then added acceptance for existing projected snapshot fields through the same validation, storage, and event-history path as direct snapshot field commands.
- `feat(server): update propagation proposals` added a pending-only full replacement update command for staged propagation proposals, including command replay protection, update event/revision history, route projection refresh, and accept-after-update coverage.
- `refactor(server): split script propagation acceptance` moved script-block propagation acceptance target preparation into a focused module, reducing the main acceptance coordinator below decomposition limits before adding structured segment-regeneration acceptance.
- `feat(ui): update propagation proposals` added frontend DTO, command API helper, and projection-store action support for backend-owned propagation proposal updates.
- `feat(server): replace script segment blocks` added a transaction-scoped script segment replacement storage primitive that soft-deletes omitted blocks, spans, and locks using existing `deleted_event_id` columns while projections continue to read only active rows.
- `feat(server): accept script segment regeneration proposals` added structured `ScriptPatch` payload support to propagation proposals and accepts pending segment-regeneration proposals by upserting regenerated segment blocks and soft-deleting omitted blocks in one proposal acceptance transaction.
- `feat(server): preserve regenerated script patch spans` extended segment-regeneration acceptance to consume patch-provided spans/provenance, reject patch-created locks, preserve existing locked spans, and soft-delete omitted unlocked spans on retained blocks.
- `feat(ai): use external llama.cpp runtime` removes the managed Python/PyTorch diffusion runtime, deletes `/ai/diffusion/*`, drops PyO3 and launcher Python setup, adds a local OpenAI-compatible llama.cpp backend, and narrows the AI config panel to external provider configuration.
- `feat(ai): persist generated child plans` added durable child-plan IDs, relational SQLite child-plan/current-state rows for generated child previews and their references, command/event/revision history for AI-created plans, and returns the persisted plan ID from `/ai/generate-children`.
- `feat(timeline): apply durable child plans` threaded durable child-plan IDs through apply-children commands, validates the stored plan is pending and belongs to the captured parent, and marks the plan applied in the same event-history transaction as the timeline child replacement.
- `feat(server): project durable child plans` added a backend-owned child-plan list projection route over persisted generated plans, child rows, references, status, created timestamps, and revision-derived projection versions.
- `feat(server): reject durable child plans` added a focused child-plan rejection command route that records review history, transitions pending generated plans to rejected, returns the backend-owned child-plan projection, and keeps the new command surface split out of the already dense semantic command router.
- `refactor(server): split ai route modules` split the AI route surface into focused generation entrypoint, generation runtime, child planning, context preview, config/status, and support modules while preserving route behavior and AI route coverage.
- `refactor(ui): split beat editor components` kept `BeatEditor.svelte` as the node-editor coordinator, extracted header, child context, planning action, notes, and AI prompt preview surfaces into focused components, moved shared editor styling out of the component body, and removed the unused legacy `BeatPlanEditor.svelte`.
- `style: normalize workspace formatting` repaired the local Lefthook path, made the Rust hook use Cargo's edition-aware formatter, and applied the dedicated Rust/Prettier formatting cleanup so workspace formatting checks can be treated as normal validation gates again.
- `refactor(server): add project database owner` added a backend-owned project database lifecycle handle shared with transitional project-path state, and routed command/projection/AI active database lookups through that owner instead of direct route access to `AppState.project_path`.
- `docs(renderer): document Bevy dependency boundary` recorded the Bevy leaf-crate dependency review, including disabled default features, current `std`-only usage, wasm-only browser interop dependencies, and the rule that new Bevy feature families require a fresh dependency review.
- `feat(bible): add render graph projection` added a backend-owned bible render graph DTO, deterministic layout helper, neighborhood indexes, SQLite-backed projection route, and frontend projection helper for the future Bevy bible graph host without adding renderer dependencies to core/server.
- `feat(ui): add change review panel` added a sidebar review surface over the backend-owned change review projection, showing accepted events, object revisions, and field deltas while refreshing from websocket events without local canonical history state.
- `feat(renderer): add Bevy bible graph bridge` added an isolated Bevy ECS leaf crate that receives backend bible render graph projections, rebuilds disposable node/edge entities, exposes wasm interop, and emits validated selection/inspect commands without adding Bevy dependencies to core or server.
- `feat(ui): cache bible render graph projections` added a discardable frontend projection cache for backend-owned bible render graph projections and refreshes it from bible websocket events for future Bevy graph host consumption.
- `feat(renderer): expose bible graph neighborhoods to wasm` exposed backend-projected bible graph neighborhood indexes through the isolated Bevy wasm bridge so graph highlighting can consume projection data without renderer-owned graph state.
- `feat(ui): add bible render graph outline` added a keyboard-accessible Svelte graph outline derived from backend bible render graph projections so future Bevy graph selection has a semantic command alternative without local graph ownership.
- `feat(renderer): project timeline relationships into Bevy` extended the isolated Bevy timeline scene rebuild to preserve backend-projected arc tags and relationship entities for future curve and overlay rendering without giving the renderer canonical state.
- `feat(renderer): derive timeline relationship curves` added projection-derived Bevy timeline relationship curve control data with endpoint validation, keeping curve geometry disposable and rebuilt from backend timeline render projections.
- `feat(renderer): add Bevy timeline playhead state` added transient renderer-owned playhead state bounded by backend timeline render projection duration and exposed it through the wasm bridge without persisting playhead position.
- `feat(renderer): expose timeline relationship curves to wasm` made projection-derived timeline relationship curve DTOs serializable and exposed them through the wasm bridge so browser hosts can consume Bevy curve data without owning canonical timeline state.
- `feat(renderer): align timeline split commands with backend` updated Bevy timeline split command emission to include backend-required replacement node IDs and validate those IDs against the loaded projection before commands leave the renderer boundary.
- `docs(plan): make frontend projection completion next` reorders the next implementation focus so Svelte ownership cleanup, projection cache enforcement, and legacy read/write deletion happen before Bevy timeline replacement resumes.
- `docs(plan): inventory tauri migration contracts` recorded the Milestone 7
  HTTP route, WebSocket/Y.Doc event, frontend helper, route-test, lifecycle, and
  renderer-WASM surfaces that must move from Axum/browser transport to
  backend-owned services and Tauri command/event adapters.
- `refactor(server): expose backend runtime library` moved the server crate's
  module root into `lib.rs`, isolated current Axum router/listener composition in
  `axum_runtime.rs`, and reduced `main.rs` to tracing plus legacy server startup
  so future Tauri bindings can depend on backend runtime modules instead of the
  binary entrypoint.
- `refactor(server): decouple validation from axum errors` introduced a
  backend-neutral `BackendError`, maps it into `ApiError` only at the legacy Axum
  boundary, and moved local CORS origin validation into `axum_runtime.rs` so
  reusable backend validators compile without HTTP response ownership.
- `refactor(server): extract project service from axum routes` moved project
  create, get, update, save, load, list, and Y.Doc load seeding behavior into
  `project_service.rs`, leaving the legacy Axum project routes as transport
  adapters while adding service-level coverage for the non-HTTP path.
- `feat(desktop): add initial tauri shell` added `src-tauri` as an isolated
  workspace member with Tauri 2.11 desktop configuration, minimal capabilities,
  backend runtime composition, a desktop health command, and a read-only
  `project_get` command backed by `project_service` without adding Tauri
  dependencies to core, server, or renderer crates.
- `feat(ui): prefer tauri project read transport` added a focused
  `desktopTransport.ts` IPC helper, enabled Tauri's global API for the desktop
  shell, and moved `getProject()` onto the `project_get` desktop command when
  available while retaining the legacy HTTP fallback during migration.
- `feat(desktop): expose project service commands` added Tauri commands for
  project create, update, save, load, and list operations over `project_service`
  and routed the matching frontend project helpers through desktop IPC when the
  Tauri transport is present.
- `feat(desktop): route object field command through tauri` extracted the
  object-field command path into `command_service.rs`, kept the legacy Axum
  route as a service adapter, exposed `command_object_field` in Tauri, and made
  the frontend command helper prefer desktop IPC when available.
- `feat(desktop): route script commands through tauri` moved script block and
  script lock command handling into `command_service.rs`, exposed
  `command_script_block` and `command_script_lock` through Tauri, and made the
  frontend script command helpers prefer desktop IPC when available.
- `feat(desktop): route story arc commands through tauri` moved story arc
  create, update, and delete command handling into `command_service.rs`, exposed
  matching Tauri commands, and made the frontend story arc command helpers
  prefer desktop IPC when available.
- `feat(desktop): route object field projection through tauri` introduced
  `projection_service.rs`, kept the legacy object-field projection route as a
  service adapter, exposed `projection_object_field` through Tauri, and made the
  frontend projection helper prefer desktop IPC when available.
- `feat(desktop): route story projection reads through tauri` moved script
  document and story arc list projection reads into `projection_service.rs`,
  kept the legacy Axum projection routes as service adapters, exposed
  `projection_script_document` and `projection_story_arcs` through Tauri, and
  made the matching frontend projection helpers prefer desktop IPC when
  available.
- `feat(desktop): route timeline projection reads through tauri` moved timeline
  render and selected-node editor projection reads into `projection_service.rs`,
  kept the legacy Axum routes as service adapters, exposed
  `projection_timeline_render` and `projection_selected_node` through Tauri,
  and made the matching frontend projection helpers prefer desktop IPC when
  available.
- `feat(desktop): route bible graph projections through tauri` moved bible graph
  node detail, node list, schema list, and render graph projection reads into
  `projection_service.rs`, kept the legacy Axum routes as service adapters,
  exposed matching Tauri commands, and made the frontend bible graph projection
  helpers prefer desktop IPC when available.
- `feat(desktop): route review projections through tauri` moved story arc
  progression and change review projection reads into `projection_service.rs`,
  kept the legacy Axum routes as service adapters, exposed
  `projection_story_arc_progression` and `projection_change_review` through
  Tauri, and made the matching frontend projection helpers prefer desktop IPC
  when available.
- `feat(desktop): route semantic proposal projections through tauri` moved
  bible-reference proposal and propagation-proposal list projection reads into
  `projection_service.rs`, kept the legacy Axum routes as service adapters,
  exposed `projection_bible_reference_proposals` and
  `projection_propagation_proposals` through Tauri, and made the matching
  frontend projection helpers prefer desktop IPC when available.
- `feat(desktop): route semantic support projections through tauri` moved
  semantic dependency and durable child-plan projection reads into
  `projection_service.rs`, kept the legacy Axum routes as service adapters, and
  exposed `projection_semantic_dependencies` and `projection_child_plans`
  through Tauri.
- `feat(desktop): route bible graph bootstrap commands through tauri` moved
  bible graph node creation and canonical-root ensure commands into
  `command_service.rs`, kept the legacy Axum routes as service adapters,
  exposed `command_bible_graph_node` and `command_bible_graph_roots` through
  Tauri, and made the matching frontend command helpers prefer desktop IPC when
  available.
- `feat(desktop): route bible graph edit commands through tauri` moved bible
  graph field, edge, and snapshot-field command handling into
  `command_service.rs`, kept the legacy Axum routes as service adapters,
  exposed `command_bible_graph_field`, `command_bible_graph_edge`, and
  `command_bible_graph_snapshot_field` through Tauri, and made the matching
  frontend command helpers prefer desktop IPC when available.
- `refactor(desktop): split bible graph command service` extracted bible graph
  desktop command handling into a focused backend service module and moved shared
  command-service helpers into a support module, reducing `command_service.rs`
  below the decomposition threshold before more Milestone 7 command migration.
- `feat(desktop): route bible reference proposal commands through tauri` moved
  bible-reference proposal create, reject, and accept command handling into a
  focused backend command service, kept the legacy Axum routes as service
  adapters, exposed matching Tauri commands, and made the frontend command
  helpers prefer desktop IPC when available.
- `feat(desktop): route propagation proposal commands through tauri` moved
  propagation proposal create, reject, update, and accept command handling into
  the focused semantic command service, kept the legacy Axum routes as service
  adapters, exposed matching Tauri commands, and made the frontend command
  helpers prefer desktop IPC when available.
- `feat(desktop): route timeline node lock command through tauri` moved timeline
  node lock command handling into a focused backend timeline command service,
  kept the legacy Axum route as a service adapter, exposed
  `command_timeline_node_lock` through Tauri, and made the frontend helper
  prefer desktop IPC when available.
- `feat(desktop): route timeline node notes command through tauri` moved
  timeline node notes command handling and its Y.Doc note-write side effect into
  the focused backend timeline command service, kept the legacy Axum route as a
  service adapter, exposed `command_timeline_node_notes` through Tauri, and made
  the frontend helper prefer desktop IPC when available.
- `feat(desktop): route timeline delete node command through tauri` moved
  timeline node deletion and its Y.Doc cleanup/hierarchy invalidation side
  effects into the focused backend timeline command service, kept the legacy
  Axum route as a service adapter, exposed `command_timeline_delete_node`
  through Tauri, and made the frontend helper prefer desktop IPC when available.
- `feat(desktop): route timeline split node command through tauri` moved
  timeline split-node command handling into the focused backend timeline command
  service, kept the legacy Axum route as a service adapter, exposed
  `command_timeline_split_node` through Tauri, and made the frontend helper
  prefer desktop IPC when available.
- `feat(desktop): route timeline create node command through tauri` moved
  timeline node creation and its Y.Doc initialization/hierarchy invalidation
  side effects into the focused backend timeline command service, kept the
  legacy Axum route as a service adapter, exposed
  `command_timeline_create_node` through Tauri, and made the frontend helper
  prefer desktop IPC when available.
- `refactor(desktop): split tauri command adapters` decomposed the Tauri shell
  into focused error, health, project, command-domain, projection-domain, and
  setup modules so desktop transport registration remains a thin adapter layer
  before more Milestone 7 command migration.
- `feat(desktop): route timeline range command through tauri` moved timeline
  node range updates into the focused backend timeline command service, kept the
  legacy Axum route as a service adapter, exposed
  `command_timeline_node_range` through Tauri, and made the frontend helper
  prefer desktop IPC when available.
- `feat(desktop): route timeline relationship delete command through tauri`
  moved timeline relationship deletion into the focused backend timeline command
  service, kept the legacy Axum route as a service adapter, exposed
  `command_timeline_delete_relationship` through Tauri, and made the frontend
  helper prefer desktop IPC when available.
- `feat(desktop): route timeline relationship create command through tauri`
  moved timeline relationship creation and backend-owned relationship ID
  derivation into the focused backend timeline command service, kept the legacy
  Axum route as a service adapter, exposed
  `command_timeline_create_relationship` through Tauri, and made the frontend
  helper prefer desktop IPC when available.
- `feat(desktop): route timeline apply children command through tauri` moved
  apply-children validation, backend-owned child ID derivation, timeline
  persistence, Y.Doc child/notes side effects, and semantic invalidation into
  the focused backend timeline command service, kept the legacy Axum route as a
  service adapter, exposed `command_timeline_apply_children` through Tauri, and
  made the frontend helper prefer desktop IPC when available.
- `refactor(server): split timeline command request DTOs` extracted
  backend-owned timeline command request DTOs, validation entrypoints, and
  derived result-ID conversion into a focused server module so the timeline
  command service and legacy route adapter remain under decomposition
  thresholds after the apply-children migration.
- `refactor(ui): split timeline command api helpers` extracted timeline command
  helper functions into `timelineCommandApi.ts` and shared HTTP command
  transport into `commandTransport.ts` while preserving the existing
  `commandApi.ts` re-export surface, keeping touched frontend modules below
  decomposition thresholds.
- `feat(desktop): emit backend server events through tauri` added a desktop
  event bridge that subscribes to backend-owned `ServerEvent`s and emits
  stable `eidetic://server-event` Tauri payloads for the future WebSocket
  replacement path, with serialization tests covering event type and field
  preservation.
- `feat(ui): prefer tauri server events in desktop shell` added a transport
  neutral frontend server-event client that listens to `eidetic://server-event`
  through Tauri when desktop event transport is available and falls back to the
  legacy WebSocket client otherwise, preserving the existing projection refresh
  orchestration handlers while covering event dispatch and teardown.
- `feat(desktop): route AI status config through tauri` extracted AI
  status/config behavior into a host-neutral backend service, exposed Tauri
  commands for desktop status/config reads and writes, and made the frontend AI
  helpers prefer desktop IPC while retaining the legacy HTTP fallback.
- `feat(desktop): route model list through tauri` extracted Pumas model-library
  list/search behavior into a host-neutral backend service, exposed a Tauri
  projection command for desktop model picker reads, and made the frontend model
  helper prefer desktop IPC while preserving legacy route behavior.
- `feat(desktop): route AI context preview through tauri` moved AI prompt
  preview construction into the backend AI service, exposed a Tauri read command
  for desktop context preview, and made the frontend context helper prefer
  desktop IPC while retaining the legacy route fallback.
- `feat(desktop): route PDF export through tauri` moved screenplay PDF export
  into a host-neutral backend service, exposed a Tauri command returning PDF
  bytes, and made the frontend export helper wrap desktop bytes into the same
  `Blob` contract while retaining the legacy HTTP fallback.
- `refactor(renderer): remove wasm renderer bridges` deleted the timeline and
  bible graph wasm-bindgen bridge modules, removed wasm-only renderer
  dependencies, and updated renderer docs to record native desktop Bevy host
  integration as the production path.
- `feat(server): add backend task supervisor` added a backend-owned task
  lifecycle supervisor, moved AppState-owned autosave and Y.Doc manager tasks
  behind it, and wired desktop window teardown to abort supervised backend
  tasks instead of leaving them detached.
- `refactor(server): supervise route background tasks` moved legacy AI
  generation, batch generation, and reference embedding route background work
  onto the backend task supervisor so the remaining route fallback paths share
  the same desktop shutdown behavior as AppState-owned tasks.
- `feat(desktop): route references through tauri` extracted reference
  list/upload/delete behavior into a host-neutral backend service, exposed
  Tauri reference commands, and made frontend reference helpers prefer desktop
  IPC while preserving legacy route fallback and supervised embedding.
- `feat(desktop): route child generation through tauri` moved AI child-plan
  generation into the backend AI service, exposed a Tauri command for desktop
  decomposition requests, and made the frontend child-generation helper prefer
  desktop IPC while keeping the legacy route as a thin fallback adapter.
- `feat(desktop): route script generation through tauri` moved streaming script
  generation and batch generation orchestration into a host-neutral backend
  service, exposed Tauri commands for desktop generation requests, and made the
  frontend generation helpers prefer desktop IPC while keeping legacy routes as
  fallback adapters.
- `refactor(launcher): run tauri desktop by default` changed the canonical
  launcher target from the legacy Axum server to the Tauri desktop binary,
  starts Vite only as the dev webview asset server, removes browser auto-open
  behavior, and updates README workflow language for the desktop app.
- `refactor(ui): remove websocket event fallback` made Tauri event transport
  mandatory for backend events, deleted the legacy browser WebSocket/Yjs sync
  client and tests, and removed unused frontend Yjs dependencies after notes
  editing moved through backend timeline commands/projections.
- `refactor(ui): remove main api http fallback` removed legacy fetch fallback
  paths from the main frontend API helper for project, reference, AI, model,
  export, and persistence operations so those helpers now require Tauri IPC.
- `refactor(ui): remove projection http fallback` removed legacy fetch fallback
  paths from focused frontend projection helpers so all active projection reads
  now require Tauri IPC and fail fast when the desktop transport is missing.
- `refactor(ui): remove timeline command http fallback` removed legacy fetch
  fallback paths from timeline-specific command helpers so timeline mutations
  now enter the backend only through Tauri IPC from the desktop frontend.
- `refactor(ui): remove command http fallback` removed legacy fetch fallback
  paths from the remaining frontend command helpers and reduced the shared
  command transport helper to command ID generation.
- `refactor(server): remove axum listener runtime` deleted the standalone
  server binary, Axum listener/router composition, static file host, WebSocket
  host, and the now-unused `tower-http` dependency while retaining route
  adapters for the remaining service-test migration.
- `refactor(server): remove axum route adapters` deleted the remaining legacy
  Axum route adapter tree, Axum-only error adapter, and now-unused `axum` and
  `tower` dependencies after Tauri commands/projections became the active
  frontend/backend boundary.
- `refactor(ui): remove legacy dev server proxy` deleted the Vite `/api` and
  `/ws` proxy targets plus the unused desktop-transport availability helper so
  the frontend development server no longer advertises a loopback HTTP or
  WebSocket backend path.
- `refactor(core): remove wasm target dependencies` removed the core crate's
  `wasm32` dependency block and updated the domain-layer README so core remains
  host-agnostic without carrying browser/WASM-specific support.
- `docs(server): retire websocket-owned wording` updated backend Y.Doc and
  state comments so realtime events are described as desktop/document
  subscribers instead of the removed WebSocket transport.
- `refactor(ui): rename server event handlers` renamed frontend `wsHandlers`
  and `wsTypes` modules to server-event terminology after backend event
  delivery became Tauri-only.
- `feat(desktop): add release smoke startup probe` added an `eidetic-desktop
  --smoke` path that initializes the backend runtime, emits JSON health,
  shuts down supervised backend tasks, and exits without opening a window, plus
  a launcher `--release-smoke` command over the release binary.
- `fix(ui): defer server event client creation` moved root-page server event
  client construction into `onMount` and added an SSR regression test so
  SvelteKit's server render no longer requires Tauri globals.
- `feat(desktop): add floating graph renderer controls` added explicit
  backend-owned graph renderer open, focus, and close commands, renderer-window
  status fields, and Svelte launch/focus/status controls so graph workspace no
  longer starts the production Bevy path through the embedded viewport panel.
- `refactor(desktop): remove embedded viewport path` deleted the unused Svelte
  embedded viewport component/helpers and removed the registered Tauri
  child-surface viewport host/commands so production graph refresh now keys off
  the backend graph renderer lifecycle instead of WebView panel attachment.
- `refactor(renderer): rename graph host window status` replaced remaining
  native-panel terminology in the Bevy bible graph host, desktop renderer owner,
  and graph renderer IPC status with renderer-window naming so the next native
  work does not inherit embedded viewport language.
- `fix(desktop): distinguish graph scene readiness` separated Bevy graph scene
  readiness from visible renderer-window readiness in the desktop status
  projection so Svelte no longer reports a visible native window before the OS
  window proof exists.
- `refactor(desktop): bound graph renderer queue` replaced the graph renderer
  owner's unbounded command channel with a fixed-capacity queue and explicit
  queue-full error so renderer work cannot accumulate without backpressure.
- `fix(desktop): keep graph focus read-only` changed the graph renderer focus
  command to report current renderer status instead of starting the renderer,
  because focusing is not meaningful until the visible native window exists.
- `feat(desktop): add graph renderer window strategy status` added typed graph
  renderer window strategy and capability fields so the backend projection
  explicitly reports Bevy/winit floating-window intent and pending native runner
  support before the visible OS window proof lands.
- `feat(desktop): add graph renderer window lifecycle status` added a typed
  renderer-window lifecycle projection so graph window state is reported as a
  backend-owned state machine instead of being inferred from booleans and
  status text.
- `feat(desktop): identify graph renderer window kind` added a shared desktop
  renderer-window kind contract and marked the bible graph renderer status as
  the graph window, preparing the lifecycle surface for timeline reuse without
  creating a second frontend owner.
- `refactor(ui): derive graph renderer status labels from lifecycle` moved the
  graph renderer control label mapping into a tested pure projection helper so
  Svelte reads backend lifecycle status instead of inferring renderer state from
  loose booleans.
- `fix(desktop): validate graph renderer window bounds` rejected zero-sized
  renderer window bounds before starting the graph renderer, making size hints
  a validated desktop-host boundary instead of raw dimensions.
- `fix(desktop): project graph renderer focus support` added an explicit
  backend-owned focus capability flag and made the Svelte focus action depend
  on that projection instead of assuming every open renderer can be focused.
- `feat(desktop): accept graph renderer size hints` added an optional
  validated renderer-window size hint to the graph renderer open command so
  placement/size input has an explicit desktop request contract before native
  OS window support lands.
- `refactor(renderer): bound graph command buffer` added a fixed capacity to
  the Bevy bible graph leaf renderer command buffer so renderer-emitted command
  intents cannot grow without backpressure.
- `refactor(desktop): share graph renderer queue capacity` made the desktop
  graph renderer owner queue derive its capacity from the Bevy graph renderer
  command buffer constant so the two backpressure limits cannot drift.
- `refactor(desktop): normalize graph renderer capability status` expanded the
  renderer-window capability contract across Rust and TypeScript so
  platform-unproven, platform-unsupported, runner-error, and verified-support
  are first-class backend states instead of contradictory combinations of
  `pending_native_runner`, reason strings, and support booleans.
- `fix(desktop): project graph renderer startup failures` replaced graph
  renderer startup `expect` paths with typed runner-error status projections so
  native runner or renderer-owner startup failure does not crash Tauri setup and
  remains visible through the backend-owned renderer status contract.
- `refactor(desktop): consolidate graph projection refresh state` moved active
  graph renderer projection request identity and refresh coalescing state behind
  one desktop-owned mutex so related request/refresh transitions cannot be
  updated through independent mutable paths.
- `feat(desktop): report native renderer smoke preflight` added
  `eidetic-native-renderer-smoke --report-only` so the diagnostic Bevy/winit
  preflight path records platform, backend, threading model, window config,
  command, and report-only result without opening a renderer window or changing
  production capability status.
- `refactor(desktop): isolate renderer platform strategy` moved native graph
  renderer platform detection into a dedicated desktop platform-strategy module
  with local README invariants, keeping `cfg()` checks out of shared renderer
  host, supervisor, command, UI, and backend projection code.
- `feat(renderer): add native window close control` added a renderer-local
  Bevy native window control handle so the desktop supervisor can request close
  through a leaf-crate lifecycle signal without giving the renderer access to
  Tauri, SQLite, or durable project state.
- `test(desktop): record native renderer diagnostic smoke proof` reran the
  diagnostic Bevy/winit graph renderer smoke path on local Linux with
  `cargo run -p eidetic-desktop --bin eidetic-native-renderer-smoke -- --any-thread --auto-close-ms 500`;
  the command exited successfully after the auto-close window run. This proves
  only the standalone diagnostic path and does not mark production Tauri runner
  support as verified.
- `feat(desktop): add native renderer window thread owner` added a desktop
  native renderer window thread handle with injectable runner startup, close
  request signaling, completion/panic result reporting, and bounded stop
  waiting so the supervisor can own Bevy event-loop lifecycle without detached
  tasks.
- `feat(desktop): run native renderer through supervisor` wired the first
  production `NativeRendererSupervisor` path to start the Bevy window thread,
  project running/closed/failed status, refresh completion or panic state from
  status/focus requests, and close with bounded joined teardown while keeping
  unit tests on injected non-GUI runner threads.
- `feat(renderer): report native window readiness` added a renderer-produced
  ready signal to the Bevy native window control path and projects that signal
  through the desktop window thread/supervisor status instead of treating thread
  liveness as window readiness.
- `test(desktop): add graph renderer lifecycle smoke` added
  `eidetic-desktop --graph-renderer-smoke` to exercise the desktop-managed graph
  renderer lifecycle through open/status/focus/close/reopen/project-close/
  app-shutdown and fail if any stage reports an error or mismatched open state.
- `fix(desktop): keep native renderer event loop alive` changed renderer close
  into a hide/show lifecycle transition inside the long-lived Bevy app and
  reserved `AppExit` for owner shutdown, so local Linux
  `eidetic-desktop --graph-renderer-smoke` now proves open/status/focus/close/
  reopen/project-close/app-shutdown without recreating winit.

Discovered issues:

- Resolved: real native graph renderer lifecycle smoke on local Linux exposed
  Bevy/winit `RecreationAttempt` when the supervisor stopped the Bevy app on
  close and then tried to reopen by creating another event loop in the same
  process. Renderer close/reopen now hides/shows the existing native window
  inside one long-lived Bevy app, and app shutdown is the only path that sends
  `AppExit`. Local proof command
  `cargo run -p eidetic-desktop --bin eidetic-desktop -- --graph-renderer-smoke`
  exits 0 with ready open/reopen snapshots.
- Resolved: reference upload still spawned unmanaged embedding work from the
  legacy route path. Reference embedding now enters through the backend task
  supervisor so desktop shutdown can abort it with the rest of the runtime.
- Resolved: the Vite development server still proxied `/api` and `/ws` to the
  removed loopback server after active frontend helpers became Tauri-only. The
  proxy and unused fallback detection helper were removed so missing desktop
  transport now fails at the Tauri adapter boundary.
- Resolved: `eidetic-core` still declared `wasm32`-only `uuid` and `getrandom`
  dependencies from the earlier browser-host plan. Those direct target
  dependencies were removed; any remaining WASM-related lockfile entries are
  transitive dependency metadata from third-party desktop dependencies.
- Resolved: backend realtime comments still described WebSocket clients after
  WebSocket delivery moved behind Tauri events and the listener was deleted.
  Comments now describe desktop event and document update subscribers.
- Resolved: frontend event refresh orchestration still used `wsHandlers` and
  `wsTypes` names after the transport moved to Tauri events. The modules,
  imports, tests, and route/store docs now use server-event terminology.
- Resolved: the launcher documented release-smoke requirements in Milestone 7
  but had no release-smoke command. The desktop binary now owns a headless
  `--smoke` startup probe and the launcher exposes `--release-smoke` for the
  packaged release artifact.
- Resolved: the root Svelte page created the Tauri event client during module
  initialization, so SvelteKit SSR failed with `[500] GET /` whenever
  `window.__TAURI__` was unavailable. Event client creation now happens only in
  `onMount`, and `page.ssr.test.ts` covers server rendering without Tauri event
  transport.
- Resolved: the superseded embedded viewport host and helpers were still
  registered after the graph workspace moved to floating renderer controls,
  keeping the X11/WebView child-surface path reachable from desktop IPC. The
  registered commands, host modules, Svelte component, helpers, and tests were
  deleted; graph projection refresh now checks the graph renderer owner.
- Resolved: the renderer host still exposed `native_panel_*` APIs and status
  fields after the embedded panel path was retired. The Bevy graph crate,
  desktop host, tests, frontend status DTO, and dependency notes now use
  renderer-window terminology.
- Resolved: graph renderer status conflated Bevy scene initialization with
  visible native window readiness. The desktop status projection now exposes
  `renderer_scene_ready`, `renderer_window_visible`, and
  `renderer_window_ready` separately so native window proof work has a truthful
  baseline.
- Resolved: the desktop graph renderer owner used an unbounded command channel
  even though Milestone 8 requires bounded renderer queues. The owner now uses a
  fixed-capacity queue and reports `QueueFull` instead of accepting unlimited
  pending renderer commands.
- Resolved: the graph renderer focus command started the renderer lifecycle even
  though no visible native window can be focused yet. Focus now remains a
  read-only status command until native window focus support lands.
- Resolved: native graph renderer window capability was only represented by a
  free-text status message. The desktop graph renderer status now reports the
  Bevy/winit floating-window strategy and pending native-runner capability as
  typed backend-owned projection data.
- Resolved: renderer-window capability could still report a pending capability
  while reason/support fields described platform-unproven, unsupported,
  runner-error, or verified support. Capability is now the primary typed state,
  support booleans are derived by the backend from that capability, and Svelte
  command/status logic keys off the capability instead of reason strings.
- Resolved: renderer startup still used `expect` for the native runner boundary
  and Tauri-managed graph renderer owner. The host now projects native runner
  startup failures as typed runner-error status, and Tauri setup installs an
  unavailable renderer owner instead of crashing when the owner thread cannot be
  started.
- Resolved: graph renderer projection request identity and refresh coalescing
  state were stored behind separate mutexes even though refresh correctness
  depends on their combined value. They now live in one lock-protected desktop
  state object.
- Resolved: the diagnostic native renderer smoke path could open a window but
  had no machine-readable preflight record for standards review. The smoke
  binary now has a report-only JSON mode that records the platform/backend/
  threading/config/command used before a real visual smoke run.
- Resolved: renderer platform detection still lived in shared renderer-window
  status code. Platform detection is now isolated in the desktop
  `bevy_graph_host::platform_strategy` module tree, with renderer-window status
  consuming a platform-neutral function.
- Resolved: the Bevy native smoke app had no external close control for a
  production supervisor to use. The leaf renderer crate now exposes a
  renderer-local close handle and controlled native-window app configuration.
- Resolved: the current diagnostic smoke proof had not been recorded after the
  report-only preflight mode landed. Local Linux worker-thread diagnostic smoke
  now has both a report-only JSON record and a successful auto-close window run
  recorded in the plan.
- Resolved: the desktop supervisor had no bounded lifecycle primitive for a
  long-lived Bevy window thread. A desktop-owned window thread handle now owns
  close signaling, completion observation, panic capture, and bounded stop
  waits.
- Resolved: wiring the supervisor to the native window thread made graph host
  unit tests create real Bevy/winit event loops, which is non-deterministic and
  caused event-loop recreation failures. Host and owner tests now inject a
  backend-owned non-GUI window-thread starter so tests validate the same
  lifecycle contract without opening platform windows.
- Resolved: native renderer `window_ready` still came from supervisor-thread
  liveness after the production runner path landed. The Bevy native renderer now
  marks readiness through its control resource, and the desktop supervisor
  projects readiness only after observing that renderer-produced signal.
- Resolved: graph renderer window lifecycle could only be inferred from open,
  scene-ready, visible, and message fields. The status projection now includes
  an explicit lifecycle enum that future native window support can advance
  without adding frontend-owned state inference.
- Resolved: graph renderer status did not identify which renderer kind owned
  the lifecycle record, even though Milestone 8 requires the window host to
  support graph now and timeline later. The desktop status now carries a shared
  renderer-window kind so future timeline work can reuse the same lifecycle
  vocabulary.
- Resolved: graph renderer controls still derived user-visible renderer state
  directly from `renderer_window_open` and `renderer_window_visible` after the
  backend added a lifecycle enum. The controls now use a tested lifecycle
  display adapter, keeping lifecycle interpretation projection-driven and
  avoiding duplicate frontend state inference.
- Resolved: graph renderer window bounds were accepted as raw dimensions and
  could start the renderer before invalid zero-sized hints were rejected. The
  desktop host now validates bounds before lifecycle state changes.
- Resolved: graph renderer controls enabled focus whenever the renderer was
  open, even though focus is unsupported until a visible native window exists.
  Focus support now crosses the desktop boundary as typed status and the UI
  treats it as projection data.
- Resolved: graph renderer open requests had no typed path for optional
  renderer-window size hints, even though Milestone 8 requires size/placement
  hints to be validated by the floating renderer host. Open requests now accept
  a validated size hint and reject zero dimensions before renderer startup.
- Resolved: the desktop graph renderer owner queue was bounded, but the Bevy
  bible graph leaf renderer still stored emitted command intents in an
  unbounded `Vec`. The renderer command buffer now has the same fixed capacity
  and returns a typed queue-full error.
- Resolved: the desktop owner queue and Bevy leaf renderer command buffer used
  separate fixed-capacity literals. The desktop owner now derives its queue
  capacity from the renderer crate's exported command-buffer capacity.
- Resolved: `src-tauri/src/lib.rs` exceeded the 500-line decomposition
  threshold while registering mixed project, command, projection, setup, and
  error-adapter responsibilities. The Tauri shell was split into focused
  modules, and command/projection adapters were further separated by domain so
  touched desktop modules stay below decomposition thresholds.
- Resolved: commit hooks reported `Can't find lefthook in PATH` because the root `node_modules/lefthook` symlink pointed at a stale path. The local symlink was repaired and the hook config now uses edition-aware Cargo formatting instead of direct `rustfmt` invocation.
- Resolved: baseline `cargo fmt --all -- --check` reported pre-existing formatting drift in core/server Rust files. A dedicated formatting slice normalized the workspace instead of mixing formatting churn into feature commits.
- Resolved: `rustfmt` on `crates/server/src/main.rs` previously recursed through out-of-line modules and reformatted unrelated baseline-drift server files. After the dedicated formatting cleanup, repo-wide Rust formatting is expected and can be used as a validation gate.
- Resolved: adding Bevy 0.16.1 to an isolated leaf crate pulled 94 packages into `Cargo.lock`. The renderer crate now documents the dependency review: Bevy remains isolated from core/server, default features stay disabled, current usage is `std`-only, wasm interop dependencies are target-scoped, and future render/window/asset/text/input features require a new dependency review.
- Resolved: `ui/src/lib/components/editor/BeatEditor.svelte` exceeded the component decomposition threshold after child-planning work. The editor was split into focused header, child-context, planning-action, notes, and prompt-preview components; `BeatEditor.svelte` now stays under the threshold and the unused oversized legacy `BeatPlanEditor.svelte` was removed.
- Resolved: `crates/server/src/routes/ai.rs` exceeded the 500-line decomposition threshold after durable child-plan work. AI generation entrypoints, generation runtime/persistence helpers, child planning, context preview, config/status, and support helpers were split into focused modules, with each touched AI route module under the decomposition threshold.
- Resolved: the managed diffusion runtime pulled Python/PyTorch into normal desktop startup even though Eidetic should use external inference. The server no longer compiles or spawns diffusion/PyO3 code, the launcher no longer provisions Python packages, the frontend no longer calls diffusion routes, and text generation now has a local llama.cpp adapter for an externally managed server.
- Moved to Milestone 11: Milestone 6 exposed that valence/arousal overlays have
  no canonical backend source of truth for valence, arousal, mood, or affect
  scores. Affect data now has its own milestone after the bible graph/context
  influence surface and before timeline renderer overlay work so Bevy never
  owns renderer-local affect state.
- Resolved: `crates/server/src/history_store.rs` exceeded the decomposition threshold after adding read-side change review loading. Revision summaries, object revision reads, and change review history queries were split into `history_read_store.rs`.
- Resolved: timeline structural commands record sparse SQLite command/event/revision rows with replay protection across node range, node create/delete/split, child replacement, node lock, node notes, and relationship create/delete commands, and all of those command handlers persist SQLite current-state rows transactionally. Timeline render projections, read-only legacy timeline routes, broad save/autosave preservation, and all timeline command response projections now use SQLite current-state rows instead of trusting the in-memory project mirror.
- Resolved: timeline command routes no longer mutate the in-memory project mirror after SQLite writes. Command-specific Y.Doc and websocket side effects remain explicit route side effects, while durable timeline state is owned by SQLite current-state rows.
- Resolved: timeline command routes no longer validate against or mutate the in-memory timeline mirror. Node range, node lock/notes, create/delete/split, relationship create/delete, and apply-children commands all load the active SQLite project for validation/history input and return SQLite-backed projections while preserving command-specific Y.Doc and websocket side effects.
- Resolved: AI context preview, generation request construction, child generation request construction, batch child discovery, generation status updates, generated-content status updates, and scene recap reads/writes now use persisted SQLite timeline/story-arc state before transitional mirror updates.
- Resolved: story arc commands validate against SQLite, record command/event/revision rows and SQLite current-state rows transactionally, return SQLite-backed list projections, and no longer refresh `Project.arcs`; story arc list/progression projections read arcs from SQLite; AI prompt construction hydrates arcs from SQLite before calling core prompt builders; and broad project save/autosave preserves SQLite arc current state instead of overwriting it from stale `Project.arcs`.
- Resolved: child planning proposals still contain character, location, and prop references, but timeline apply-children intentionally does not mutate bible graph state. A backend-owned pending bible reference proposal command/projection path now exists so those references can be stored for review without bible graph side effects.
- Resolved: applied AI child plans now preserve edited character, location, and prop references in the apply-children command payload and create pending bible reference proposals transactionally with the timeline child replacement.
- Resolved: proposal review can now reject pending bible reference proposals with command/event/revision history and SQLite current-state status updates.
- Resolved: proposal review can now accept pending bible reference proposals by composing the proposal status update and bible graph node creation in one command/event/revision transaction.
- Resolved: the AI context preview prompt now consumes a graph-backed bible context projection from SQLite, including persisted graph nodes, fields, snapshots, and edges.
- Resolved: generate-children previews now receive durable backend-owned child-plan IDs and relational SQLite storage with event-history revisions as soon as AI returns the editable plan.
- Resolved: apply-children now consumes the durable child-plan ID, validates parent identity, captures the selected parent before async generation in `BeatEditor`, and marks the plan applied through backend-owned history instead of treating the preview identity as disposable metadata.
- Resolved: durable child plans now expose a backend-owned projection route so generated plans can be inspected across sessions before or after application.
- Resolved: durable child plans now expose an explicit backend-owned rejection command for generated plans the user chooses not to apply.
- Resolved: propagation proposal script-block acceptance now applies staged downstream script block text through the event history path and preserves existing locked-span protection.
- Resolved: propagation proposal review no longer owns target acceptance implementation details; future propagation targets should extend the focused acceptance module or split by target before adding enough behavior to exceed decomposition thresholds.
- Resolved: propagation proposal create/reject/accept routes now emit the semantic proposal refresh event, and the frontend keeps propagation proposal projections in a separate discardable cache from bible-reference proposals.
- Resolved: bible snapshot field propagation proposals now target existing snapshot fields unambiguously by node, snapshot, part, field key, and field ID; acceptance updates only existing projected fields and does not create snapshots implicitly.
- Resolved: pending propagation proposals can now be updated before accept/reject through a backend-owned command that records `AiProposalUpdated` history and replaces the proposal payload atomically without mutating bible/script targets.
- Resolved: script-block propagation acceptance target preparation no longer lives in the main acceptance coordinator, leaving room for future target handlers without exceeding module decomposition limits.
- Resolved: frontend propagation proposal caches now expose the backend update command path and refresh from the returned projection instead of mutating proposal state locally.
- Resolved: script storage now has a focused soft-delete primitive for blocks/spans/locks omitted from a segment replacement, removing the stale-row blocker before structured segment-regeneration acceptance is wired.
- Resolved: segment-level regeneration proposals are no longer review-only; the proposal contract can carry a structured `ScriptPatch`, and acceptance applies the target segment's regenerated block list through the existing command/event/revision transaction path.
- Resolved: segment regeneration acceptance now applies patch-provided spans and provenance for regenerated blocks, rejects patch-created locks, preserves retained locked spans, and removes omitted unlocked spans from retained blocks.
- Resolved: pre-existing dead-code warnings in `diffusion/types.rs` and `ydoc.rs` blocked a future `-D warnings` gate. Unused diffusion/Y.Doc command variants, the unconsumed content-change feed, the unused write helper, and production-only unused snapshot fields were removed or narrowed to tests; `cargo check -p eidetic-server` is now warning-free.
- Resolved: the cloned-project undo/redo routes still existed after cloned snapshot producers were removed. The routes, websocket event, frontend API helpers, shortcuts, toolbar controls, and transient UI flags were deleted; future undo/redo must enter through revision-backed command/event history.
- Resolved: the first implementation attempt exposed the stale Pumas path and lockfile state as a build metadata blocker. The path and lockfile were fixed in earlier build slices, and subsequent implementation slices now use Cargo verification instead of stale metadata.
- Resolved: command/projection routes previously reached into `AppState.project_path` directly because `AppState` had no backend-owned database lifecycle owner. `ProjectDatabase` now owns active project database state and exposes the active path/connection boundary for command, projection, AI, and project route surfaces while transitional autosave state is retired.
- Resolved: `crates/server/src/projection_service.rs` exceeded the 500-line
  decomposition threshold after Milestone 8 projection work. Semantic
  dependency projection request parsing, filtering, loading, and error mapping
  now live in a focused projection module while the existing service API is
  re-exported for callers.
- Resolved: bible render graph projections no longer load the full graph before
  applying request limits. `bible_render_graph_query.rs` now owns the
  SQLite-bounded read path for default, focused-root, selected-node, and search
  requests; the core projection adapter remains a pure deterministic final
  projection step.
- Resolved: Milestone 8 now has executable context-stack and context-influence
  boundary DTOs in `eidetic-core`, including typed evaluation/influence IDs,
  source layer, influence kind, confidence, reason, provenance, and bible
  node/edge references.
- Resolved: context evaluations and influence records now have relational
  SQLite storage, command/event/revision history, idempotent command replay,
  desktop command/projection routes, and TypeScript command/projection helpers.
  The next context slice can build active hierarchy stack and graph-highlight
  projections on top of these backend-owned records.
- Resolved: selected timeline nodes now have a backend-owned context-stack
  projection that walks the persisted timeline ancestor chain and returns the
  active premise/act/sequence/scene/beat layers for UI and graph highlighting
  without deriving hierarchy context in Svelte or Bevy.
- Resolved: selected timeline nodes now seed bounded bible render graph
  projections from persisted context influence records. The query service loads
  the latest influences for the selected clip, includes referenced bible nodes
  and edge endpoints as bounded graph seeds, and returns visible influence DTOs
  so Bevy and Svelte can highlight backend-owned active context paths without
  deriving graph relevance locally.
- Resolved: the leaf Bevy bible graph renderer now consumes influence
  projection payloads by rebuilding read-only influence ECS entities, exposing
  influence counts and node/edge influence lookup helpers, and emitting
  validated transient influence-selection commands without owning durable graph
  relevance.
- Resolved: the Svelte bible graph outline now requests render graph
  projections for the selected timeline node and displays active influence
  counts from backend-owned projection data. Timeline selection changes refresh
  the bounded graph projection instead of deriving context relevance in the
  frontend.
- Resolved: Bevy leaf renderer crates now use Bevy 0.18.1 with
  `default-features = false` and the `std` feature only. Dependency reviews in
  the graph and timeline renderer READMEs record the `cargo tree --depth 2`
  checks and keep Bevy isolated from `eidetic-core` and `eidetic-server`.
- Resolved: shell toolbar markup and styling moved from `AppShell.svelte` into
  `AppToolbar.svelte`, reducing shell responsibility before graph-focused and
  split workspace modes are added.
- Resolved: the shell now has transient script, graph, and split workspace
  modes. The central graph workspace consumes backend-owned bible render graph
  projection caches through `GraphWorkspacePanel.svelte` and emits only
  transient graph selection, while the bottom timeline remains mounted below
  the workspace.
- Resolved: frontend bible graph selection is now a typed transient union for
  nodes, edges, influence paths, context layers, and neighborhoods. Node detail
  panels derive a node ID only from node selections, so future graph commands do
  not overload `selectedGraphNodeId`.
- Resolved: the right inspector now handles non-node graph selections from the
  current bounded render graph projection. Influence paths, edges, context
  layers, and neighborhoods are adapted through a pure projection helper, while
  the central graph workspace exposes influence selection without storing
  durable graph state in Svelte.
- Resolved: the graph workspace now exposes keyboard-accessible projection
  lists for inspectable influences, edges, and neighborhoods. Side-list rows
  are derived by pure helpers, emit only transient typed selections, and keep
  `GraphWorkspacePanel.svelte` below the component decomposition threshold.
- Resolved: selected timeline context stacks now have a focused frontend
  projection cache and graph workspace context-layer list. Context layer
  selection feeds the same transient graph selection/inspector path without
  deriving hierarchy state locally.
- Resolved: the desktop crate now exposes a native Bevy bible graph host
  boundary for renderer start/stop, projection application, validated command
  draining, and panic-to-error conversion. The host depends on the Bevy leaf
  renderer without making `eidetic-server` depend on Bevy or Tauri.
- Resolved: Bevy `App` is not `Send`, so the graph renderer host is no longer
  stored directly in Tauri managed state. `DesktopBibleGraphRendererOwner` is a
  `Send + Sync` desktop boundary that owns a dedicated renderer thread, request/
  reply command queue, command draining, and shutdown joining from Tauri window
  teardown.
- Resolved: the native graph renderer owner now has projection refresh
  plumbing from backend mutation events. The desktop bridge refreshes the
  backend-owned bible render graph projection into the renderer thread only
  while a graph renderer window is active.
- Resolved: bible render graph projections included selected context influence
  records but derived their envelope version only from bible node and edge
  revisions. Render graph envelopes now include context evaluation and context
  influence revisions so influence-only changes are cache-visible and
  reload/history review can tie graph highlights to the change that produced
  them.
- Resolved: selected timeline context influence graph projections now have a
  file-backed SQLite reload regression test. The test records graph nodes,
  edges, and influence records, closes the connection, reopens the database,
  and proves the render graph payload, version, and change event are rebuilt
  from persisted backend state.
- Resolved: graph renderer Svelte controls marked projection requests as synced
  before backend open/set-projection commands succeeded. The local request
  marker now advances only after a successful backend status response so failed
  projection updates stay visible and retryable without creating frontend-owned
  durable graph state.
- Open: `eidetic-bevy-bible-graph` currently builds a Bevy scene resource graph
  and native visual entities, but `BibleGraphRendererApp::new_renderer_window`
  does not yet install/run a Bevy 0.18.1 window event loop. The desktop host
  therefore truthfully reports `PendingNativeRunner`; Milestone 8 still needs a
  per-platform floating-window runner strategy that consumes backend
  projections and command channels without moving durable graph state into
  Bevy or Svelte.
- Updated: the native runner blocker is an implementation proof, not an
  architecture redesign. The next runner work must land as a sequence of
  pass/fail slices: a minimal real Bevy/winit window runner behind
  `NativeRendererPlatformStrategy`, current-platform open/status/focus/close/
  reopen/teardown verification, typed unsupported status for unverified
  platforms, and only then graph projection rendering in the visible window.
- Resolved: graph renderer lifecycle derivation could report `visible` from an
  impossible state where the renderer was not running or the scene was not
  ready. The lifecycle helper now requires a running, scene-ready renderer
  before visible-window state can be projected.
- Resolved: graph renderer status dropped the desktop strategy's visible-window
  support boolean before reaching Tauri/Svelte. Status projections now expose
  `renderer_window_visible_supported` explicitly so unsupported native runner
  capability degrades through typed backend-owned status instead of frontend
  enum inference.
- Resolved: graph renderer UI status treated the pending native runner state as
  a normal waiting state even when the desktop backend reported no visible
  window support. The Svelte status projection now uses the backend support
  boolean to report the current renderer as unavailable until the native runner
  lands.
- Updated: the remaining Milestone 8 Bevy window work is now a formal native
  window runner gate. The gate must prove or reject the Bevy 0.18.1 plus Tauri
  event-loop strategy before graph visuals replace the Svelte semantic outline,
  and it records ownership, platform, dependency, verification, and re-plan
  constraints from the coding standards.
- Superseded: the native renderer runner is no longer an in-thread pending
  implementation owned directly by the graph host. It now runs behind a bounded
  `NativeRendererRunnerHandle` and `NativeRendererSupervisor` state machine.
  The real Bevy/winit event-loop path still must be implemented under that
  supervisor and prove open/focus/close/reopen/teardown before visual graph
  replacement.
- Updated: native renderer runner status now projects a typed runner lifecycle
  (`closed`, `open_requested`, `visible`) separately from scene lifecycle and
  visible-window capability, and now also projects the supervisor lifecycle.
  The supervisor can therefore record open/focus/close intent without Svelte or
  the graph host inferring runner state from booleans before the real
  Bevy/winit event-loop path lands.
- Updated: native renderer runner request/reply failures now degrade through
  typed backend status with a bounded reply timeout and `last_error` projection
  instead of silently returning pending capability state. The real Bevy/winit
  runner still needs panic/status reporting from the actual event-loop thread
  before the visible window gate can pass.
- Resolved: native renderer runner shutdown now uses an explicit bounded stop
  request and joins the runner thread through `NativeRendererRunnerHandle::stop`
  instead of blocking indefinitely in `Drop`. Stop failures degrade through
  typed runner-error status and `last_error` projection.
- Updated: desktop graph renderer owner request/reply calls now have a bounded
  reply timeout and typed timeout error, so Tauri command handlers do not wait
  indefinitely if the renderer owner thread stops responding during native
  runner development.
- Updated: graph renderer status now carries a typed desktop platform
  projection (`linux`, `macos`, `windows`, or `unsupported`) selected behind
  the desktop renderer strategy boundary. The native window runner still must
  prove backend-owned open/focus/close behavior per platform before reporting
  visible-window support.
- Updated: the native window runner gate must no longer treat a Linux proof as
  cross-platform completion. Bevy/winit's `run_on_any_thread` strategy may be
  proven for Linux and Windows, while macOS needs a separate main-thread or
  Tauri-runtime-compatible strategy before it can report visible-window
  support. Platforms without a proven strategy must remain typed unsupported
  through backend status.
- Updated: native renderer platform strategy selection now lives behind a
  desktop-owned `NativeRendererPlatformStrategy` boundary. Linux worker-thread,
  Windows worker-thread, macOS main-thread, and unsupported-platform outcomes
  are selected explicitly while all unproven platforms continue to report
  pending native runner capability.
- Updated: the native runner handle now starts through
  `NativeRendererPlatformStrategy` instead of hard-coding a generic pending
  runner. Current platform and explicit strategy startup share the same bounded
  request/reply owner path, while all real window candidates still report
  pending or unsupported capability until their smoke proof passes.
- Resolved: the native runner platform strategy now exposes an explicit
  threading model. Linux and Windows are worker-thread proof candidates, macOS
  is a main-thread proof candidate, and unsupported platforms cannot enter the
  minimal-window proof path.
- Resolved: graph renderer status now projects the native runner threading
  model through the backend command contract and TypeScript mirror, so the UI
  can display platform proof state without owning platform inference logic.
- Updated: `eidetic-bevy-bible-graph` now owns a minimal native-window runner
  configuration path for the Bevy/winit smoke scene. Desktop platform strategy
  still gates execution and visible-window support; this slice only provides
  the leaf renderer configuration needed by the later smoke proof.
- Updated: the desktop platform strategy now maps proof candidates to the
  minimal Bevy window config: Linux/Windows set the any-thread winit flag,
  macOS keeps main-thread execution, and unsupported platforms produce no
  runner config.
- Updated: native runner startup is now represented as a typed backend plan:
  supported proof candidates carry threading plus minimal-window config, while
  unsupported platforms remain pending-only before any event loop is started.
- Updated: a dedicated `eidetic-native-renderer-smoke` desktop binary now owns
  the manual Bevy/winit minimal-window smoke path. It runs outside the main app
  so Milestone 8 can validate native window behavior before production status
  reports verified visible-window support.
- Updated: the smoke binary is diagnostic-only. Production visible-window
  support still requires a desktop-owned `NativeRendererSupervisor` proof under
  the Tauri runtime; a standalone Bevy/winit window proof must not flip
  production capability status by itself.
- Updated: the diagnostic smoke binary now accepts a nonzero auto-close
  duration so native-window preflight commands can close deterministically
  without manual interaction. This remains a diagnostic aid only and does not
  mark production visible-window support as verified.
- Resolved: the diagnostic smoke scene now explicitly installs the Bevy
  accessibility and input plugins required by Bevy/winit window creation and
  uses a wall-clock auto-close deadline instead of requiring Bevy time. Local
  Linux X11 preflight command
  `cargo run -p eidetic-desktop --bin eidetic-native-renderer-smoke -- --any-thread --auto-close-ms 500`
  exited successfully. This proves the standalone diagnostic window path only;
  production capability stays pending until the desktop supervisor owns the
  event-loop lifecycle under Tauri.
- Updated: the remaining blocker will be resolved by replacing the pending
  runner with one supervisor-owned production path per verified platform, not
  by adding frontend fallbacks, compatibility layers, or parallel renderer
  ownership. Unproven platforms keep typed unavailable status until their own
  supervisor proof passes.
- Updated: `NativeRendererSupervisor` now owns the native renderer state
  machine behind the runner handle. Status projects an explicit supervisor
  lifecycle through Rust and TypeScript, while visible-window support remains
  pending until the production event-loop proof is implemented.
- Updated: renderer-window status now carries an explicit backend-owned
  verified-support field through Rust and TypeScript. UI command delivery and
  waiting/unavailable display logic can no longer treat size, visibility, or
  visible-support booleans as proof that the native runner is verified.
- Resolved: renderer-window status now carries an explicit typed
  unsupported/capability reason through the Rust status DTO, TypeScript mirror,
  and UI status display. Svelte no longer has to infer pending, unsupported,
  error, or verified support states from messages, platform names, or boolean
  combinations.
- Updated: bible render graph projection reads no longer mirror their response
  into the Bevy renderer. Renderer projection mutation now flows through a
  shared desktop-owned projection refresh module used by graph renderer
  open/request-update commands and the desktop mutation-event bridge. The
  remaining native runner slice still needs to turn this into a long-lived
  renderer subscription before the Bevy graph becomes the primary visual
  surface.
- Resolved: graph renderer projection writes now flow through
  `GraphRendererProjectionOwner`, a managed desktop owner that stores the
  active request, coalesces refreshes, loads backend projections, and performs
  renderer writes. Svelte request changes and backend-event invalidations no
  longer own parallel projection write paths.
- Resolved: graph renderer projection refreshes now enter through the managed
  desktop projection owner, which coalesces overlapping refresh requests into
  one in-flight projection load plus one follow-up refresh.
- Resolved: the temporary Svelte-facing projection write command semantics have
  been replaced with an active renderer projection request update. Svelte sends
  bounded request input only; the desktop-owned projection owner loads and writes
  the renderer projection through the coalesced backend path.
- Resolved: graph renderer commands now flow through the desktop event bridge
  instead of a Svelte command-drain interval. The frontend consumes typed
  selection/inspect command events through the same server-event client as
  other backend-owned events.
- Resolved for the bounded prototype: the Bevy graph scene and native visual
  paths still rebuild by despawning and respawning projection entities, but the
  renderer crate now rejects snapshots above the documented full-rebuild
  envelope of 500 nodes and 1,000 edges. Larger or more frequent primary graph
  views must add keyed entity diffing by node/edge/influence ID before this
  rebuild strategy can be expanded.
- Resolved: stale embedded-viewport documentation and the unused
  `raw-window-handle` dependency were removed after the production child-surface
  path was rejected. Raw handles must be reintroduced only through the native
  runner safety and platform verification path if a later gate proves they are
  required.
- Resolved: AI bible context projection now consumes the bounded bible render
  graph query defaults instead of loading every non-system graph node. Prompting
  context is capped by the same node/edge limits as the interactive graph while
  the later agent harness milestone still replaces this preview/generation
  context with explicit graph tools.
- Resolved: context stack projections now prefer latest recorded distilled
  context evaluations for each selected timeline ancestor before falling back
  to timeline node recap text. Lower hierarchy layers can consume refined
  parent context from backend-owned context evaluation history without
  re-evaluating the whole bible graph in Svelte or Bevy.
- Resolved: context evaluation service writes now use the evaluation payload's
  `created_at_ms` for command and change-event history instead of recording
  zero-valued timestamps. This keeps context influence history review,
  before/after traceability, and undo/redo ordering aligned with the actual
  evaluation time.
- Resolved: graph renderer focus commands now route through the desktop
  renderer owner and native runner boundary instead of returning passive status
  from the Tauri command adapter. The supervisor still reports unsupported
  focus truthfully until a real native window runner lands.
- Resolved: bounded bible render graph requests now include and enforce a
  `max_edges` limit in core contracts, frontend request builders, and the
  SQLite-backed query path. Dense local graph neighborhoods can no longer
  return an unbounded edge set after node/depth/search limits have been
  applied.
- Updated: desktop server-event, graph-renderer projection, and graph-renderer
  command bridges now have a `DesktopEventBridgeOwner` that tracks spawned task
  handles and aborts them on app/window shutdown.
- Updated: frontend renderer-window status now has a focused projection/status
  store used for display and renderer lifecycle controls. Durable graph facts
  and selection-changing commands still flow through backend projections and
  renderer command application helpers.
- Resolved: `AppShell.svelte` is back under the component decomposition
  threshold. Central workspace rendering now lives in `AppWorkspace.svelte`,
  graph detail selection lives in `GraphRightInspector.svelte`, and AI status
  markup lives in `AiStatusIndicator.svelte`, leaving the shell focused on
  layout composition and panel sizing.
- Resolved: `crates/server/src/routes/commands.rs` and `crates/server/src/routes/commands_tests.rs` exceeded the decomposition thresholds while owning many command handlers and route tests. Timeline command handlers and command route coverage were split into focused modules before adding more semantic proposal or Bevy bridge command surfaces.
- Resolved: `crates/server/src/routes/projections_tests.rs` exceeded the decomposition threshold after adding SQLite-backed story arc route coverage. Script, timeline, and story projection route tests were split into a focused out-of-line module.
- Resolved: frontend bible editing mutated broad `Entity` caches and whole detail objects. Legacy entity detail, node-link display/unlinking, websocket entity refreshes, and `storyState.entities` were removed; UI bible edits now use focused graph projection stores instead of broad entity cache patching.
- Resolved: `crates/server/src/bible_graph_store.rs` exceeded the 500-line decomposition threshold while owning schema setup, node state, part/field state, and projection reads. It was split into schema, node/projection, and part/field storage modules before edge/snapshot work.
- Resolved: the first script document store implementation exceeded the 500-line decomposition threshold while owning schema setup, value codecs, current-state writes, projection reads, and tests. It was split into schema, codec, store, and test modules before the script command route was committed.
- Resolved: `cargo check -p eidetic-server` reported non-test dead-code warnings in `history_store.rs` (`RevisionOperation` import and `load_command`). The replay helper is now test-only and the production import set is warning-free.
- Resolved: `ui/src/lib/components/sidebar/bible/StoryBibleTab.svelte` exceeded the 250-line component decomposition threshold after moving list/navigation to graph projections. Category/root mapping and graph-node creation controls were extracted before schema editor work.
- Resolved: bible graph, story arc, projection primitives, timeline models/helpers, AI runtime, websocket, project/reference, and child planning DTOs were split out of `ui/src/lib/types.ts` into focused modules, and legacy entity DTOs were removed from the frontend type surface. `types.ts` is now a compatibility barrel; new code should import focused owner modules directly instead of adding contracts to the barrel.
- Resolved: legacy AI extraction and consistency routes read `node.content.content` and committed bible/script side effects directly. Those routes, frontend consumers, automatic generation follow-up mutation, and emitted websocket events were removed; future semantic work must re-enter through proposal contracts.
- Resolved: `unlock_node` derived content status from legacy `node.content.content`. Unlock now leaves status unchanged because script document projections own durable screenplay text.
- Resolved: `ui/src/lib/stores/bibleGraphNodeProjection.svelte.test.ts` exceeded the 500-line decomposition threshold while covering list, detail, create, field, and edge cache behavior. Read/cache behavior and command cache-write behavior were split into separate test files before schema editor work.
- Resolved: `crates/server/src/timeline_command.rs` exceeded the 500-line decomposition threshold after adding timeline command history recording. History-recording helpers were split into `timeline_command_history.rs` so the mutation applicator remains easier to reason about before the larger node-delete and child-replacement slices.
- Resolved: Milestone 7 route/service extraction still had Axum-shaped route
  handlers and route tests after equivalent service and Tauri command surfaces
  existed for the active desktop path. The legacy route tree, route tests,
  Axum-only error adapter, and direct Axum/Tower server dependencies were
  deleted; future coverage must target backend services or Tauri adapters.
- Resolved: the first Tauri dependency resolution selected `tauri` 2.10.3 with
  newer 2.11 runtime crates, which failed inside `tauri-runtime-wry`. The desktop
  crate now pins `tauri` to 2.11.2 so the runtime stack resolves consistently.
- Resolved: Milestone 7 lifecycle compliance was blocked by detached backend
  tasks. Autosave, Y.Doc, AI generation, batch generation, and reference
  embedding now run through the backend task supervisor, and Tauri window
  teardown aborts supervised work through the shared backend lifecycle owner.
- Resolved: the root launcher still built and ran the legacy `eidetic-server`
  binary, waited for `127.0.0.1:3000`, and opened a browser. The launcher now
  targets `eidetic-desktop`, uses Vite only for the Tauri development webview,
  and release runs execute the desktop binary directly.
- Resolved: the frontend still had a browser WebSocket fallback that owned Yjs
  binary sync and could act as a second realtime transport. Backend events now
  require Tauri event transport, and notes editing remains command/projection
  driven without frontend Yjs synchronization.
- Resolved: `ui/src/lib/api.ts` still carried browser HTTP fallback behavior
  after equivalent Tauri commands existed for its active surfaces. The helper
  now uses desktop IPC directly and reports missing Tauri transport as a
  configuration error.
- Resolved: `ui/src/lib/projectionApi.ts` still carried browser HTTP fallback
  behavior after equivalent Tauri commands existed for active projection reads.
  Projection helpers now use desktop IPC directly and report missing Tauri
  transport as a configuration error.
- Resolved: `ui/src/lib/timelineCommandApi.ts` still carried browser HTTP
  fallback behavior after equivalent Tauri commands existed for timeline
  mutations. Timeline command helpers now use desktop IPC directly and report
  missing Tauri transport as a configuration error.
- Resolved: `ui/src/lib/commandApi.ts` still carried browser HTTP fallback
  behavior after equivalent Tauri commands existed for non-timeline command
  surfaces. Command helpers now use desktop IPC directly, wrapper tests no
  longer preserve route URLs as frontend behavior, and `commandTransport.ts`
  no longer owns fetch behavior.
- Resolved: `eidetic-server` still exposed a standalone Axum listener, static
  web host, and WebSocket host after the launcher moved to Tauri. The server
  crate no longer builds a listener binary or static/WebSocket runtime.
- Resolved: `crates/server/src/command_service.rs` reached 681 lines after bible
  graph command extraction. Bible graph command handling now lives in the
  focused `command_service_bible.rs` module and shared helpers live in
  `command_service_support.rs`, reducing the main service file below the
  decomposition threshold before further command migration.
- Resolved: `src-tauri/src/bevy_graph_host.rs` exceeded the 500-line
  decomposition threshold after adding the renderer owner. The desktop Bevy
  graph boundary is now split into focused host, owner, and test modules before
  adding projection subscription plumbing.
- Resolved: the desktop bible render graph projection command returned the
  backend-owned graph projection only to Svelte. It now mirrors the same
  projection payload into the managed Bevy renderer owner when available,
  giving the renderer thread a projection-driven data path without creating a
  second durable graph owner.
- Resolved: the native graph renderer command surface validated node and
  influence selections but omitted edge selection while Milestone 8 requires
  typed transient edge selection. The Bevy renderer and desktop owner now
  validate projected edge IDs and emit `select_edge` renderer commands.
- Resolved: the native graph renderer owner had no desktop IPC read surface for
  lifecycle status or validated renderer commands. Tauri now exposes status and
  command-drain commands that return transport-safe projections without giving
  the renderer durable graph mutation authority.
- Resolved: the frontend had no transport contract for native graph renderer
  status or drained interaction commands. It now has typed desktop helpers and
  a pure command application adapter that updates transient graph selection
  only, leaving durable graph facts backend-owned.
- Resolved: the graph workspace did not own a native renderer command-drain
  lifecycle, so Bevy selections would not reach the Svelte detail/review
  surfaces. The workspace now starts a bounded drain loop on mount and applies
  validated renderer commands to transient graph selection only.
- Superseded: Milestone 8 previously named a shared embedded viewport host and
  the desktop runtime added a command-side lifecycle registry for graph/timeline
  panel state. That registry must now be replaced or renamed around floating
  renderer window lifecycle before more renderer work lands.
- Superseded: the frontend added embedded viewport helpers and a borderless
  Svelte panel lifecycle surface for graph/timeline viewports. Those APIs
  should not be extended as a production path; Svelte should expose floating
  renderer launch/focus/status/close controls instead.
- Superseded: graph workspace mode mounted the `graph-main` borderless viewport
  panel while the Svelte outline remained the temporary visible semantic graph.
  The visible graph replacement now targets an app-managed floating Bevy graph
  window.
- Superseded: graph viewport lifecycle currently starts the desktop graph
  renderer owner. The next renderer slice must move this lifecycle trigger to
  the floating renderer host so graph/timeline windows share one standards-
  compliant lifecycle owner.
- Open: the central graph workspace still uses the Svelte outline as the
  visible graph surface while Bevy is a projection/command consumer. Finishing
  Milestone 8 now requires app-managed floating Bevy graph window integration
  with a fresh render/window dependency review before replacing the Svelte
  outline as the primary visual graph.
- Resolved: the fresh Bevy 0.18.1 graph-render dependency review confirmed the
  graph crate currently enables only the `std` feature and keeps Bevy isolated
  to the leaf renderer. Native visual rendering must land in a separate slice
  that opens an app-managed floating renderer window, accepts backend-owned
  projection updates, and proves a minimal scene before graph nodes/edges
  replace the Svelte outline.
- Resolved: graph visual styling is now renderer-owned instead of being a
  future Svelte concern. The Bevy bible graph leaf crate derives disposable
  visual primitives for projection nodes and edges, including positions,
  colors, radii, stroke widths, and influence highlight flags for the native
  panel renderer to consume.
- Resolved: renderer-owned graph visual primitives now cross the desktop
  boundary through an explicit Tauri command and TypeScript mirror types. This
  keeps future native panel smoke tests and diagnostics on the same Bevy-owned
  visual contract without exposing Bevy ECS internals to Svelte.
- Resolved: native Bevy graph rendering now has an explicit opt-in feature gate
  instead of expanding the default renderer dependency surface. The
  `native_render` feature enables the reviewed Bevy 2D render/window/winit stack
  with Linux Wayland/X11 backends. Existing borderless-panel naming should be
  retired or isolated as part of the floating renderer host slice.
- Resolved: the native graph renderer now has a feature-gated scene setup that
  records Eidetic graph colors, sets the Bevy clear color, and spawns exactly
  one marked `Camera2d`. The remaining work is floating window lifecycle, not
  Tauri surface embedding.
- Resolved: the desktop graph host now enables the graph renderer's
  `native_render` feature and starts the renderer through the current native
  constructor. Renderer status diagnostics should be renamed from panel-centric
  readiness to floating renderer window readiness in the next lifecycle slice.
- Superseded: the current embedded viewport host is still a Tauri command-side
  bounds and lifecycle registry and does not own or attach a native child
  surface for Bevy inside the Svelte panel. This child-surface embedding path is
  no longer the production target. The next renderer slice must replace or
  rename that host around floating renderer window lifecycle instead of adding
  platform-specific WebView child-surface attachment.
- Superseded: embedded viewport state includes a typed native surface
  attachment status. That diagnostic can remain while the experiment is removed
  or retired, but production graph rendering should report floating renderer
  window lifecycle/readiness instead of panel attachment readiness.
- Resolved: the SQL-backed bounded bible render graph query now reapplies the
  normalized `max_nodes` limit after ancestor expansion. This prevents focused
  node, focused root, or influence queries from exceeding the backend projection
  size boundary while preserving required node priority inside the limit.
- Resolved: the graph workspace now refreshes bible render graph projections
  with an explicit bounded request keyed to the selected timeline node. The
  central graph view follows selected-clip context through backend projection
  reads instead of relying only on a default whole-graph cache refresh.
- Resolved: project activation and the sidebar bible outline now use the same
  explicit bounded render graph request shape as the central graph workspace,
  keeping interactive graph projection reads consistently request-shaped across
  Svelte surfaces.
- Resolved: context influence writes now emit a typed backend event for the
  affected timeline node. The desktop Bevy projection bridge and Svelte event
  handlers refresh bounded graph projections from that event, so influence path
  highlights are invalidated by the context write that changed them instead of
  waiting for an unrelated bible or timeline mutation.
- Resolved: desktop validation exposed dead-code warnings for the planned
  native surface attachment transition state. Those future-use APIs are now
  marked with explicit lint-allow reasons, but the remaining Milestone 8
  renderer slice should retire or replace the attachment state with floating
  renderer window lifecycle state.
- Resolved: the desktop Bevy projection bridge now maps context influence
  change events into selected-timeline bounded render graph requests for the
  affected timeline node instead of refreshing the renderer with the default
  graph projection.
- Resolved: the native Bevy graph panel now rebuilds projection-derived visual
  node and edge components inside the renderer ECS whenever a projection lands.
  This prepares the native scene to render graph primitives from backend-owned
  projections while the visible floating renderer window lifecycle remains open.
- Resolved: feature-enabled graph crate tests exposed that plain renderer apps
  compiled with `native_render` could try to update native visual resources
  without installing the native plugin. Native visual rebuilds now no-op unless
  the native panel resource exists, preserving the leaf renderer's headless
  projection tests.
- Resolved: desktop graph renderer status now reports native visual node and
  edge counts separately from logical projection scene counts, giving smoke
  checks a backend-owned diagnostic that the native Bevy panel has consumed the
  projection into render-prep components.
- Superseded: embedded viewport surface state records a typed platform
  attachment strategy in addition to pending/attached status. This remains
  useful as a record of the abandoned embedding experiment, but production work
  should replace it with floating renderer window lifecycle status.
- Resolved: graph renderer startup now seeds the native Bevy graph renderer from
  the backend-owned render graph projection service immediately after renderer
  startup. A failed seed rolls back renderer startup so the desktop host does
  not report a running graph window without an initial backend projection.
- Resolved: graph renderer startup accepts the same bounded render graph request
  used by the graph workspace, so initial Bevy renderer seeding is scoped to the
  selected timeline clip/playhead context instead of racing a default whole-view
  seed against the workspace projection refresh.
- Superseded: embedded viewport mounting rejects duplicate viewport ids instead
  of replacing an existing panel lifecycle record. The floating renderer host
  should preserve the one-owner rule with renderer-window IDs instead of
  mounted-panel IDs.
- Superseded: graph viewport mount and resize propagate validated physical
  panel bounds into the native Bevy graph host. The floating renderer host may
  accept size/placement hints, but it should not depend on WebView panel bounds
  or child-surface smoke checks.
- Decision: local dependency inspection confirmed that native WebView
  child-surface attachment would require platform-specific paths instead of a
  portable `RawWindowHandle` embedding API. Wry documents raw parent-handle
  child windows as Linux X11-only and recommends GTK container embedding for
  Wayland; winit exposes unsafe `with_parent_window` child-window creation, but
  the current Bevy host does not own a portable embedding lifecycle. Milestone
  8 should avoid that maintenance burden by moving to app-managed floating
  Bevy renderer windows.
- Superseded: embedded viewport surface state separates detected parent surface
  capability from renderer child-window lifecycle. The floating renderer host
  should keep explicit renderer-window lifecycle states but remove the
  production dependency on detected parent-surface capabilities.
- Superseded: X11/XCB surface detection captured the native parent window id
  into the renderer child-window lifecycle state for the embedding experiment.
  Production graph rendering no longer depends on that parent id; it should use
  backend-owned floating renderer window lifecycle instead.
- Superseded: the deleted frontend graph renderer command-drain bridge had
  accepted invalid polling intervals and needed teardown coverage. Renderer
  commands now flow through the desktop event bridge, so frontend polling
  interval validation is no longer part of the production design.
- Resolved: selected timeline context influences could be loaded by the bounded
  SQL query and then filtered out by the pure render adapter when the same
  request also included search/focus filters. Influence nodes and influenced
  edge endpoints are now treated as required within the already-bounded graph
  input, with core and server regression coverage.
- Resolved: bounded render graph SQL search treated user `%`, `_`, and
  backslash input as SQLite `LIKE` wildcards even though the pure projection
  adapter treats search text literally. Search patterns are now escaped at the
  query boundary with regression coverage for literal wildcard characters.
- Resolved: the desktop graph renderer visual-snapshot read path implicitly
  started the renderer through the shared mutable renderer helper. Snapshot
  reads now report the missing projection/window state without creating
  renderer lifecycle state, preserving backend-owned lifecycle boundaries for
  read-only diagnostics.
- Resolved: the graph workspace transient node selection did not participate in
  the bounded render graph request, so selecting a bible node could not ask the
  backend for that node's neighborhood. Workspace projection requests now carry
  both selected timeline context and selected graph node focus while keeping
  durable graph data backend-owned.
- Resolved: an already-open graph renderer was only seeded on open, so later
  workspace request changes could refresh the Svelte projection without
  updating the Bevy projection consumer. The desktop bridge now exposes a
  bounded projection sync command, and the renderer controls reseed open graph
  windows when the backend projection request changes.
- Resolved: desktop backend-event refreshes for the graph renderer used
  default/event-derived projection requests, which could overwrite the
  UI-focused bounded request active in the open renderer. The desktop host now
  tracks the active graph renderer projection request and event refreshes reuse
  that backend-owned request shape.
- Resolved: the Story Bible tab applied search/category filters only to the
  local list while the render graph request still used the broad timeline-only
  projection. The tab now projects search text, category root focus, selected
  graph node, and selected timeline context into the backend-bounded render
  graph request.
- Resolved: frontend backend-event handlers refreshed bible render graph
  projections with default or event-derived timeline-only requests, which could
  overwrite a focused graph/search projection in the cache. The projection
  store now tracks the active bounded request and event refreshes reuse that
  request.
- Resolved: explicit bounded bible render graph searches that matched no nodes
  fell back to the default graph, which made empty filter results display
  unrelated bible state. The SQL-backed render graph query now only uses default
  nodes for unfiltered projection requests while preserving timeline influence
  seeding.
- Resolved: bible render graph projections were produced from bounded requests
  but did not expose the focused root, selected graph node, or selected
  timeline node that shaped the projection. The projection DTO now carries
  request identity metadata so Bevy and Svelte consumers can stay projection
  driven instead of reconstructing ownership from local state.
- Resolved: the pure core bible render graph adapter still returned the default
  graph for explicit empty filters even after the SQL-backed bounded query was
  fixed. Core and SQL projection paths now agree that an explicit search or
  focus filter with no matches produces an empty bounded graph.
- Resolved: graph outline selection display still used local transient
  selection even though bounded render graph projections now expose the
  selected graph node that shaped the backend response. Svelte graph outlines
  now prefer projection-selected identity for display while preserving
  transient selection as request input and explicit test-only override.
- Resolved: the desktop graph renderer close command used the owner-thread
  shutdown path, so closing the graph renderer left the managed owner unable to
  reopen the floating renderer during the same app session. Renderer close now
  drops renderer state while keeping the owner thread available; owner shutdown
  remains reserved for teardown/drop.
- Resolved: graph renderer open rollback paths also used terminal owner
  shutdown after size-hint or projection-seeding failures. Failed opens now
  close renderer state through the reusable lifecycle path so the app can retry
  without recreating the managed desktop owner.
- Resolved: the Tauri bible render graph projection read command mirrored every
  projection into the Bevy owner, and that mirror path could start renderer
  lifecycle from a passive read. Projection reads now mirror only when the graph
  renderer is already open; explicit renderer open/set-projection commands own
  renderer startup.
- Resolved: open-renderer projection refreshes used separate status and
  set-projection owner requests, so a close racing between them could restart
  the renderer from a passive/event refresh. The desktop graph owner now exposes
  an atomic update-if-open request used by projection mirrors, event refreshes,
  and set-projection commands.

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

- Completing the frontend projection-only boundary before more renderer integration.
- Timeline nodes as timed context chunks.
- Composable story bible graph, schemas, parts, edges, snapshots, assets, and references.
- Script document, script segments, blocks, spans, locks, provenance, and patch proposals.
- Semantic claims, dependencies, propagation proposals, change review, undo/redo, and before/after history.
- SQLite schema and repositories for canonical state, revisions, projections, and assets.
- Command, event, and projection DTOs.
- Bevy timeline and bible graph renderer windows as projection consumers.
- Svelte shell, forms, inspectors, editors, and accessibility command alternatives.
- Tests, documentation, lifecycle management, validation, and dependency placement required by standards.

Out of scope:

- Building a second/parallel frontend for the projection UI.
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

## Milestone 6: Frontend Projection-Only Completion

This is the next section to complete before further renderer replacement work. The goal is not a separate frontend. The goal is one Svelte shell whose durable reads come from backend projections and whose durable writes go through backend commands.

Standards reviewed for this milestone:

- `Coding-Standards/ARCHITECTURE-PATTERNS.md`: backend-owned data, dependency direction, executable boundary contracts, single owner for stateful flows.
- `Coding-Standards/CODING-STANDARDS.md`: backend-owned data, no optimistic updates, file/component decomposition review, single responsibility.
- `Coding-Standards/FRONTEND-STANDARDS.md`: declarative rendering, event-driven synchronization, timer/request cleanup, selector strategy, embedded gesture-control checks.
- `Coding-Standards/CONCURRENCY-STANDARDS.md`: message passing over shared mutable state and stale async response guards.
- `Coding-Standards/TESTING-STANDARDS.md`: vertical slice verification, replay/recovery/idempotency checks, async lifecycle regression checks.
- `Coding-Standards/DOCUMENTATION-STANDARDS.md`: README traceability, API consumer contract, structured producer contract.
- `Coding-Standards/SECURITY-STANDARDS.md`: runtime validation at process/API boundaries and no trusted renderer payloads.
- `Coding-Standards/TOOLING-STANDARDS.md`: lint/typecheck/test/static gate enforcement.

Allowed frontend-owned state:

- current selection, hover, focus, panel state, menus, draft form fields, drag previews, scroll, zoom, active tool, and renderer viewport/camera/playhead state;
- pending/loading/error state for requests;
- discardable projection caches keyed by backend projection identity and version.

Forbidden frontend-owned state:

- timeline structure, node ranges, track/lane definitions, relationships, story arcs, bible facts, script blocks, script locks, semantic dependencies, accepted/pending proposals, AI outputs, undo/redo history, or broad project mirrors treated as editable state.

Tasks:

- Audit every frontend store and classify it as `projection cache`, `transient UI state`, or `legacy ownership`.
- Delete or rewrite every `legacy ownership` store/path so durable changes enter through backend command helpers and projection responses.
- Make focused projection stores the only durable read source for timeline render data, script documents, bible graph details/schema/render graph, story arcs, semantic proposals, propagation proposals, change review, AI status/config, child plans, and project/reference metadata.
- Remove remaining UI code that hydrates durable state from broad `Project` or `Timeline` DTOs when a focused projection exists.
- Remove local patch/update helpers that mutate durable frontend objects after saves; command responses must replace projection caches or trigger projection refresh.
- Ensure websocket handlers refresh projections or invalidate caches; they must not patch durable entities locally.
- Keep Svelte timeline components temporarily only as projection consumers and command emitters; do not add new durable timeline behavior to the DOM renderer.
- Add static/lint/test enforcement for banned legacy APIs, broad durable mutations, and direct writes to projection cache payloads outside cache replacement functions.
- Update frontend READMEs to document each touched store/component as projection cache or transient UI state.
- Validate renderer-originated command payloads at the backend/API boundary. Frontend TypeScript types are not a trust boundary; command IDs, node IDs, ranges, relationship endpoints, notes, locks, and project/session identifiers must be checked by shared backend validators before state changes.
- Add decomposition reviews for any touched store/module over roughly seven public functions and any touched Svelte component over 250 lines. Split by responsibility when the milestone adds code to an already large component.

Implementation order:

- Create the store audit table first so every follow-on edit has an explicit owner classification.
- Split `ui/src/lib/stores/timeline.svelte.ts` into transient timeline viewport/tool state and projection-derived selectors. Remove `timelineState.timeline`; keep zoom, scroll, playhead, viewport width, active tool, snapping, and connection drag as frontend-owned transient state.
- Convert the temporary Svelte timeline renderer to `TimelineRenderProjection`/`TimelineRenderModel` inputs only. `Timeline.svelte`, `LevelTrack.svelte`, `StoryNodeClip.svelte`, `StructureBar.svelte`, `RelationshipLayer.svelte`, and timeline gaps must read from projection-backed/cache-backed models instead of broad `Timeline` DTOs.
- Replace `editorState.selectedNode` with selected identifiers and transient editor state. The selected node's durable fields must come from a focused projection or the timeline render model, not a stored `StoryNode` object.
- Remove broad project/timeline hydration from project create/load. `Project` responses may open a backend session and provide lightweight project metadata, but visible durable state must be refreshed through projections.
- Rewrite websocket handling so timeline, hierarchy, script, bible, generation, and proposal events refresh or invalidate affected projection caches. Do not call broad read APIs and patch frontend durable objects in event handlers.
- Move AI status/config out of the editor store into a dedicated projection/status cache so the editor store remains transient editor interaction and generation-progress state.
- Replace legacy editor mutations for notes, locks, and generation status with command helpers plus projection cache replacement/refresh. Draft text fields may be local while editing, but persisted note content, lock state, and generated status are backend-owned.
- Coalesce websocket-triggered projection refreshes per projection identity when event bursts arrive, so removing broad mirrors does not introduce redundant projection fetches.

Concurrency and lifecycle requirements:

- Projection refresh coalescing must have one explicit owner module. Components and websocket handlers may request a refresh/invalidation, but they must not independently start overlapping refresh state machines for the same projection identity.
- Every async projection refresh must guard against stale responses overwriting newer projection versions. Use request IDs, projection version checks, abort/cancellation, or an equivalent explicit stale-response guard.
- Websocket subscriptions, timers, debounced note saves, and refresh coalescers must have deterministic teardown tests for unmount/project close/reconnect.
- Event bursts must be handled by queued or coalesced projection refreshes, not by shared mutable broad DTO mirrors.
- Failed refreshes must preserve the last confirmed projection cache and surface bounded diagnostic context through the existing error/pending state; failed requests must not clear durable visible data unless the backend confirms the project/session is gone.

Accessibility and frontend interaction requirements:

- Timeline and editor command controls remain keyboard-accessible while the DOM timeline is temporary. Pointer-only timeline interactions need semantic or keyboard command alternatives before the DOM path is considered compliant.
- New or modified buttons must use semantic `<button type="button">` elements or an explicitly justified accessible generic element with role, focus, keyboard handling, and accessible name.
- Embedded controls inside draggable, pannable, zoomable, or timeline gesture areas require smoke checks for pointer capture/release, focus/blur, keyboard access, Escape/cancel behavior, and parent gesture conflicts.
- Tests should prefer accessible selectors such as role plus name. Geometry-dependent timeline tests must isolate pure geometry helpers or mock `getBoundingClientRect()` explicitly.

Known legacy ownership paths to remove:

- Resolved: `ui/src/lib/stores/timeline.svelte.ts` no longer stores
  `timelineState.timeline` and no longer exports broad timeline query helpers.
- Resolved: `ui/src/lib/stores/editor.svelte.ts` no longer stores
  `selectedNode: StoryNode | null`; selected-node durable fields now come from
  `selectedNodeEditorProjection.svelte.ts`.
- Resolved: `ui/src/lib/stores/wsHandlers.ts` refreshes projection caches on
  timeline, hierarchy, script, bible, generation, and proposal events instead
  of calling broad timeline/content reads or patching selected node objects.
- Resolved: websocket-triggered projection refreshes now go through
  `projectionRefreshQueue.ts`, which coalesces event bursts per projection
  identity and schedules one follow-up refresh if another request arrives while
  a refresh is in flight.
- Resolved: `aiStatus.svelte.ts` owns the AI status polling lifecycle and now
  guards overlapping refreshes with request IDs so stale status responses cannot
  overwrite newer backend status.
- Resolved: `ui/src/lib/components/layout/SplashScreen.svelte` no longer
  hydrates broad `Project`/`Timeline` DTOs into durable frontend stores; the
  project store keeps active-session metadata only.
- Resolved: `ui/src/lib/components/timeline/Timeline.svelte` renders the
  temporary DOM timeline shell from `TimelineRenderProjection` /
  `TimelineRenderModel`, including structure, gaps, tracks, and command hit
  targets.
- Resolved: `ui/src/lib/components/timeline/LevelTrack.svelte` renders clips
  and arc colors from `TimelineRenderModel` instead of broad timeline node/arc
  helpers.
- Resolved: `ui/src/lib/components/editor/BeatEditor.svelte` reads selected
  node, parent, children, siblings, adjacent parents, notes, lock, and content
  status from `SelectedNodeEditorProjection` and refreshes that projection
  after editor commands.
- Resolved: `ui/src/lib/README.md` now documents projection refresh usage
  instead of assigning broad timeline/project data into frontend stores.

Discovered implementation gaps:

- Resolved: `TimelineRenderProjection` now exposes structure bar segments and
  gaps, and `TimelineRenderModel` has selectors for visible tracks, clips by
  track/level, node lookup, and adjacent clip bounds.
- Resolved: `BeatEditor.svelte` now uses a focused selected-node/editor
  projection with node identity, hierarchy context, notes, lock state, content
  status, children, parent, siblings, adjacent parents, and child-level metadata.
- Resolved: legacy broad HTTP read helpers and server routes for `/timeline`,
  `/timeline/nodes/{id}/children`, `/timeline/gaps`, and
  `/nodes/{id}/content` were deleted after grep confirmed projection-migrated UI
  paths no longer call them.
- Resolved: timeline create-node, split-node, and create-relationship routes now
  accept omitted result IDs, derive stable backend-owned IDs from command ID and
  result role for idempotent replay, and return the confirmed timeline render
  projection. The temporary Svelte timeline no longer generates node or
  relationship IDs for those commands.
- Resolved: timeline apply-children routes now accept omitted child node IDs,
  derive stable backend-owned child IDs per command ID and child index for
  idempotent replay, and the editor no longer generates child node IDs when
  applying AI child plans.
- Resolved: story arc creation now accepts omitted arc IDs, derives a stable
  backend-owned arc ID from the command ID for idempotent replay, and the arc
  list UI no longer generates durable arc IDs.
- Resolved: bible graph node, edge, snapshot, and snapshot-field commands now
  accept omitted durable IDs, derive stable backend-owned IDs with existing
  graph prefixes from the command ID, and the bible UI no longer generates those
  durable IDs locally. The remaining frontend `crypto.randomUUID()` call is the
  command-envelope idempotency key in `commandApi.ts`.
- Resolved: bible graph command routes and snapshot-field route tests were split
  out of the shared command route/test modules. `commands.rs`,
  `commands_bible.rs`, `commands_bible_tests.rs`, and
  `commands_bible_snapshot_tests.rs` are all under decomposition thresholds.
- Resolved: `projectionCacheGuards.ts` now provides shared version-based
  projection replacement guards, and `timelineRenderProjection.svelte.ts` uses
  it so older refresh or command-response envelopes cannot overwrite a newer
  cached timeline render projection. `bibleGraphSchemaProjection.svelte.ts`,
  `bibleRenderGraphProjection.svelte.ts`, and
  `changeReviewProjection.svelte.ts` now use the same guard for read-only
  projection refreshes. `storyArcProjection.svelte.ts`,
  `semanticProposalProjection.svelte.ts`, and
  `propagationProposalProjection.svelte.ts` now guard both refresh and command
  response envelopes. `objectFieldProjection.svelte.ts` and
  `scriptDocumentProjection.svelte.ts` now guard keyed refresh and command
  response envelopes. `selectedNodeEditorProjection.svelte.ts` now combines
  request-id suppression with backend projection version guards.
  `bibleGraphNodeProjection.svelte.ts` now guards keyed node detail refreshes,
  node list refreshes, command response cache writes, and edge target
  invalidation against stale backend envelopes.
- Resolved: `ui/src/lib/stores/README.md` now classifies `wsHandlers.ts` as
  projection refresh orchestration instead of mixed legacy ownership, matching
  the current websocket handler implementation.
- Resolved: `projectionOnlyGuards.test.ts` now fails if deleted broad timeline
  ownership, selected-node durable object ownership, legacy timeline/content
  helpers, legacy node mutation helpers, or direct projection payload patching
  patterns are reintroduced into UI source.
- Resolved: `projectionRefreshQueue.ts` now resolves queued waiters when the
  queue is cleared during teardown/project close, so refresh coalescing cannot
  leave callers hanging after lifecycle cleanup.
- Resolved: `wsHandlers.ts` teardown now unsubscribes registered websocket
  handlers and clears queued projection refreshes, with tests proving events
  after teardown and queued pre-teardown refreshes do not mutate projection
  stores.
- Resolved: `BeatEditor.svelte` now routes debounced note saves through a
  small lifecycle-owned helper that captures the selected node ID at schedule
  time and cancels pending work on destroy, preventing stale notes from saving
  into a later selection or after unmount.
- Resolved: `ArcDetail.svelte` now routes debounced story arc metadata saves
  through a lifecycle-owned helper that captures the arc ID and field at
  schedule time and cancels pending work on destroy.
- Resolved: `ws.test.ts` now covers `WsClient` lifecycle behavior for reconnect
  timer cancellation, manual disconnect without reconnect, event handler
  unsubscribe, and detaching outgoing Yjs updates on disconnect.
- Resolved: `projectSession.ts` now owns project activation for create/load:
  it clears queued projection refreshes, transient editor and bible selection
  state, project-scoped projection caches, stores lightweight active project
  metadata, and refreshes focused timeline, story arc, script, bible, proposal,
  and change-review projections.
- Resolved: timeline apply-children route validation now rejects non-positive
  or non-finite child weights at the backend/API boundary before command
  conversion, persistence, or projection updates, matching the child-plan
  storage invariant instead of silently clamping renderer-originated values.
- Resolved: timeline command route DTOs now reject unknown top-level, payload,
  and child payload fields during JSON deserialization, so renderer-sent
  `project_id`, `session_id`, or other non-contract fields fail before backend
  command handling instead of being silently ignored.
- Resolved: split-node route coverage now proves duplicate result IDs and
  result IDs that collide with existing timeline nodes are rejected through the
  backend command API without recording a command.
- Resolved: create-node route coverage now proves blank, overlong, and
  unsupported-character names are rejected at the backend/API boundary before
  command handling or persistence.
- Resolved: bible graph node, edge, and snapshot-field route DTOs now reject
  unknown top-level and payload fields during JSON deserialization, so
  renderer-sent `project_id`, `session_id`, or non-contract graph fields fail
  before command handling instead of being silently ignored.
- Resolved: story arc create route DTOs now reject unknown top-level and payload
  fields during JSON deserialization, so renderer-sent `project_id`,
  `session_id`, or non-contract arc fields fail before backend story command
  handling.
- Resolved: the shared `CommandEnvelope<T>` now rejects unknown top-level
  fields during JSON deserialization, so command routes that deserialize
  directly into shared envelopes also reject renderer-sent `project_id`,
  `session_id`, or other non-contract envelope fields before command handling.
- Resolved: script block and script lock command payload contracts now reject
  unknown payload fields during JSON deserialization, so renderer-only script
  edit or lock metadata cannot enter backend command handling silently.
- Resolved: `projectionOnlyGuards.test.ts` now verifies the store ownership
  audit in `ui/src/lib/stores/README.md` stays aligned with every non-test
  store module, and the audit now includes project-session activation,
  projection cache guard, and projection refresh queue infrastructure.
- Resolved: touched editor and temporary timeline command controls now use
  explicit `type="button"` semantics, and `projectionOnlyGuards.test.ts` fails
  if editor/timeline Svelte command buttons omit an explicit type.
- Resolved: `timelineCommandFlow.test.ts` now covers a user-visible timeline
  lock command path through the real frontend command helper and projection
  store, proving backend-returned projections replace the cache while backend
  validation errors preserve the last confirmed projection.
- Resolved: Milestone 6 final validation passed with the full UI Vitest suite,
  full backend command route suite, UI typecheck/lint, and server check after
  the projection-only guard, accessibility, and command-flow slices landed.

Simplification opportunities:

- Add a small projection command helper/factory for repeated `pending/error/replace projection` store logic after the ownership cleanup lands.
- Move timeline adjacency, parent/child, gap, arc, relationship, and visible-track lookup into projection models or memoized projection-derived indexes instead of repeated component-local array scans.
- Treat Y.Doc note text as an editing transport/cache only; backend commands and projections remain the durable source for saved note content.
- Keep old broad DTO modules only for compatibility while endpoints remain; new UI code imports focused DTO owner modules directly.
- Prefer pure projection model adapters and selectors over component-local orchestration so timeline/editor behavior is unit-testable without DOM or server mocks.
- Keep API/projection contracts append-only during the milestone when practical. If removing legacy broad DTO use requires an API-breaking rewrite, record the compatibility impact and delete obsolete consumers in the same verified slice because this repo does not require backwards compatibility.

Verification:

- Store audit document or table exists and lists every `ui/src/lib/stores/*` owner classification.
  `projectionOnlyGuards.test.ts` now fails if any non-test store module is
  missing from `ui/src/lib/stores/README.md` or if the audit references a stale
  store file.
- Unit tests prove command responses replace projection caches without optimistic durable mutation.
- WebSocket handler tests prove events refresh/invalidate projection caches instead of patching durable objects.
- Static checks or focused tests fail if banned legacy helpers, broad project mutation paths, or old timeline mutation APIs are reintroduced.
  `projectionOnlyGuards.test.ts` covers the deleted broad timeline ownership,
  selected-node durable ownership, legacy timeline/content helper, legacy node
  mutation helper, and direct projection payload patching patterns.
- `types.ts` remains a compatibility barrel only; new code imports focused DTO owner modules directly.
- No UI component writes to backend-owned fields except through command helpers.
- Svelte timeline still renders during the transition, but only from projection-backed/cache-backed inputs and backend-confirmed commands.
- Cross-layer acceptance test covers one user-visible command path from Svelte command emission through backend validation/command handling to returned or refreshed projection display.
  `timelineCommandFlow.test.ts` covers the timeline lock command path through
  frontend command emission, backend response handling, projection cache
  replacement, and backend validation error preservation.
- Backend route tests cover invalid timeline child weights so renderer-originated
  apply-children payloads cannot bypass backend validation before projection
  replacement.
- Backend route tests cover unexpected `project_id`/`session_id` fields on
  timeline commands, and the full timeline command route suite passes with
  strict unknown-field rejection enabled.
- Backend split-node route tests cover invalid generated-result ID payloads,
  including equal left/right IDs and collisions with existing timeline node IDs.
- Backend create-node route tests cover invalid name payloads for required,
  length, and character-set validation branches.
- Backend bible graph route tests cover unexpected `project_id`/`session_id`
  fields, and the bible graph command route suite passes with strict
  unknown-field rejection enabled for node, edge, and snapshot-field commands.
- Backend story arc create route tests cover unexpected `project_id`/
  `session_id` fields, and the object/story command route suite passes with
  strict unknown-field rejection enabled for create-arc commands.
- Backend script block route tests cover unexpected `project_id`/`session_id`
  fields on direct shared-envelope command routes, and the full command route
  suite passes with strict envelope rejection enabled.
- Backend script block route tests cover unexpected payload fields, and the
  full command route suite passes with strict script block/lock payload
  rejection enabled.
- Replay/recovery/idempotency tests prove duplicate command IDs, websocket reconnects, and projection refresh after project reload do not create duplicate edits or stale UI state.
- Async lifecycle tests cover stale response suppression, refresh coalescing, debounced note save cleanup, websocket subscription cleanup, and project close/reopen.
  `projectionRefreshQueue.test.ts` covers refresh coalescing and queued-waiter
  resolution during teardown, and `wsHandlers.test.ts` covers websocket
  subscription cleanup plus queued refresh cleanup.
  `debouncedNodeNotesSave.test.ts` covers selected-node capture, newer-edit
  cancellation, and destroy cleanup for debounced note saves.
  `debouncedStoryArcMetadataSave.test.ts` covers equivalent capture,
  cancellation, and destroy cleanup for story arc metadata saves.
  `ws.test.ts` covers websocket reconnect timer cleanup and Yjs listener
  teardown on disconnect.
  `projectSession.test.ts` covers project activation ordering so project
  close/reopen paths clear lifecycle and transient state before projection
  refreshes repopulate backend-owned data.
- Accessibility checks cover keyboard alternatives and embedded timeline/editor control conflicts for any touched gesture-heavy controls.
  Editor and temporary timeline command buttons now have explicit button type
  semantics, with a static guard covering touched editor/timeline Svelte
  surfaces.
- Documentation checks confirm touched `ui/src/lib/**` directories have README ownership/lifecycle updates and that projection stores document API consumer and structured producer contract expectations where applicable.
- Typecheck, lint/static guard checks, and the affected frontend/backend test suites pass before committing each logical slice.
  Final Milestone 6 closeout validation passed:
  `npm run test`, `npm run typecheck`, `npm run lint`,
  `cargo test -p eidetic-server routes::commands -- --nocapture`, and
  `cargo check -p eidetic-server`.

Re-plan triggers:

- A focused projection required by the UI does not exist and would require new backend projection contracts beyond the milestone's intended frontend cleanup.
- Removing broad `Timeline` or `StoryNode` mirrors exposes missing backend command validation or missing projection fields.
- Refresh coalescing introduces ordering ambiguity that cannot be proven with request IDs/version checks.
- A touched component crosses the decomposition threshold and cannot be split safely inside the current slice.
- Accessibility parity for a temporary DOM interaction would require building the Bevy renderer first.

Exit criteria:

- The frontend can be described as `projection caches + transient interaction state` with no durable local ownership exceptions.
- Remaining DOM timeline replacement work can proceed without untangling durable frontend state at the same time.
- The milestone satisfies the reviewed coding standards: backend-owned durable data, no optimistic updates, validated API boundaries, single-owner async refresh lifecycle, documented ownership, accessibility-preserving controls, and verified cross-layer projection flow.

## Milestone 7: Tauri Desktop Shell And WASM Removal

This milestone is a prerequisite for native Bevy renderer work. Eidetic is a
standalone desktop application, not a browser-first app, so the runtime should
move to a Tauri shell before the Bevy timeline renderer becomes the primary
timeline surface. The desktop migration also removes the Axum HTTP/WebSocket
boundary instead of embedding a loopback server inside the desktop app.

Status: Completed. The active production desktop path is Tauri IPC/events over
backend-owned services; the Axum listener, route adapters, WebSocket server,
browser dev proxy, browser-open launcher flow, and direct workspace-owned WASM
bridge/dependency surfaces have been removed. Final closeout validation passed
`./launcher.sh --test`, `./launcher.sh --build-release`,
`./launcher.sh --release-smoke`, and live-code scans for removed Axum,
WebSocket, HTTP proxy, and WASM bridge surfaces.

Decisions:

- Use Tauri as the desktop application shell.
- Keep Svelte as the desktop UI shell for panels, forms, inspectors, proposal
  review, script editing, accessibility controls, and command alternatives.
- Keep durable state backend-owned. Tauri, Svelte, and Bevy must consume
  projections and emit commands; they must not own canonical project data.
- Replace Axum routes and WebSocket delivery with Tauri command and event
  contracts. The backend runtime remains Rust-owned, but it is invoked through
  the desktop shell rather than through local HTTP.
- Do not use WASM as the Bevy renderer integration path.
- Remove existing wasm-bindgen renderer bridges once native desktop host
  contracts replace them.
- Keep Bevy as an isolated renderer dependency. Bevy must not become a
  dependency of `eidetic-core` or the backend runtime.

Tasks:

- Add a Tauri application scaffold that packages the Svelte frontend and starts
  the Rust backend runtime in a desktop-controlled lifecycle.
- Extract backend application services from the Axum server entrypoint into
  reusable Rust modules owned by the backend runtime: project database
  lifecycle, command validation, projection reads, AI gateway access, event
  publication, and shutdown.
- Replace active frontend HTTP API and WebSocket callers with Tauri command and
  event adapters. Commands must remain command-in/projection-out, versioned, and
  validated at the Rust boundary.
- Remove the Axum server route surface, local listener startup, HTTP CORS
  policy, and browser-open launcher flow once equivalent Tauri command/event
  coverage exists.
- Add launcher/build commands for desktop development, release build, and smoke
  validation.
- Define the native Bevy host boundary for future renderer milestones:
  projection input, command output, lifecycle ownership, queue bounds, error
  handling, and shutdown behavior.
- Remove `wasm-bindgen`, `serde-wasm-bindgen`, and `wasm32` renderer bridge
  modules from `eidetic-bevy-timeline` and `eidetic-bevy-bible-graph` after the
  desktop host boundary exists.
- Update renderer READMEs and planning notes so WASM/browser canvas is recorded
  as rejected for Eidetic's production renderer path.
- Verify Svelte still behaves as a projection-only shell inside Tauri and does
  not reintroduce durable local project state.

Standards compliance constraints:

- Treat Tauri IPC as the only production desktop frontend/backend boundary.
  Raw frontend payloads are untrusted until parsed into validated Rust command
  and projection request types; internal services must not accept unchecked
  `String`, `PathBuf`, numeric range, or mode values when a validated type would
  encode the invariant.
- Keep Tauri as a thin binding/composition layer. Backend application services
  own command handling, projection reads, persistence, AI gateway use, event
  production, and history transactions; those services must compile and test
  without Tauri, Axum, or frontend code.
- Keep package dependency direction explicit: Tauri app crate depends on backend
  runtime/services and shared contracts; backend runtime/services do not depend
  on Tauri; `eidetic-core` does not depend on Tauri, Bevy, Axum, or UI crates.
- Replace HTTP route tests with service-level command/projection tests plus
  Tauri command adapter tests. The adapter tests must prove error mapping,
  payload validation, projection envelope preservation, and no optimistic UI
  state changes.
- Replace WebSocket delivery with Tauri event delivery or a bounded event-drain
  adapter. Event subscriptions must have explicit unsubscribe/teardown behavior,
  bounded buffering or documented overflow policy, and stale-response guards in
  frontend stores.
- Move every spawned backend task behind a lifecycle owner used by the Tauri
  composition root. Startup, shutdown, cancellation, panic logging, and task
  draining must be explicit and idempotent.
- Keep durable state in SQLite and transactional history paths. Tauri, Svelte,
  and Bevy can cache only discardable, versioned projections and transient
  interaction/render state.
- Centralize path, URL, project, asset, import, export, and reference validation
  in backend-owned validators before Tauri commands perform filesystem, shell,
  or external URL work. Tauri permissions/capabilities must be minimal and
  documented.
- Keep the root `launcher.sh` as the canonical entrypoint. Tauri development,
  release, test, and release-smoke paths must satisfy launcher argument
  forwarding, explicit target selection, state isolation, and GUI smoke
  requirements.
- Run a dependency review before adding Tauri or changing Bevy integration.
  Tauri dependencies belong only in the desktop app crate/package, and removed
  Axum, tower-http, wasm-bindgen, serde-wasm-bindgen, and wasm32-only
  dependencies must be deleted when no longer used.
- Update READMEs and contract documentation in every touched `src/` directory:
  backend runtime/services, Tauri command/event adapters, frontend API adapter,
  renderer crates, and launcher behavior.

Blast radius to review during implementation:

- `crates/server/src/main.rs`, `routes/`, `ws.rs`, `static_files.rs`,
  `validation.rs`, `state.rs`, `ydoc.rs`, AI route/runtime modules, export,
  project/reference modules, and route tests.
- `ui/src/lib/api.ts`, `commandApi.ts`, projection and proposal API helpers,
  `ws.ts`, `wsTypes.ts`, websocket-driven stores, route setup, and their tests.
- Workspace manifests, `ui/package.json`, Tauri config, launcher commands,
  release smoke tooling, lockfiles, and generated or shared DTO artifacts.
- `crates/bevy_timeline` and `crates/bevy_bible_graph` wasm exports,
  wasm-only dependencies, bridge tests, and renderer READMEs.

Implementation slices:

1. Inventory active HTTP routes, WebSocket messages, frontend API helpers, and
   route tests; define the Tauri command/event contract map before moving code.
2. Extract backend services behind validated command/projection/event APIs while
   preserving existing route behavior long enough to compare service outputs.
3. Add the Tauri scaffold and prove a minimal vertical slice: health command,
   one read projection, one command-in/projection-out mutation, and one backend
   event refresh without starting an Axum listener.
4. Replace frontend `fetch` and WebSocket consumers with a single Tauri adapter
   layer and update frontend tests around adapter mocks rather than HTTP URLs.
5. Delete Axum routes, WebSocket server, static-file server, CORS/local-origin
   policy, browser-open runtime path, and now-obsolete route tests after Tauri
   command/event coverage is equivalent.
6. Update launcher, release smoke, dependency manifests, README contract docs,
   and dependency audit notes.
7. Remove WASM renderer bridges and wasm-only dependencies after the native
   desktop host boundary exists.

Re-plan triggers:

- Any backend service starts depending on Tauri, Svelte, Bevy, Axum, or UI test
  utilities.
- A Tauri command directly mutates SQLite, project state, Y.Doc state, or
  proposal state instead of calling the backend service command path.
- Frontend stores begin patching durable state locally or showing backend-owned
  changes before a returned projection/event confirms them.
- Event delivery needs polling that is not isolated to a boundary adapter with
  bounded buffers, deterministic cleanup, and a documented overflow policy.
- Tauri lifecycle cannot shut down spawned tasks deterministically or cannot
  report startup failures without panics.
- Removing Axum exposes a missing service contract that cannot be covered by a
  focused command/projection API without broad redesign.

Verification:

- Desktop dev command opens the Tauri app and reaches a backend-owned health
  command without starting an Axum listener.
- Release/smoke command proves the packaged desktop runtime starts, loads the
  Svelte shell, and invokes backend commands through Tauri.
- Backend command/projection tests still pass after moving runtime invocation
  from Axum routes to Tauri commands/events.
- Tauri command/event adapter tests cover payload validation, serde round trips,
  error mapping, event subscription teardown, stale-response handling, and
  projection envelope preservation.
- Frontend typecheck, lint, and projection-only guard tests pass.
- Renderer crates compile without wasm target dependencies after bridge
  removal.
- Lifecycle tests or smoke checks cover app startup, backend shutdown, and
  failure reporting when backend runtime initialization fails.
- Dependency review confirms Tauri is leaf-only, removed Axum/WASM dependencies
  are not retained unused, and new heavy dependencies are justified or
  feature-gated.
- No production desktop path depends on Axum, local HTTP, browser-open startup,
  wasm-bindgen, or wasm32 renderer bridge modules.

## Milestone 8: Bible Graph And Context Influence View

Current implementation status:

- Completed foundation: `BibleRenderGraph`, context stack, context evaluation,
  and context influence DTOs exist as backend-owned contracts with TypeScript
  mirror types and test coverage.
- Completed foundation: context evaluations and influence records have
  relational SQLite storage, command/event/revision history, idempotent command
  replay, desktop commands, and projection routes.
- Completed foundation: bounded bible render graph requests cover focused root,
  selected graph node, selected timeline node, search, neighborhood depth,
  `max_nodes`, and `max_edges`; the SQLite query path avoids loading and
  positioning the whole graph for normal interactive projections.
- Completed foundation: selected timeline nodes can load the active context
  stack and seed graph highlights from persisted context influence records.
- Completed foundation: graph workspace, split workspace, Svelte outline,
  side-list navigation, right inspector details, and transient typed graph
  selection are in place as projection-only UI surfaces.
- Completed foundation: the Bevy graph leaf crate is on Bevy 0.18.1, remains
  isolated from `eidetic-core` and `eidetic-server`, builds projection-derived
  scene/native visual entities, enforces the prototype full-rebuild envelope,
  and exposes validated renderer command output.
- Completed foundation: desktop graph renderer ownership is isolated behind a
  bounded owner thread, a native runner handle, a supervisor state machine,
  projection refresh coalescing, and typed renderer-window status.

Current open blockers:

- The projection, context, and floating renderer ownership foundation is in
  place, but the actual product graph experience is not complete. The current
  floating Bevy window is a 2D/sprite-based native renderer proof with basic
  labels, colors, selection, panning, lifecycle, and projection handoff. It is
  not yet the target 3D graph.
- A dedicated 3D bible graph experience milestone now follows this milestone
  before agent harness work. Agent tooling must not depend on the graph being
  "done" until that milestone provides a usable 3D visual surface.

Standards compliance review:

- Reviewed standards: `CODING-STANDARDS.md`, `ARCHITECTURE-PATTERNS.md`,
  `FRONTEND-STANDARDS.md`, `CONCURRENCY-STANDARDS.md`,
  `CROSS-PLATFORM-STANDARDS.md`, `INTEROP-STANDARDS.md`,
  `DEPENDENCY-STANDARDS.md`, `PLAN-STANDARDS.md`, `TESTING-STANDARDS.md`,
  `ACCESSIBILITY-STANDARDS.md`, and Rust async/cross-platform/unsafe/
  dependency standards.
- The current milestone direction is compliant only if the remaining work keeps
  backend-owned durable state, typed projection contracts, one renderer
  lifecycle owner, no optimistic frontend writes, no fallback renderer path, and
  no scattered platform behavior.
- The implementation must treat renderer runner work as an app/composition-root
  concern, not a domain/core concern. Bevy, Tauri, winit, platform handles, and
  event-loop ownership stay out of `eidetic-core` and `eidetic-server`.
- The implementation must use message passing or one clearly-owned state
  machine for related renderer request, refresh, lifecycle, and shutdown state.
  Split mutable state may exist only when the fields are independent and tests
  prove stale writes cannot replace newer projections.
- Any parallel worker wave for this milestone requires an explicit worker-wave
  plan first. Shared contracts, status enums, generated schemas, lockfiles, and
  renderer lifecycle integration are serial ownership points.

Remaining tasks:

- Keep Tauri/Rust/TypeScript renderer contracts updated together. Any status,
  command, event, or projection wire-shape change must include strict boundary
  validation and Rust/TypeScript tests in the same slice.
- Keep the renderer runner sync-core/async-shell compliant: pure planning,
  status mapping, validation, and projection adaptation stay synchronous; async
  is limited to Tauri/runtime boundaries and unavoidable I/O.
- Isolate platform behavior behind platform strategy modules. Linux, Windows,
  macOS, and unsupported implementations must compile through a shared
  platform-neutral API, with `cfg()` kept in thin platform/factory modules
  rather than scattered through renderer/domain logic.
- Add a native window runner gate before replacing any Svelte graph surface.
  The gate must prove that Bevy 0.18.1 can open, focus, close, reopen, and
  tear down a real floating native window under the Tauri desktop runtime using
  the smallest clear-color/grid scene. The gate is per platform: Linux
  worker-thread strategy is now locally proven for the current Linux desktop
  environment by `eidetic-desktop --graph-renderer-smoke`, so it may report
  verified support on Linux. Windows must remain unproven until the same
  lifecycle gate passes there, and macOS requires a separate
  main-thread/Tauri-runtime-compatible strategy before it can report support.
  Until a platform-specific gate passes, `renderer_window_visible_supported`
  remains false for that platform and the backend projects typed unavailable
  status. The Svelte outline remains only a keyboard-accessible semantic view,
  not a fallback renderer path.
- Add an explicit native runner proof checklist before graph visuals are wired
  to the visible window. The checklist must cover minimal Bevy/winit window
  creation, status heartbeat, focus request behavior, close, reopen, project/
  app shutdown teardown, panic/error projection, command responsiveness while
  the event loop is running, and no fake visible-window status.
- Replace the current pending supervisor implementation with one production
  runner path per verified platform. The supervisor remains the single
  lifecycle owner for native renderer startup, worker/main-thread placement,
  command channel, status channel, shutdown/join handle, panic/error
  projection, and verified capability state.
- Add explicit platform strategy modules for the native runner gate. The
  platform-neutral host owns the public command/status API, while platform
  strategies own only event-loop placement and capability proof. A strategy
  that cannot be verified must return typed unsupported status instead of
  faking visible-window support.
- Add a shared floating Bevy renderer host for native visual windows. The host
  must support renderer kind (`graph` now, `timeline` later), open/close,
  focus/status, optional size/placement hints, projection subscription,
  command delivery, and deterministic teardown. It must not embed child
  surfaces into the WebView or make the renderer runtime the owner of business
  logic.
- Keep the native Bevy bible graph host as a projection consumer. It receives
  `BibleRenderGraph`/influence projections and emits validated selection,
  focus, inspect, and navigation commands only.
- Consolidate graph renderer projection delivery into one desktop-owned
  subscription path. Projection-route mirroring, server-event refreshes, and
  Svelte projection-set commands must not remain parallel ways to mutate the
  renderer's active projection.
- Deliver renderer commands through the desktop event bridge or native renderer
  event path before the Bevy graph becomes the primary surface.
- Wire graph projection rendering into the visible floating Bevy window only
  after the native runner gate passes for the target platform.
- Keep Svelte filters, search, detail panels, review panels, and
  keyboard-accessible alternatives around the same backend projections and
  commands. The Svelte outline remains an accessibility/inspection surface, not
  the primary visual graph and not a fallback renderer.
- Remove or demote the old Svelte primary graph surface only after the Bevy
  graph covers selection, inspection, filtering, navigation, focus, close, and
  reopen under backend-owned status.
- Keep full Bevy ECS/native-visual rebuilds only while graph projections are
  strictly bounded and refreshes are coalesced. If graph caps, refresh frequency,
  force-layout behavior, or renderer interaction requirements exceed that
  prototype envelope, add keyed entity diffing before making the Bevy graph the
  primary visual surface.
- Update affected `README.md` files or add an ADR when ownership boundaries,
  platform strategy modules, native runner lifecycle, dependencies, or
  verification policy change. New source directories must include a README.

Context model:

- The bible graph owns world, story, character, relationship, place, object,
  culture, faction, rule, theme, and production knowledge.
- Timeline clips own scoped context for a specific layer of the story
  hierarchy. A clip may reference graph nodes, distill inherited context, and
  create proposals for new graph knowledge, but durable graph state remains
  backend-owned.
- Context evaluations record what graph knowledge was considered for a task,
  which nodes/edges/paths were selected, which parent contexts were inherited,
  which distilled context was produced, and which downstream outputs were
  influenced.
- The graph view visualizes both the durable bible and the active reasoning
  path for the selected playhead/clip. Strong highlights represent directly
  used context, softer highlights represent inherited context, and dimmed nodes
  represent graph knowledge outside the current context window.
- The UI surface is split by responsibility: the app shell owns layout and
  floating window controls, Svelte owns graph controls/details/review and
  keyboard command alternatives, and Bevy owns floating renderer windows for
  realtime visuals. The graph window lands first; the timeline window reuses
  the same host later.
- Bevy may own transient camera, hover, simulation, animation, and unsaved
  layout state. Durable graph facts, influence records, saved layout decisions,
  and accepted proposals must enter through backend commands and projections.
- The milestone is visualization- and projection-focused. LLM workflow tooling
  consumes these graph/context projections in the next milestone instead of
  being built into the renderer or Svelte graph surface.
- Graph render projections are bounded read models, not a full-project mirror.
  Full-graph exports or diagnostics can exist separately, but interactive graph
  surfaces must request a bounded context by focus, playhead/clip, search, or
  root.
- UI selection is transient and typed by selection kind. A selected edge,
  influence path, or context layer must not be squeezed through
  `selectedGraphNodeId` or broad durable frontend objects.

Implementation order:

Completed foundation, do not reimplement unless verification fails:

- Backend `BibleRenderGraph`, context stack, context evaluation, and context
  influence contracts.
- Relational SQLite current-state/history storage for context evaluations and
  influence records.
- Bounded SQLite graph/context query service, deterministic projection adapter,
  and selected-clip influence highlighting.
- Hierarchical context refinement projection support that prefers recorded
  distilled parent context before falling back to timeline recap text.
- Svelte workspace placement, split view, side lists, right inspector, graph
  controls, and keyboard-accessible semantic outline.
- Bevy graph leaf host and desktop owner boundary for projection consumption,
  visual snapshots, bounded command queues, panic-to-error conversion, and
  prototype full-rebuild limits.
- Renderer-window status capability normalization: pending, platform-unproven,
  platform-unsupported, runner-error, and verified-support are explicit Rust/
  TypeScript states, with reason strings treated as detail rather than the
  primary source of support state.
- Renderer startup failure projection: native runner and renderer-owner startup
  failures produce typed runner-error status instead of desktop startup panics.
- Renderer projection request and refresh coalescing state share one
  lock-protected desktop owner value.
- Diagnostic native renderer smoke preflight can emit a report-only JSON record
  without opening a window. This remains diagnostic-only and does not verify
  production visible-window support.
- Native renderer platform checks are isolated in the desktop platform strategy
  module tree with local invariants documented.
- The Bevy leaf renderer exposes a close-control handle for future supervisor
  shutdown without owning desktop or durable backend state.
- Local Linux diagnostic Bevy/winit smoke proof succeeded with
  `--any-thread --auto-close-ms 500`; production support remains unverified
  until the Tauri-owned supervisor path proves the same lifecycle.
- Desktop native renderer window thread ownership now has a bounded close/stop
  primitive ready for supervisor integration.
- The production native renderer supervisor path now starts the Bevy window
  thread, reports running/closed/failed status, refreshes completion/panic
  state on status/focus, and performs bounded joined close.
- Native renderer `window_ready` is now based on a Bevy renderer-produced ready
  signal observed by the desktop window thread, not on supervisor-thread
  liveness.
- Renderer close/reopen now keeps the Bevy/winit event loop alive and performs
  hide/show transitions on the existing native window. Local Linux
  `eidetic-desktop --graph-renderer-smoke` passes open/status/focus/close/
  reopen/project-close/app-shutdown with ready open/reopen snapshots.
- Linux native graph renderer support is now marked as verified after local
  Tauri-owned lifecycle proof. Windows and macOS remain typed unproven
  strategies until they have matching platform-specific runtime proof.
- The floating native graph window has a projection handoff from the desktop
  owner into Bevy window control state. The native app consumes the latest
  `BibleRenderGraph` projection and rebuilds native visual ECS entities.
- The floating native graph window now uses Bevy's render stack and attaches
  sprite-backed node/edge primitives to native graph visual ECS entities. The
  lifecycle smoke readiness budget was raised to account for render-stack
  startup while still requiring an actually ready native window.
- Renderer command delivery now flows through the desktop event bridge. The
  old Svelte command-drain interval and direct drain command API were removed,
  so graph renderer selection/inspect commands are no longer polled by the
  frontend.
- Renderer projection delivery now has one managed desktop projection owner.
  Graph renderer open, request-update, and backend-event refresh paths call the
  same owner for active request state, coalesced projection loading, and
  renderer writes.
- Native Bevy graph windows now have a bounded command path back to the desktop
  renderer owner. Validated native producers cover node selection, node
  inspection, edge selection, and influence selection from native visual
  entities and feed the same desktop event bridge used by other graph renderer
  commands.
- Native graph focus and navigation command contracts now exist end-to-end.
  The native Bevy graph validates node-backed focus/navigation intents, the
  desktop event bridge serializes them as typed renderer commands, and Svelte
  applies them only to transient projection-selection surfaces.
- The graph workspace no longer renders the Svelte outline as the always-on
  primary graph surface. The Bevy renderer controls occupy the main graph
  workspace, and the Svelte outline is available only as an explicit secondary
  projection inspector.
- Native Bevy graph visual refreshes now use keyed node/edge/influence upserts
  by stable graph IDs and remove stale visual entities after each projection
  refresh. The old native full-despawn visual rebuild path is no longer the
  default for bounded graph refreshes.
- Milestone 8 standards/blast-radius review found stale UI copy and test
  wording that still described the Svelte outline as a renderer fallback. That
  wording was removed so the frontend presents the Bevy renderer controls as
  the graph surface while keeping the outline as an explicit secondary
  projection inspector.
- Follow-up renderer usability review found the native graph window still used
  a borderless prototype configuration, category node colors fell through to
  placeholder pale sprites, labels were not rendered, and native selection/
  navigation command contracts were not connected to window input. The native
  runner now opens as a decorated OS window, graph colors and labels render from
  backend projections, left-click selection emits backend-owned renderer
  commands, and WASD/arrow-key panning moves the Bevy graph camera.

Remaining implementation order:

1. Keep Svelte graph filters, details, review, and semantic outline
   projection-only as secondary controls/accessibility surfaces.
2. Treat Milestone 8 as implementation-complete only for backend-owned graph
   projection delivery, context influence projection, and native floating
   renderer ownership.
3. Move product graph work into Milestone 9: true 3D scene, 3D layout,
   navigation, edge readability, labels, selection highlighting, and usable
   graph editing/inspection workflows.

Standards gates:

- `BibleRenderGraph`, context-stack, context-evaluation, and influence DTOs are
  executable boundary contracts with explicit serde shape, typed IDs/enums,
  validation, TypeScript mirror types, and round-trip tests before Bevy or
  Svelte consumes them.
- The bounded graph/context query service lives behind a backend service
  boundary and owns SQL/index strategy. Core projection adapters remain pure
  and synchronous; Bevy/Svelte never query SQLite or derive canonical context.
- Context influence writes are transactional, idempotent, history-backed, and
  replayable. Failed influence writes must leave graph and timeline projections
  unchanged.
- Interactive graph projections must enforce depth, count, search, and
  neighborhood limits at the backend boundary. Renderer or UI-supplied limits
  are validated before any graph traversal or allocation.
- The AppShell/layout decomposition is a prerequisite, not a cleanup task.
  Workspace mode, right inspector, shortcut ownership, timeline sizing, and
  renderer lifecycle must be separate modules under component/file thresholds.
- The Bevy graph host has one desktop lifecycle owner, bounded command queues,
  subscription teardown, panic/cancellation reporting, and no detached tasks.
- The floating renderer host has one lifecycle owner per renderer window. It
  validates size/placement hints before use, has deterministic open/focus/close
  and teardown behavior, and keeps renderer-local state separate from backend
  projections and Svelte stores.
- The native runner boundary must not run a blocking Bevy/winit event loop
  inside the synchronous Tauri request/reply handler. Tauri command paths must
  remain responsive while a renderer window is open, and runner status must be
  observable without waiting for the event loop to stop.
- Runtime creation and long-lived renderer event loops must live in the desktop
  composition root. Library/core crates may expose synchronous planning,
  validation, projection adaptation, and renderer-local APIs, but they must not
  create global runtimes or spawn unmanaged tasks.
- Every spawned thread/task, event subscription, timer, command queue, and
  renderer subscription must have a lifecycle owner, bounded capacity where
  applicable, deterministic teardown, and panic/error projection at the owner.
- Related renderer state transitions must be atomic at their owner boundary.
  Do not update request identity, refresh state, visible support, lifecycle, or
  shutdown state through unrelated mutable paths when correctness depends on
  their combined value.
- The native window runner gate is blocking for visual graph replacement. It
  must not set `renderer_window_visible_supported`, `renderer_window_visible`,
  `renderer_window_ready`, or focus support from inferred or hardcoded state;
  those fields must come from the actual runner capability/status path.
- The minimal native runner proof must be isolated from graph projection
  rendering. A clear-color/grid window can be verified first, but it may not
  consume durable graph data, own graph selection, or mark the Bevy graph as
  primary until the runner lifecycle is proven.
- A platform runner may report `verified_support` only after an executable or
  documented smoke proof covers open, status, focus, close, reopen, teardown,
  panic/error projection, and command responsiveness for that platform. A proof
  on one OS/windowing backend must not change capability status for another.
- Renderer-window status must include an explicit unsupported/capability reason
  enum owned by the backend status contract. UI code may display the reason, but
  must not infer pending, unsupported, failed, or supported states from free-form
  messages, platform names, or combinations of booleans.
- Tauri command payloads, renderer event payloads, renderer status DTOs, and
  Rust/TypeScript mirror types are executable interop contracts. Unknown fields,
  malformed IDs, contradictory states, unbounded limits, and renderer-shaped
  durable mutations must be rejected at the boundary.
- Native window support is a per-platform capability, not a global milestone
  flag. Linux X11/Wayland, Windows, and macOS each need their own verified
  runner outcome. The app may enable the visible Bevy graph on platforms that
  pass while unsupported platforms continue to project typed unsupported status
  and keep the Svelte outline as a keyboard-accessible semantic view, not as a
  fallback renderer path.
- No fallback or legacy renderer path may be added to resolve native runner
  risk. The permitted unavailable state is typed backend status. If a platform
  cannot run the production supervisor, it remains unavailable until a verified
  platform strategy is implemented or the renderer architecture is replanned.
- Renderer projection delivery has exactly one active writer owned by the
  desktop renderer host. Frontend stores may cache backend responses and request
  focus/filter changes, but they must not independently push renderer
  projections in parallel with backend event refresh or projection-route
  mirroring.
- Renderer projection refreshes are coalesced by the desktop renderer host or a
  dedicated desktop-owned subscription owner. Event bridges and Svelte controls
  may enqueue invalidation/request updates only; they must not start independent
  projection loading loops for the same renderer.
- Desktop renderer bridges, backend event bridges, and command-delivery loops must
  have explicit lifecycle owners, tracked handles or unsubscribe paths, bounded
  queues, and deterministic teardown. Detached task patterns are not acceptable
  for native renderer integration.
- The floating renderer host must not be introduced as a separate frontend
  application layer. It consumes backend projections, emits backend command
  intents, and owns only renderer-local resources. Any future split-process
  renderer option requires a new ADR, standards pass, and explicit proof that
  backend ownership is unchanged.
- If native window integration requires raw OS handles, `unsafe`, or FFI, that
  work must be isolated in a thin desktop-owned module with a safe API,
  feature-gated when possible, documented with safety invariants, and covered by
  a separate verification plan before it can replace the current typed
  unavailable path.
- Renderer window support must be cross-platform by construction: platform
  behavior is isolated behind desktop runtime strategy modules, unsupported
  capabilities degrade through typed status projections, and implementation
  slices must not mark Linux X11-specific behavior as the general solution.
- Platform implementations must be separated by platform module/file where the
  behavior is more than a tiny documented `cfg()` exception. UI, domain, server,
  and core projection code must call platform-neutral renderer strategy APIs.
- The Bevy/winit `run_on_any_thread` path must be treated as a Linux/Windows
  candidate only. macOS must use a separately proven main-thread or
  Tauri-compatible strategy before reporting visible-window support. If that
  strategy is not available, macOS remains unsupported in backend status
  without blocking other proven platforms.
- Svelte graph controls own only drafts, filters, focus, local expansion, and
  projection caches. Selection that changes generation or persistence must go
  through backend commands/projections.
- Bevy 0.18.1 remains a leaf dependency. The upgrade includes `cargo tree`
  cost review, feature selection notes, and proof that `eidetic-core` and
  server query logic do not depend on Bevy.
- Any new Bevy render/window/input features or Tauri window/runtime dependencies
  must be justified in the dependency review for the owning leaf crate, with
  `cargo tree` notes and proof that `eidetic-core` and `eidetic-server` remain
  free of Bevy/Tauri dependencies.
- New dependencies must be declared by the narrowest owning crate/package, not
  the workspace root unless genuinely shared. Feature selection must stay
  minimal, heavy dependencies must be feature-gated or justified, and lockfile
  changes belong in the same atomic slice as the dependency review.
- Platform-specific Bevy/winit features are reviewed per runner strategy. Linux
  X11/Wayland feature enablement, Windows support, and macOS support must be
  described separately so dependency configuration does not silently encode a
  Linux-only implementation as the default cross-platform path.
- Raw-window-handle or platform-handle dependencies are absent by default. If
  the native runner gate proves one is required, the dependency must live only
  in `eidetic-desktop`, be used by the module that owns the platform handle,
  and be covered by the raw-handle safety/verification gate above.
- Full despawn/respawn Bevy graph rebuilds are allowed only for small bounded
  projections during the prototype runner gate. Primary graph rendering requires
  either documented caps plus coalesced refresh proof or keyed ECS/native-visual
  diffing that updates changed nodes, edges, and influence highlights without
  rebuilding the whole scene every refresh.
- Implementation slices must keep source files and UI components below the
  decomposition review thresholds where practical. If a touched file exceeds
  500 lines, a touched UI component exceeds 250 lines, or a module/service grows
  beyond one clear responsibility, split it or record why the shape is still
  safe.
- Before implementation resumes, git status must be clean for implementation
  files or the dirty files must be explicitly allowed. Plan/documentation edits
  may remain dirty only while plan setup is in progress.

Verification:

- Vertical slice: select a scene clip -> backend returns active context stack ->
  graph projection highlights directly used scene nodes, inherited premise/act/
  sequence nodes, and the edges explaining those paths.
- UI placement tests or smoke checks prove graph workspace, split view, sidebar
  controls, right-panel details, and bottom timeline selection can coexist
  without creating a second durable graph owner.
- Floating renderer smoke checks prove the graph window opens and closes under
  desktop lifecycle control, routes pointer/focus input to Bevy only while the
  renderer window owns focus, and keeps all durable state changes behind
  backend command/projection confirmation.
- Native window runner gate checks prove the minimal Bevy window opens, focuses,
  closes, reopens, and tears down under Tauri without blocking the Svelte shell,
  leaking renderer threads/tasks, or setting visible-window status from fake
  state. The smoke check must run before graph nodes/edges become the primary
  visual surface.
- Native runner proof verification records the exact platform/backend tested,
  the command used to run the smoke proof, the observed lifecycle transitions,
  and the unsupported/pending status for every platform not proven in that
  slice. The proof must show that status/focus/close commands return while the
  Bevy/winit event loop is active.
- Native runner verification records separate outcomes for Linux X11, Linux
  Wayland, Windows, and macOS. Linux/Windows worker-thread proofs do not mark
  macOS supported; macOS support requires a separate main-thread/Tauri-runtime
  proof or remains typed unsupported.
- Status contract tests prove pending, platform-unproven, platform-unsupported,
  runner-error, and verified-support cases serialize through Rust and TypeScript
  without relying on free-form messages or frontend inference.
- Status contract tests also prove contradictory capability/reason/lifecycle
  combinations cannot be represented or are rejected before crossing the Tauri
  boundary.
- Runner responsiveness checks prove Tauri commands can query status, request
  focus, update the active projection request, and close the renderer while the
  Bevy/winit event loop is running.
- Projection-delivery tests prove a graph refresh event updates an open
  renderer through one desktop-owned subscription path, without duplicate
  projection writes from route mirroring, server-event refresh, and Svelte
  projection-set commands.
- Projection coalescing tests prove backend event bursts and rapid Svelte
  focus/filter/search changes produce one in-flight projection load plus a
  tracked follow-up refresh, not overlapping renderer writes.
- Renderer bridge lifecycle tests prove backend event bridges, renderer
  projection subscriptions, command queues, and command-delivery bridges are
  cancelled or unsubscribed on renderer close, project close, and app shutdown.
- Dependency cleanup checks prove stale embedded-viewport documentation is gone
  and raw-window-handle/platform-handle dependencies are either absent or
  justified by the native runner safety plan.
- Embedded-viewport retirement checks prove no production graph path depends on
  WebView child-surface attachment, X11 parent window IDs, or panel-bound
  resize contracts. Any remaining experimental code must be feature-gated,
  documented as non-production, and unreachable from launcher/default builds.
- Platform strategy tests or compile checks prove renderer window lifecycle is
  selected through a single desktop platform boundary and reports unsupported
  capabilities through typed status instead of panicking or falling back to
  domain/UI branching.
- Cross-platform checks compile the desktop runner strategy for the required
  Rust targets where local tooling permits; unsupported or untested platform
  behavior must be represented as typed capability/status degradation, not as a
  silently selected Linux-only path.
- Rust async/lifecycle tests prove spawned runner tasks/threads, event bridges,
  subscriptions, command queues, and shutdown paths have tracked owners, bounded
  stop behavior, joined teardown, and panic/error reporting.
- Interop tests prove renderer commands/status/events round-trip through Rust
  serde and TypeScript consumer types using the actual Tauri command/event
  shapes.
- Projection bounding tests prove large bible graphs return bounded
  neighborhoods by selected clip, focused root, filter, or search result.
- Query-service tests prove graph/context reads do not use full graph scans for
  selected-clip, focused-root, selected-node, or search/filter projections.
- Layout helper unit tests prove deterministic derived positions/indexes for
  stable inputs.
- Selection/neighborhood index tests prove selected-node, parent-context, and
  influence-path lookups do not require component-local graph scans.
- History/reload tests prove accepted graph proposals and context influence
  records rebuild the same projections after project reload.
- App shell decomposition tests/smoke checks prove workspace mode, right
  inspector, shortcuts, export/save controls, and timeline sizing remain owned
  by focused modules and stay under component decomposition thresholds.
- Renderer lifecycle tests prove floating renderer subscriptions, commands, and
  transient renderer state are torn down on window close/project close.
- Dependency review proves Bevy 0.18.1 remains isolated to leaf renderer crates
  and that any render/window/input features are justified before they land.
- Dependency review or compile checks prove platform-specific Bevy/winit feature
  choices are intentional per runner strategy and do not make untested Linux
  X11/Wayland assumptions the default for Windows or macOS.
- Dependency ownership checks prove new Rust crates or npm packages are declared
  at the crate/package that executes them, and `cargo tree`/package tree notes
  are recorded for any new or expanded native runner dependency surface.
- Renderer performance tests or smoke metrics prove the current full-rebuild
  strategy stays within the documented bounded prototype envelope, or that keyed
  diffing has replaced it before the Bevy graph becomes primary.
- Frontend tests prove graph filters/details/review controls replace projection
  caches from backend responses and do not mutate durable graph state locally.
- Accessibility tests or smoke checks prove the Svelte semantic outline and
  graph controls remain keyboard reachable, have accessible names, preserve
  focus indicators, and do not conflict with Bevy pointer/focus ownership.
- Documentation checks prove touched source directories have current README
  ownership/lifecycle notes and that platform/unsafe/dependency decisions link
  to an ADR or the milestone plan section that owns the decision.
- AI context projection tests for later graph-driven generation must prove
  prompting uses bounded graph/context queries instead of loading every
  non-system bible node and every node detail projection.

Re-plan triggers:

- Bevy/winit cannot open, focus, close, or tear down a native window safely
  under the Tauri runtime.
- The only workable Bevy/winit runner design would block Tauri command replies
  or require waiting for the renderer event loop to exit before status/focus/
  close/projection commands can complete.
- Any platform requires untestable or inline OS-specific behavior to make the
  runner work.
- The runner requires raw OS handles, `unsafe`, FFI, or split-process behavior
  not already covered by an ADR, safety plan, and standards pass.
- A proposed runner path would make Bevy, Svelte, or renderer-local state own
  durable graph facts, context influence records, accepted proposals, or saved
  layout decisions.
- Renderer status cannot represent unsupported, pending, failed, and supported
  runner states through typed backend-owned fields without UI inference.
- Renderer projection delivery cannot be reduced to one desktop-owned writer
  without reintroducing frontend-owned durable state or unbounded event fanout.
- Renderer projection invalidation cannot be coalesced without losing ordering,
  dropping the latest active request, or allowing stale projection writes to
  replace newer renderer state.
- Full Bevy ECS/native-visual rebuilds cannot meet the bounded prototype
  performance envelope and keyed diffing would require changing renderer or
  projection ownership boundaries.
- The native runner requires broad dependency changes outside leaf renderer or
  desktop crates, or makes `eidetic-core`/`eidetic-server` depend on Bevy,
  Tauri, or platform windowing crates.
- Bevy/winit feature selection cannot be cleanly separated by platform strategy,
  or the visible runner requires platform dependencies that cannot be locally
  verified and typed as unsupported on unproven platforms.
- Verification cannot prove deterministic startup, focus, close, reopen,
  project-close teardown, panic reporting, and no fake visible-window status.

Exit criteria:

- Backend-owned bible render graph, context stack, context evaluation, and
  influence projections exist and are bounded, reloadable, and validated.
- The floating Bevy graph renderer host is owned by the desktop composition
  root, has typed lifecycle/status, bounded command delivery, deterministic
  teardown, and projection refresh coalescing.
- Svelte graph controls/details/outline remain projection-only secondary
  surfaces and do not own durable graph state.
- The old 2D SVG relationship graph is removed or no longer a supported graph
  view.
- The remaining user-facing 3D graph requirements are explicitly carried by
  Milestone 9 instead of being hidden as follow-up polish.

## Milestone 9: Native 3D Bible Graph Experience

Purpose:

- Turn the renderer foundation from Milestone 8 into a usable 3D spatial graph
  for the story bible before agent tooling depends on graph interaction.
- Use `/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/whip-docs/`
  as a visual and interaction reference, especially `src/lib/graph-v0/`, while
  implementing the Eidetic graph in Bevy with Eidetic colors and backend-owned
  projection contracts.
- Keep the graph as an inspector/editor surface for backend-owned bible data.
  Bevy owns transient camera, hover, animation, selection preview, and local
  layout simulation only.

Tasks:

- Replace the current 2D/sprite renderer proof with a true 3D Bevy graph scene:
  `Camera3d`, 3D node meshes, depth-aware positions, 3D edge geometry, lighting,
  and bounds-aware framing.
- Render structural edges from backend projections so canonical roots and
  parent/child relationships are visible even before the user adds semantic
  bible relationships.
- Render explicit bible graph edges with distinct styles from structural edges:
  semantic relationship edges, timeline/context edges, influence edges, and
  proposal edges must be visually distinguishable.
- Add bounded label behavior modeled after the whip-docs graph: labels for
  selected/focused nodes, nearby nodes, canonical roots, and search matches;
  avoid labeling every distant node by default.
- Add selection highlighting: selected node, incident edges, adjacent nodes,
  second-level neighborhood labels, graph-distance dimming, and selected edge/
  influence highlighting.
- Add usable camera/navigation controls: orbit, pan, zoom, frame selection,
  focus selected neighborhood, clear focus, and keyboard graph navigation.
- Completed: backend-owned camera command intents for reset, fit, frame node,
  frame edge, frame influence, node navigation, and neighborhood navigation now
  route through the Bevy graph app, native runner, floating window control,
  Tauri command boundary, and TypeScript API. These are transient renderer
  presentation commands, not Svelte-owned durable graph facts.
- Add tested hit testing for nodes and edges using Bevy ray picking or an
  equivalent renderer-local selection index. Selection output must still become
  typed backend/desktop renderer commands.
- Keep Svelte as the durable editor surface for add/edit/remove node fields,
  create/remove edges, inspect details, review proposals, and keyboard
  alternatives. Direct Bevy editing can be added only as backend commands that
  wait for confirmed projections.
- Completed: Graph workspace node creation uses the shared backend-owned
  category create flow from the Bible tab. It validates schema availability,
  ensures canonical roots, sends a backend create command, refreshes the render
  projection, and selects the confirmed node projection without optimistic
  graph mutation.
- Add graph workspace controls for view mode, category filter, edge-kind
  filter, search, selected-clip/playhead context, frame selected, and focus
  neighborhood. These controls may own local drafts only.
- Add active-context and playhead/clip highlighting so selecting a timeline
  clip can show which bible nodes, edges, parent contexts, and distilled
  context layers influence that point.
- Add empty-bible usability: canonical scaffold/category roots are visible,
  adding the first character/location/object is obvious from Svelte controls,
  and the graph never appears as disconnected unexplained squares.
- Port the reference architecture conceptually into Rust/Bevy: pure layout
  helpers, selection index, neighborhood/highlight derivation, visibility
  filtering, camera framing helpers, and renderer systems separated from graph
  domain services.
- Preserve the shared floating renderer host from Milestone 8. Do not add a
  second renderer lifecycle, local HTTP/WebSocket transport, WASM bridge,
  WebView child-surface path, split-process renderer sidecar, or frontend-owned
  projection writer.

Implementation order:

- Define the render-facing visual model for 3D nodes, structural edges,
  semantic edges, labels, selected/highlighted/dimmed states, and camera
  framing before changing Bevy systems.
- Add pure Rust layout and selection-index helpers adapted from the whip-docs
  concepts, with tests independent of Bevy rendering.
- Extend backend `BibleRenderGraph` projections only if the current contract
  lacks renderer-neutral data required for structural edges, edge classes,
  label priority, or active context highlights. Keep all such fields bounded
  and validated.
- Replace the native 2D graph scene with the smallest true 3D slice: canonical
  roots, one user node, one structural edge, one semantic edge, labels for
  focused nodes, orbit/pan/zoom, and node selection.
- Add neighborhood highlighting, edge selection, framing, focus mode, and
  search/filter-driven projection refresh after the basic 3D slice is stable.
- Add playhead/selected-clip influence highlighting after graph selection and
  labels are usable.
- Keep Svelte detail/edit/review controls available throughout. Bevy visual
  actions must have semantic Svelte command alternatives for accessibility.
- Update `crates/bevy_bible_graph/README.md`, layout/sidebar READMEs, and this
  plan whenever ownership, view modes, renderer commands, or dependency
  features change.

Standards gates:

- Bevy graph code remains a leaf renderer. `eidetic-core` and `eidetic-server`
  must not depend on Bevy, Tauri runtime types, renderer ECS components, or
  platform windowing crates.
- Durable graph facts, edges, fields, snapshots, proposals, context
  evaluations, influence records, saved layout decisions, and history remain
  backend-owned SQLite state changed only through idempotent backend commands.
- Renderer-local camera, hover, animation, temporary layout simulation,
  selection preview, and focus mode are disposable and rebuildable from
  backend projections.
- Graph projections remain bounded by focus, root, search, selected timeline
  node, playhead context, neighborhood depth, `max_nodes`, and `max_edges`.
  The renderer must reject or refuse unbounded graph payloads.
- Completed: graph projection requests can now carry backend-owned edge-kind
  filters. Core and SQLite filtering both apply the filter before edge limits
  so matching edges are not dropped because an earlier nonmatching edge filled
  the bounded result window.
- Completed: Graph workspace edge-kind controls now submit those filters as
  projection request inputs; the frontend owns only the transient control
  selection and does not filter or mutate durable graph facts locally.
- Structural edges shown for parent/child hierarchy must be projection facts or
  deterministic projection derivatives, not Bevy-invented durable graph state.
- Selection/highlight changes restyle existing Bevy entities where practical.
  Full rebuilds are reserved for graph/projection/layout changes and must stay
  within documented bounded caps unless keyed diffing covers the path.
- Completed: native graph rebuilds now keep renderer-local mesh/material caches
  for repeated node radii, edge dimensions, and visual state materials, so
  frequent bounded projection refreshes reuse assets instead of allocating new
  handles for identical visuals.
- Renderer commands for node selection, edge selection, focus, inspect,
  navigation, and any future Bevy-initiated edits use typed IDs and strict
  validation at the desktop/backend boundary.
- Svelte may own draft form fields, filter inputs, selected transient view
  controls, and projection caches. It must not optimistically mutate persisted
  bible graph state.
- Any new Bevy render, mesh, text, picking, asset, or material features require
  dependency review in the owning leaf crate and proof that core/server remain
  free of renderer dependencies.
- Platform behavior remains behind the desktop renderer strategy boundary from
  Milestone 8. This milestone must not reintroduce embedded viewport,
  child-surface, X11-only, or raw-window-handle production paths.

Verification:

- Layout tests prove deterministic 3D positions for canonical tree,
  weighted/radial, layered, and focused-neighborhood layouts.
- Selection-index tests prove incident-edge, adjacent-node, second-level
  neighborhood, visible-edge filtering, and graph-distance highlighting without
  rescanning all edges per selection.
- Renderer tests or smoke checks prove the 3D graph opens, renders nodes,
  renders structural and semantic edges, shows bounded labels, selects nodes
  and edges, frames selection, and closes/reopens through the Milestone 8 host.
- Camera command tests prove backend-issued reset/fit/frame/navigation intents
  are validated, routed through the desktop/native renderer boundary, and
  applied to the Bevy camera without requiring Svelte camera controls.
- Completed: native camera command queue tests prove the floating renderer
  control path is bounded and drains transient camera intents without retaining
  stale commands.
- Projection tests prove empty/new projects show canonical scaffold edges and
  adding nodes/edges through Svelte commands updates the 3D projection after
  backend confirmation.
- Frontend tests prove graph filters/details/review/edit controls remain
  projection-only and do not become a second graph owner.
- Interaction smoke checks cover orbit, pan, zoom, frame selected, clear focus,
  keyboard navigation, node click, edge click, and Svelte keyboard alternatives.
- Active-context tests prove selecting a timeline clip/playhead context
  highlights the graph nodes, edges, and parent context layers that influenced
  that point.
- Dependency checks prove Bevy feature expansion stays isolated to renderer
  crates and desktop composition code.

Exit criteria:

- The bible graph is visible as an actual 3D Bevy graph in an app-managed
  floating native renderer window.
- A new project shows a meaningful canonical scaffold instead of disconnected
  blank/square nodes.
- A user can add bible nodes and edges through Svelte, see them appear in the
  3D graph, select nodes/edges in Bevy, and inspect/edit details in Svelte.
- The graph supports orbit/pan/zoom/frame/focus navigation and readable
  bounded labels.
- Selection highlights neighborhoods and dims unrelated graph regions.
- Selected timeline/playhead context can highlight active graph influence.
- The Svelte outline remains a secondary semantic/accessibility inspector, not
  the primary graph surface and not a fallback renderer path.
- The graph is usable enough that Milestone 10 agent harness/tooling can build
  on graph/context projections without hiding basic graph usability gaps.

## Milestone 10: Agent Harness And Graph Tooling

Tasks:

- Add backend-owned agent workflow contracts for workflow definitions, scoped
  tool manifests, agent runs, tool calls, tool results, tool budgets, and
  workflow policy.
- Add an agent harness executor that gives each workflow only the graph, script,
  timeline, proposal, and context tools explicitly allowed by that workflow's
  manifest.
- Add graph read tools for agent workflows: search bible nodes, read a bible
  node, read a bounded neighborhood, read the active timeline context stack,
  read active graph context for a timeline node, and read influence paths.
- Add graph proposal tools for agent workflows: propose bible nodes, fields,
  edges, and timeline-context links. Proposal tools must create reviewable
  proposal records and must not mutate canonical graph state directly.
- Add a generic graph proposal model for node, field, edge, and context-link
  targets. Do not extend the existing bible-reference proposal table into a
  catch-all schema for graph workflow proposals.
- Add context-evaluation write tools that let workflows record which graph
  nodes, edges, paths, parent contexts, and distilled context outputs were used
  for a generation task.
- Add a provider-independent structured tool loop so llama.cpp/Pumas can run
  workflows without native tool-calling support, while leaving room for
  Pantograph or native tool-call providers later.
- Add a Pumas/llama.cpp endpoint resolver for harness providers so workflows
  can use the active model server even when Pumas exposes a dynamic local port.
  The harness must not depend on the static default base URL as its only
  connection strategy.
- Keep LLM graph interaction visualization-independent. The LLM receives
  backend graph/context/influence projections and tool results; it never
  receives or mutates Bevy state, visual layout, camera state, colors,
  highlights, or renderer-local selections.
- Support premise-first graph development through the harness: premise
  generation can read bounded graph context, generate premise context, propose
  bible nodes/edges/fields, and record why each proposal was made.
- Support hierarchical workflow refinement: act generation uses premise context,
  sequence generation uses premise plus act context, scene generation uses
  premise plus act plus sequence context, and lower layers consume distilled
  parent context instead of re-evaluating the whole graph by default.
- Surface harness-generated graph proposals and context evaluations through the
  Milestone 8 graph/context projections and Milestone 9 graph experience so
  users can inspect what the AI used and why.

Workflow model:

- Workflows do not receive the app. They receive a workflow intent, bounded
  backend projections, a scoped tool manifest, validation rules, proposal
  policy, and history recording.
- Tools are backend capabilities, not UI operations. No workflow tool may
  manipulate Svelte stores, Bevy ECS state, renderer layout, or local viewport
  state.
- Read tools may query bounded projections and graph context. Proposal tools
  may create reviewable graph/script/context proposals. Commit tools are
  reserved for explicit user actions or narrowly approved backend policies.
- Every tool call and tool result is recorded against an agent run with enough
  data to review, replay, debug, and explain downstream graph/script changes.
- Tool execution is validated at the backend boundary. Tool arguments must use
  typed IDs, bounded limits, idempotent command identifiers where writes are
  involved, and explicit error handling.

Implementation order:

- Define workflow, tool manifest, agent run, tool call, tool result, and tool
  policy DTOs before wiring any model provider.
- Completed: core now defines host-agnostic agent workflow, scoped tool
  manifest, run, tool call, tool result, budget, policy, and typed graph/context
  tool argument DTOs with manifest/budget validation and serialization tests.
- Add relational SQLite current-state/history storage for agent runs, tool
  calls, tool results, and workflow status.
- Completed: server now has relational SQLite current-state tables and
  history-backed store/service functions for agent runs, tool calls, tool
  results, and run-history loading. Tagged tool arguments/results are stored as
  audited payload JSON while workflow id, status, tool name/kind, sequence, and
  timestamps remain queryable columns.
- Add a mock-provider harness test path so workflow/tool execution can be
  verified without relying on a live LLM.
- Completed: server now has a mockable agent harness loop with provider/tool
  traits, manifest and budget validation before tool execution, durable
  run/call/result recording through the agent workflow store, and tests for
  successful history recording, disallowed tool rejection, and budget
  exhaustion without a live model.
- Add read-only graph tools over existing bible graph/context projections.
- Completed: server now has read-only graph tool execution over existing
  backend projections for node search, node reads, bounded bible neighborhoods,
  active graph context, timeline context stacks, and influence-path records.
  The executor returns serialized backend projection payloads, rejects
  write/proposal tools, and does not read or mutate visualization state.
- Add generic proposal storage for graph node, field, edge, and context-link
  targets before implementing proposal-producing graph tools.
- Completed: core and server now define a separate generic graph proposal model,
  SQLite current-state/history storage, and list projection for reviewable
  bible-node, bible-field, bible-edge, and timeline-context-link proposals.
  This does not reuse bible-reference proposal rows and does not mutate
  canonical graph or timeline state.
- Add proposal-producing graph tools that write generic reviewable proposal
  records and never mutate canonical graph rows directly.
- Completed: server graph proposal tools now convert agent node, field, edge,
  and timeline-context-link proposal calls into generic reviewable proposal
  records, return proposal-list projections, require explicit edge kinds, and
  leave canonical graph, timeline, and renderer state untouched.
- Add active llama.cpp endpoint resolution through Pumas/model-service seams
  before the first live-provider harness slice.
- Completed: server now has a backend-owned llama.cpp endpoint resolver that
  normalizes explicit loopback OpenAI-compatible base URLs, rejects remote or
  malformed endpoints, and resolves active Pumas runtime-profile endpoints when
  the workflow requests Pumas-owned selection instead of a fixed port.
- Add the provider-independent structured tool loop for llama.cpp/Pumas.
- Completed: server now has a provider-independent structured JSON loop that
  renders workflow/tool history prompts for text-only providers, parses exactly
  one typed tool-call or completion object, rejects markdown/non-JSON responses,
  and enforces a bounded provider-response byte budget before harness
  validation executes any tool.
- Add the smallest workflow vertical slice: premise generation reads bounded
  graph context, proposes one bible node or edge, records the tool calls and
  rationale, and exposes the proposal for user review.
- Completed: server now has the first premise graph-context workflow slice
  that runs through the harness, uses connection-aware backend graph tools to
  read active graph context and record reviewable bible graph proposals, records
  run/tool history, and verifies the canonical graph remains proposal-gated.
- Extend the workflow harness to act, sequence, scene, beat, and shot context
  refinement after the premise slice is verified.
- Completed: the context-refinement workflow factory now covers premise, act,
  sequence, scene, beat, and shot levels with typed workflow intents and the
  same bounded graph-read/proposal manifest used by the verified premise slice.

Standards gates:

- Workflow definitions, tool manifests, tool-call requests, tool results,
  proposal payloads, budgets, and provider responses are executable boundary
  contracts with runtime validation, explicit enum/string casing, and
  serialization tests.
- The harness is a backend service with a narrow provider trait. It must not
  import Tauri, Svelte, Bevy, renderer layout, or frontend projection-store
  modules.
- Tool calls are allowlisted by manifest, budgeted, bounded, and typed. Unknown
  tools, malformed arguments, unbounded graph reads, raw SQL, raw file paths,
  raw URLs, and renderer-shaped payloads are rejected before execution.
- Agent runs, tool calls, tool results, retries, cancellations, provider
  errors, proposals, and accepted outputs are recorded durably enough to replay
  or review the workflow after restart.
- Long-running workflow execution has a lifecycle owner with cancellation,
  tracked task handles, bounded provider/tool queues, and deterministic cleanup
  before project close or app shutdown.
- Proposal tools can write reviewable proposal records only. Canonical graph,
  script, timeline, affect, or context state changes require explicit
  acceptance commands through the normal backend command/event path.
- Provider URL resolution through Pumas/llama.cpp is parsed and policy-checked
  by the backend. The harness cannot depend on a hardcoded static `18080`
  endpoint as its only success path.
- The mock-provider vertical slice must pass before live-provider workflow
  execution is introduced, so the tool loop can be verified without external
  model availability.

Verification:

- Workflow manifest tests prove a workflow receives only the tools it is
  allowed to use and that unknown/disallowed tool calls are rejected.
- Tool validation tests prove bad IDs, unbounded neighborhood reads, malformed
  proposal payloads, duplicate command IDs, and renderer-shaped payloads are
  rejected before execution.
- Read-tool tests prove graph context reads are bounded, projection-driven, and
  do not expose visualization state.
- Proposal-tool tests prove node/field/edge/context-link proposals are
  reviewable records and do not mutate canonical graph state before acceptance.
- Proposal-schema tests prove graph workflow proposals are represented by the
  generic proposal model and do not overload bible-reference proposal rows.
- Agent-run history tests prove tool calls, tool results, generated output,
  proposals, errors, retries, and cancellation are recorded and reloadable.
- Provider-resolution tests prove the harness can discover or configure the
  active llama.cpp endpoint exposed by Pumas and does not assume `18080`.
- Provider-loop tests prove the structured tool loop works with a mock
  llama.cpp/Pumas-style text provider that has no native tool-calling support.
- Vertical slice: generate premise context -> harness reads bounded graph
  context -> model proposes bible nodes/edges -> user accepts proposals ->
  graph projection shows new premise-owned context and records generation
  influence.

Exit criteria:

- The agent harness can run at least one graph-aware workflow with a scoped
  tool manifest, bounded backend reads, reviewable graph proposals, and
  recorded tool history.
- The LLM can navigate the bible graph through backend read operations and
  propose node/field/edge/context-link changes without knowing about or
  mutating the visualization.
- AI-created bible graph additions are visible as proposals and become durable
  graph state only through backend acceptance commands.
- Agent runs and tool calls are traceable enough to explain what graph context
  influenced a generated timeline context.
- Graph workflow proposals use a general proposal target model instead of
  stretching bible-reference proposal storage beyond its purpose.
- The harness remains provider-independent for llama.cpp/Pumas now and future
  Pantograph integration later.

## Milestone 11: Affect Model And Prompting Semantics

Tasks:

- Add backend-owned affect contracts for valence, arousal, mood labels,
  emotional intensity, confidence, provenance, and the story scope where the
  affect applies.
- Store affect values in relational SQLite current-state rows with
  command/event/revision history. Do not store queryable affect values only as
  JSON blobs.
- Support affect targets at the useful story levels: project/story, act,
  sequence, scene, beat, shot, script segment, bible node, and bible snapshot
  where appropriate.
- Add affect commands for setting, revising, deleting, and accepting proposed
  affect changes through the same command-in/projection-out backend boundary as
  timeline, script, and bible edits.
- Add affect projections for timeline overlays, script-generation context,
  change review, and bible/script influence analysis.
- Add semantic dependency records that link affect traits to the script
  segments, timeline nodes, bible fields, and generation prompts they influence.
- Extend AI proposal flows so detected affect changes from manual script edits
  become reviewable proposals instead of silent graph/script mutations.
- Define prompt construction rules for how valence/arousal curves and local
  affect constraints influence generated script without overriding locked text
  or accepted bible facts.
- Add Svelte review/edit surfaces for affect values using projection caches and
  backend commands only. Draft slider/input state may be local, but persisted
  affect state is backend-owned.
- Document how affect values are interpreted by generation, timeline overlays,
  propagation, undo/redo, and change review.

Implementation order:

- Define the core affect DTOs and value ranges first, including validation rules
  for valence, arousal, confidence, target scope, source/provenance, and command
  IDs.
- Completed: core now defines host-agnostic affect contracts with typed
  basis-point valence, arousal, intensity, and confidence values, mood labels,
  provenance, project/timeline/script/bible targets, set/delete commands, and
  serialization/validation tests.
- Add SQLite schema/current-state/revision storage for affect values and
  dependencies.
- Completed: server now has relational SQLite current-state/history storage for
  affect values, mood-label rows, soft delete, idempotent set/delete recording,
  target-scoped projections, and queryable affect dependency rows linking
  affect traits to timeline, script, bible, and generation-prompt endpoints
  through host-neutral service functions.
- Add the smallest command/projection vertical slice: set affect for one
  timeline node, read the affect projection, and replay it after project reload.
- Completed: desktop command/projection adapters and Svelte API helpers now
  expose backend-owned affect set/projection operations for timeline-node affect
  values, with a reload/replay service test proving projections are rebuilt from
  durable SQLite state.
- Add script-generation context integration after the affect projection is
  durable and tested.
- Completed: script-generation request hydration now loads timeline-node affect
  projections with bible context, and prompt formatting appends renderer-neutral
  affect constraints for generation/decomposition with focused prompt coverage.
- Add proposal/review integration for AI-detected affect changes from manual
  script edits.
- Add timeline overlay projection output only after affect state and generation
  semantics are backend-owned.
- Completed: timeline render projections now include backend-derived
  `affect_overlays` samples for timeline-node affect values, joining durable
  affect state with current clip ranges for future Svelte/Bevy rendering.

Standards gates:

- Valence, arousal, confidence, intensity, target scope, provenance, and mood
  values are validated domain types. Internal APIs do not pass raw numbers or
  strings where invalid values would cross module boundaries.
- Affect persistence uses relational SQLite current-state, dependency, command,
  event, and revision rows for queryable values. JSON cannot be the canonical
  store for affect curves, targets, confidence, or influence records.
- Affect commands are idempotent, replayable, undoable, and transactional with
  their semantic dependency writes. A failed affect update must not partially
  update projections or prompt context.
- Prompt construction stays in sync domain helpers where possible. Async model
  calls live at the provider boundary, and locked script spans are never
  rewritten by affect prompt integration.
- Manual script edits that imply affect changes become proposals or explicit
  accepted commands; no AI analysis path may silently mutate canonical affect,
  bible, script, or timeline state.
- Svelte affect controls own draft slider/input state only. Saved affect,
  proposed affect, and overlay data are backend projections.
- Affect overlay projections are renderer-neutral contracts consumed by both
  Svelte and Bevy. Renderer crates must not invent or persist their own affect
  scores.

Verification:

- Vertical slice: set beat-level valence/arousal -> read projection ->
  generation context includes the affect constraint -> timeline overlay
  projection exposes the curve.
- History tests prove affect updates, deletes, proposal acceptance/rejection,
  undo, redo, and reload rebuild the same affect projections.
- Validation tests reject out-of-range affect values, unknown target IDs,
  duplicate command IDs, and renderer-supplied payloads that bypass backend
  validation.
- Prompt-construction tests prove locked script spans are preserved and affect
  constraints influence only unlocked/regenerated context.
- Semantic dependency tests prove the system can identify which script segments
  and timeline nodes were influenced by an accepted affect change.
- Frontend tests prove affect edit/review controls replace projection caches
  from backend responses and do not mutate durable affect state locally.

Exit criteria:

- Affect has a canonical backend-owned relational model with command/event/
  revision history.
- Valence/arousal overlays are renderable from backend projections without
  renderer-local persistent state.
- Script generation can consume affect context from projections in a
  deterministic, test-covered way.
- AI-detected affect changes are reviewable proposals before acceptance.
- Undo/redo and before/after review can trace affect changes and their
  downstream influence.

## Milestone 12: Bevy Timeline Renderer

Tasks:

- Upgrade renderer planning and crates to Bevy 0.18.1 with a fresh dependency
  review before enabling render/window/input/text features.
- Add renderer-facing timeline projection DTOs only where the existing
  backend-owned `TimelineRenderProjection` is insufficient.
- Add native Bevy timeline host as an isolated leaf renderer managed by the
  same floating renderer host pattern used by the bible graph.
- Add pan, zoom, playhead, selection, hit testing, move, resize, split, arcs,
  relationship curves, and affect overlays through backend-confirmed commands
  and projections.
- Add keyboard-accessible Svelte command alternatives for critical timeline operations.
- Remove the DOM/SVG timeline renderer after Bevy covers target interactions.

Implementation order:

- Confirm existing backend-owned `TimelineRenderProjection` and timeline
  command contracts cover the native renderer slice. Add renderer-facing DTOs
  only for missing renderer-neutral data, and define them before native host
  work.
- Remove remaining WASM renderer bridges and wasm-only dependency paths before
  introducing the native Tauri-owned Bevy host as the supported desktop path.
- Reuse the floating renderer host from Milestone 8 for the timeline renderer
  window. Timeline work may add timeline-specific renderer state, but must not
  add a second lifecycle, focus, input, or command-drain framework.
- Do not reintroduce embedded viewport, WebView child-surface, WASM, local
  HTTP/WebSocket, or split-process renderer-sidecar paths while replacing the
  timeline. If the Milestone 8 host cannot support timeline needs, re-plan the
  shared floating host contract before adding timeline-specific infrastructure.
- Add the native Bevy timeline host as a leaf renderer managed in the desktop
  composition root through that shared floating renderer host.
- Add the smallest native renderer vertical slice: receive a projection, build
  disposable ECS render state, hit-test one clip, emit one validated command,
  and apply the returned backend projection.
- Add pan/zoom/playhead/selection/move/resize/split/delete/create interactions
  through backend-confirmed commands before adding arcs, relationship curves,
  or affect overlays.
- Add affect overlays only after Milestone 11 provides backend-owned affect
  projections.
- Remove the DOM/SVG timeline renderer and its tests only after Bevy and
  Svelte accessibility alternatives cover the target interactions.

Standards gates:

- The native Bevy timeline host is managed through the shared floating renderer
  host, has tracked tasks/subscriptions, bounded command queues,
  shutdown/cancellation coverage, and no local HTTP/WebSocket/WASM transport
  fallback.
- Timeline renderer lifecycle, status, command draining, and projection
  subscription must remain backend/desktop-host owned. Svelte may expose
  controls and status but must not store durable timeline renderer state or
  mutate timeline projections optimistically.
- Bevy consumes projection snapshots and produces command requests only.
  Timeline selection, clip data, arcs, relationships, affect overlays, and
  saved layout decisions stay backend-owned.
- Renderer input payloads and dimensions are validated with checked arithmetic
  before allocation or hit-test math. Malformed renderer commands are rejected
  at the backend boundary before timeline mutation.
- Critical canvas/Bevy actions have semantic Svelte keyboard alternatives that
  call the same command path and wait for backend-confirmed projections.
- The DOM/SVG timeline removal includes dead-code and dependency cleanup so
  old render paths, wasm-bindgen wrappers, and unused web renderer dependencies
  do not remain as supported fallbacks.
- Bevy 0.18.1 feature selection and dependency cost are documented before
  render/window/input/text features are enabled, and Bevy remains out of
  `eidetic-core`.

Verification:

- Vertical slice: Bevy timeline render model receives projection and emits a validated command.
- Floating renderer reuse check proves the timeline window uses the same
  renderer lifecycle, focus, input-routing, and command-delivery owner as the graph
  window.
- Pointer, focus, keyboard, and parent-gesture conflict smoke checks.
- Projection serialization tests.
- Renderer lifecycle tests for mount/unmount subscription cleanup.
- Dependency review proving Bevy is not in `eidetic-core` and is justified/feature-gated if it adds 100+ transitive dependencies.

## Cross-Cutting Implementation Requirements

Code organization:

- Files over 500 lines require decomposition review.
- UI components over 250 lines require decomposition review.
- Modules/services with more than roughly 7 public functions or 3 responsibilities require decomposition review.
- Directories under `src/` touched by the refactor must have current READMEs.
- Feature work must not expand already-over-threshold files before the
  decomposition decision is completed and recorded.

Contracts and boundaries:

- Contracts are defined before implementations and frozen for each parallel or
  cross-layer slice.
- Public wire DTOs use explicit serde tags/casing and mirrored TypeScript types
  in the same slice.
- Boundary payloads are parsed once into validated values. Internal code must
  accept the validated types instead of raw strings, numbers, JSON blobs, or
  renderer/provider payloads.
- Append-only contract extension is preferred. Any breaking contract rewrite is
  allowed only because this refactor explicitly does not preserve old runtime
  compatibility, and the implementation must delete the replaced path in the
  same milestone.

Concurrency:

- Domain/core APIs stay synchronous unless concurrent I/O is intrinsic.
- Every `tokio::spawn` must be owned by a lifecycle manager with tracked handles, cancellation, shutdown, panic logging, and draining.
- No lock guards may be held across `.await` unless the lock type and design explicitly support it.
- Related mutable state has one owner.
- Frontend, renderer, provider, and agent event queues are bounded and document
  overflow behavior.
- Async responses from commands, provider calls, and event streams carry
  correlation/version data so stale responses can be discarded without mutating
  projection caches.

Interop:

- Tauri IPC, legacy HTTP while it remains, legacy WebSocket while it remains,
  Y.Doc bridge, and Bevy bridge payloads are trust boundaries.
- Validate every boundary payload before dispatch.
- Use explicit serde wire shapes and round-trip tests.
- Unsubscribe all event/bridge subscriptions on teardown.
- Document thread requirements for Tauri command/event adapters, Bevy bridge
  calls, and callbacks.
- Rust/TypeScript/Bevy/Tauri boundary tests must prove both sides preserve the
  same wire shape, defaults, enum meanings, and error semantics.

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
- Agent tools and provider responses must not pass raw SQL, raw paths, raw
  URLs, unbounded prompt bodies, or renderer-local payloads into domain logic.
- Boundary diagnostics must include enough bounded correlation context to debug
  failures without logging secrets, credentials, binary assets, or unbounded
  model/provider payloads.
- If any temporary local listener exists before Milestone 7 completes, bind it
  to loopback and enforce connection limits; no production desktop path may keep
  a local HTTP/WebSocket listener after Milestone 7.

Dependencies:

- Bevy and other heavy renderer dependencies stay out of `eidetic-core`.
- Dependencies are declared by the package/crate that directly uses them.
- Heavy optional features are feature-gated.
- Dependency cost is measured before adding renderer/RAG/export dependencies.
- Dependency removal is verified when old Axum, WASM, DOM/SVG renderer, or
  diffusion paths are deleted so unused packages do not remain hidden in
  manifests or lockfiles.

Tooling:

- Warnings are errors in CI.
- Generated contracts are deterministic and verified.
- Persisted schema fixtures, command fixtures, projection fixtures, and sample events have validation hooks.
- Decision traceability checks must run for touched source directories.
- Launcher run/build/test/release-smoke paths must remain the documented local
  and CI entry points for desktop verification.

## Baseline Verification Commands

Use the repository's actual scripts where they differ, but the implementation plan must provide equivalents for:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo check --workspace --all-features
cargo check --workspace --no-default-features
npm run format:check
npm run lint
npm run typecheck
npm run test
./launcher.sh --test
./launcher.sh --release-smoke
```

Performance-sensitive graph queries, projection rebuilds, and renderer projection serialization require Criterion benchmarks before performance claims or regression budgets are accepted.

Dependency-sensitive slices must also record the relevant dependency review in
the plan, using the package manager for the affected boundary, for example
`cargo tree`, `cargo tree --duplicates`, `cargo tree -i <crate>`, `npm ls`, or
equivalent commands. Bevy upgrades, Axum/WASM removal, provider integration,
and renderer feature changes always require this review.

## Risks And Mitigations

- Dual truth between old and new canonical models: delete old ownership paths in the same milestone where replacements become active.
- Bevy becoming a second state owner: Bevy receives projections and emits commands only.
- Svelte remaining a hidden state owner: finish Milestone 6 before adding more renderer work; durable UI data must be projection cache replacement only.
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
- A frontend store cannot be classified cleanly as transient UI state or projection cache.
- A UI workflow requires broad durable project/timeline mutation after focused commands/projections exist.
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
- Frontend stores contain only transient UI state or discardable projection caches.
- The DOM/SVG timeline renderer is removed.
- The 2D SVG relationship graph is replaced by the Bevy bible graph or removed as a supported graph view.
- Standards-required tests, docs, lifecycle owners, validators, and dependency reviews are present for all touched areas.
