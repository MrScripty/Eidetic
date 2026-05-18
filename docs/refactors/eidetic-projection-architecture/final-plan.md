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

Discovered issues:

- Commit hooks report `Can't find lefthook in PATH`. Commits succeed, but tooling setup is incomplete and should be fixed before treating hook execution as a verified gate.
- Baseline `cargo fmt --all -- --check` reports pre-existing formatting drift in server files. Do not mix that repo-wide cleanup into feature slices; either add a dedicated formatting cleanup slice or intentionally defer it with CI expectations updated.
- `cargo test -p eidetic-server history_store` passes but reports pre-existing dead-code warnings in `diffusion/types.rs` and `ydoc.rs`. These warnings block a future `-D warnings` gate and need a cleanup or ownership decision before CI can enforce warning-free server builds.
- The first implementation attempt exposed the stale Pumas path and lockfile state as a build metadata blocker. The path and lockfile are now fixed, and future slices should use Cargo verification instead of relying on stale metadata.
- The command route currently opens the active SQLite project path per request because `AppState` has no backend-owned database connection owner yet. Before broad route adoption, add an explicit database lifecycle owner with the same concurrency/shutdown discipline as the rest of the backend.
- Frontend bible editing currently mutates broad `Entity` caches and whole detail objects (`EntityDetail.svelte`, `story.svelte.ts`, `api.ts`, and websocket invalidation handlers). UI command adoption must use focused projection stores and avoid optimistic local patching or treating form state as canonical.

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
