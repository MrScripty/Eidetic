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
- Current product direction: finish making Svelte a projection consumer before replacing the timeline renderer with Bevy.

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

Discovered issues:

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
- Open: Milestone 6 lists valence/arousal overlays, but the codebase has no canonical backend source of truth for valence, arousal, mood, or affect scores. Before renderer work can continue on that overlay, add a backend-owned affect/overlay contract with history-backed commands, projection tests, and script/timeline influence semantics; do not add renderer-only overlay state.
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
- Open: Milestone 7 route/service extraction still has Axum-shaped route
  handlers and route tests. Reusable validators now return backend-neutral
  errors, and project route behavior now has a host-neutral service boundary,
  but command/projection/AI/export/reference route handlers and many route tests
  still use HTTP status codes as the behavior boundary; extract service
  functions and service-level tests before adding Tauri command adapters.
- Resolved: the first Tauri dependency resolution selected `tauri` 2.10.3 with
  newer 2.11 runtime crates, which failed inside `tauri-runtime-wry`. The desktop
  crate now pins `tauri` to 2.11.2 so the runtime stack resolves consistently.
- Open: Milestone 7 lifecycle compliance is blocked by detached backend tasks.
  Autosave, Y.Doc, AI generation, batch generation, and reference embedding use
  `tokio::spawn` without a runtime owner; move them behind a Tauri-owned
  backend lifecycle supervisor before deleting Axum startup.
- Resolved: `crates/server/src/command_service.rs` reached 681 lines after bible
  graph command extraction. Bible graph command handling now lives in the
  focused `command_service_bible.rs` module and shared helpers live in
  `command_service_support.rs`, reducing the main service file below the
  decomposition threshold before further command migration.

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
- Bevy timeline viewport and Bevy bible graph view as projection consumers.
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

## Milestone 8: Bevy Timeline Renderer

Tasks:

- Upgrade renderer planning and crates to Bevy 0.18.1 with a fresh dependency
  review before enabling render/window/input/text features.
- Add renderer-facing timeline projection DTOs only where the existing
  backend-owned `TimelineRenderProjection` is insufficient.
- Add native Bevy timeline host as an isolated leaf renderer owned by the Tauri
  desktop runtime.
- Add pan, zoom, playhead, selection, hit testing, move, resize, split, arcs, relationship curves, and valence/arousal overlays through backend-confirmed commands/projections.
- Add keyboard-accessible Svelte command alternatives for critical timeline operations.
- Remove the DOM/SVG timeline renderer after Bevy covers target interactions.

Verification:

- Vertical slice: Bevy timeline render model receives projection and emits a validated command.
- Pointer, focus, keyboard, and parent-gesture conflict smoke checks.
- Projection serialization tests.
- Renderer lifecycle tests for mount/unmount subscription cleanup.
- Dependency review proving Bevy is not in `eidetic-core` and is justified/feature-gated if it adds 100+ transitive dependencies.

## Milestone 9: Bevy Bible Graph View

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

- Tauri IPC, legacy HTTP while it remains, legacy WebSocket while it remains,
  Y.Doc bridge, and Bevy bridge payloads are trust boundaries.
- Validate every boundary payload before dispatch.
- Use explicit serde wire shapes and round-trip tests.
- Unsubscribe all event/bridge subscriptions on teardown.
- Document thread requirements for Tauri command/event adapters, Bevy bridge
  calls, and callbacks.

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
- If any temporary local listener exists before Milestone 7 completes, bind it
  to loopback and enforce connection limits; no production desktop path may keep
  a local HTTP/WebSocket listener after Milestone 7.

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
