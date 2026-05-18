# Codebase Impact Review

## Purpose

This review maps the projection-architecture plan onto the current codebase. It records the blast radius, anti-patterns, simplification opportunities, reasoning impact, maintainability impact, and performance impact for each touched area.

The central conclusion is unchanged: implementation should replace the old ownership model rather than layering new systems beside it.

## Cross-Cutting Findings

Current code has four overlapping state owners:

- `Project` in `AppState.project`
- timeline node `NodeContent`
- `StoryBible { entities }`
- Y.Doc text state

The plan requires one backend-owned canonical source of truth. Therefore the safest implementation path is not an adapter layer around the existing model. It is a staged replacement:

```text
command -> transactional SQLite write -> event/revision -> projection rebuild -> Svelte/Bevy render
```

Main anti-pattern to avoid:

- Adding `BibleGraph`, `ScriptDocument`, `ChangeEvent`, and Bevy render models while leaving `Entity`, Beat-owned screenplay text, snapshot undo, and DOM/SVG timeline rendering active as supported paths.

Main simplification opportunity:

- Make `Project` a bootstrap/read projection instead of the object every route clones, mutates, saves, and restores.

## `crates/core/src/project`

Current role:

- `Project` aggregates `Timeline`, `Vec<StoryArc>`, `StoryBible`, and references.
- It is the mutation and persistence unit for most server routes.

Blast radius:

- Every route that locks `AppState.project`.
- Persistence save/load.
- AI context construction.
- Undo/redo snapshots.
- UI bootstrap types.

Anti-patterns:

- Expanding `Project` with new graph/script/history fields while keeping the old fields canonical.
- Treating `Project` clone/restore as the undo model.
- Building AI contexts from the full project object after projections exist.

Simplification:

- Turn `Project` into a project metadata/read projection.
- Move mutations to command handlers that operate through focused repositories.

Reasoning and maintainability:

- A smaller `Project` makes ownership visible. Domain areas stop depending on unrelated global state.

Performance:

- Avoids repeated whole-project cloning for undo and save.
- Reduces payload size for UI bootstrap and AI context paths.

## `crates/core/src/timeline`

Current role:

- Owns tracks, flat nodes, parent-child hierarchy, time ranges, node arcs, relationships, and node content.
- `StoryLevel` already supports `Premise -> Act -> Sequence -> Scene -> Beat`.
- `NodeContent.content` currently carries script/outline text.
- `StoryNode.locked` acts as a regeneration lock.

Blast radius:

- `StoryNode`, `NodeContent`, and `ContentStatus`.
- Timeline mutation routes.
- Script panel, editor, export, AI generation, consistency reaction.
- Persistence `nodes.content_json`.
- Y.Doc node text keys.

Anti-patterns:

- Keeping `NodeContent.content` as final screenplay text.
- Keeping `ContentStatus::HasContent` as generated screenplay state.
- Using node-level `locked` as script-span protection.
- Splitting/moving nodes while implicitly splitting script content or semantic history.

Simplification:

- Replace `NodeContent` with timeline-context fields:

```text
TimelineContext
  notes
  instructions
  constraints
  recap
  context_status
```

- Keep `Timeline` responsible for time, hierarchy, arcs, and context only.
- Script ownership moves to `ScriptDocument`.

Reasoning and maintainability:

- Timeline bugs become time/hierarchy bugs, not script persistence bugs.
- Script tests can run without timeline mutation side effects.

Performance:

- Bevy timeline projections can be compact: track, node id, range, level, labels, overlays.
- Script text does not need to be shipped to the renderer for every timeline update.

## `crates/core/src/story`

Current role:

- Fixed `EntityCategory` and `EntityDetails`.
- `StoryBible` is `Vec<Entity>`.
- Entity snapshots and relations live inside entity structs.
- Prompt text formatting is embedded on `Entity`.

Blast radius:

- Story routes.
- Bible UI.
- Relationship graph.
- AI extraction.
- Prompt formatting.
- Persistence `entities`, `entity_snapshots`, `entity_relations`.
- TypeScript entity unions.

Anti-patterns:

- Replacing `EntityCategory` with a larger hard-coded enum.
- Keeping canonical category data in Rust enum variants while adding custom fields elsewhere.
- Letting `Entity::to_prompt_text` decide AI context policy.
- Persisting graph facts as JSON to avoid schema work.

Simplification:

- Replace the fixed entity model with composable graph tables:

```text
bible_nodes
bible_parts
bible_part_fields
bible_edges
bible_snapshots
bible_assets
semantic_claims
semantic_dependencies
```

- Default screenwriting concepts become seeded schemas, not special code branches.
- Prompt text comes from an AI context projection service.

Reasoning and maintainability:

- Adding a custom worldbuilding category no longer requires coordinated Rust enum, TypeScript union, Svelte branch, persistence JSON, and prompt changes.

Performance:

- Field-level rows allow indexed queries such as "locations with rainy weather active in this sequence" or "segments depending on this part field".
- Bounded graph projections avoid rendering or prompting with the full bible.

## `crates/core/src/script`

Current role:

- Contains screenplay parsing, formatting, and merge helpers.
- Does not own the canonical script artifact.
- Svelte duplicates part of the parser in `ScriptView.svelte`.

Blast radius:

- Script editor/viewer.
- Export.
- AI generation.
- Diffusion infill.
- Consistency suggestions.
- Y.Doc text synchronization.

Anti-patterns:

- Continuing to treat raw text as the only canonical script representation.
- Duplicating screenplay parsing rules in Rust and Svelte.
- Applying locks at node level rather than block/span level.

Simplification:

- Promote this module to the canonical script domain:

```text
ScriptDocument
ScriptSegment
ScriptBlock
ScriptSpan
ScriptLock
ScriptPatch
ScriptProjection
```

- Centralize parsing/formatting in Rust and expose projection DTOs to UI/export.

Reasoning and maintainability:

- Direct script editing, locks, provenance, and export become one domain problem instead of scattered editor/route/export/Y.Doc behavior.

Performance:

- Regeneration and edits can patch affected blocks/spans instead of replacing a whole Beat text blob.

## `crates/core/src/ai`

Current role:

- `build_generate_request` receives full `Project`.
- Prompt context pulls target node, siblings, arcs, entity bible context, recaps, and surrounding node content.
- Consistency uses current node content and downstream sibling/relationship heuristics.

Blast radius:

- AI routes.
- Prompt formatting.
- Script generation.
- Child planning.
- Semantic propagation.
- RAG/reference context.

Anti-patterns:

- Prompt builders querying arbitrary project internals.
- AI workflows committing script or bible state directly.
- Using name matching and sibling order as the primary impact model.
- Keeping consistency suggestions as text replacement diffs only.

Simplification:

- Use narrow AI projection DTOs:

```text
GenerationContextProjection
SemanticEditAnalysisInput
SemanticClaimProposal
ScriptPatchProposal
PropagationPlan
```

- Keep deterministic context packing testable without model calls.

Reasoning and maintainability:

- AI becomes a proposal producer, not a state owner.
- Prompt behavior can be tested from stable projection fixtures.

Performance:

- Context can be assembled from indexes and dependencies instead of scanning full project state and all entities.

## `crates/server/src/persistence.rs`

Current role:

- One 1500+ line file owns schema creation, save, load, JSON migration, v1/v2/v3 compatibility, clear-and-rewrite persistence, entity serialization, node serialization, references, and Y.Doc BLOB persistence.
- Existing canonical fields use JSON columns such as `content_json`, `details_json`, `overrides_json`, and relationship type JSON.

Blast radius:

- All durable project state.
- Load/save behavior.
- Undo/recovery semantics.
- New graph/script/history schema.
- Tests and fixture strategy.

Anti-patterns:

- Adding graph/script/history tables to the same monolithic file.
- Preserving clear-and-rewrite saves for accepted command state.
- Storing queryable canonical facts in JSON.
- Replaying full event history for normal reads instead of maintaining projections/current-state tables.

Simplification:

- Split into focused persistence modules:

```text
persistence/schema.rs
persistence/project_repo.rs
persistence/timeline_repo.rs
persistence/bible_repo.rs
persistence/script_repo.rs
persistence/history_repo.rs
persistence/projection_repo.rs
persistence/asset_repo.rs
persistence/transactions.rs
```

- Use transactional command writes, current-state tables, append-only revisions, and projection rebuild tests.

Reasoning and maintainability:

- Each repository owns one domain table set and one test surface.
- Schema changes become reviewable by domain.

Performance:

- Avoids whole-project rewrite on small edits.
- Enables targeted indexes for graph, script, dependency, and projection queries.

## `crates/server/src/state.rs`

Current role:

- Owns `Arc<Mutex<Option<Project>>>`, broadcast events, Y.Doc channels, AI config, generating/extracting/diffusing sets, project path, snapshot undo stack, vector store, save channel, diffusion channel, and model library.
- Spawns auto-save with `tokio::spawn` and drops other task handles.

Blast radius:

- Every route.
- Undo/redo.
- Background save.
- AI generation/extraction locks.
- WebSocket events.
- Y.Doc/diffusion lifecycle.

Anti-patterns:

- Keeping `Arc<Mutex<Option<Project>>>` as the central mutation path.
- Process-local snapshot undo for durable history.
- Discarded spawned task handles.
- Multiple independent `HashSet` state machines for long-running work with no durable command/event ownership.

Simplification:

```text
AppState
  command_bus
  repositories
  projection_broadcaster
  task_supervisor
  ai_generation_manager
  propagation_manager
  ydoc_bridge_manager
  bevy_bridge_manager
```

Reasoning and maintainability:

- Runtime state becomes composition wiring.
- Business state moves to commands/repositories/events.

Performance:

- Smaller locks and transactions reduce contention around AI, save, and projection updates.

## `crates/server/src/routes`

Current role:

- Routes validate some inputs inline, call `snapshot_for_undo`, mutate `Project`, emit coarse events, and trigger saves.
- Long route files own business behavior, persistence side effects, and event side effects.

Blast radius:

- All API contracts.
- TypeScript API wrapper.
- WebSocket invalidation model.
- Tests.

Anti-patterns:

- Route handlers implementing graph/script/timeline mutation logic.
- Returning ad hoc `serde_json::json!` shapes for long-lived contracts.
- Mutating frontend-visible state before command persistence is complete.

Simplification:

```text
Raw payload -> boundary validator -> typed command -> handler -> projection response
```

- Routes become thin adapters.
- Command handlers are tested without HTTP.

Reasoning and maintainability:

- HTTP is no longer the place where business invariants hide.
- Wire contracts become explicit.

Performance:

- Routes can return focused projections or event IDs rather than broad payloads.

## `crates/server/src/routes/ai.rs` And `prompt_format.rs`

Current role:

- AI routes generate content, batch-generate, react to edits, extract entities, commit extraction, generate recaps, and mutate `Project`/Y.Doc.
- Auto-extraction commits bible changes after generation without a proposal review boundary.

Blast radius:

- Script generation.
- Bible updates.
- Semantic propagation.
- Change review.
- Background task lifecycle.
- Prompt contracts.

Anti-patterns:

- AI response directly mutating canonical script/bible state.
- Auto-extraction committing graph changes without user review.
- Prompt parsing helpers using ad hoc JSON extraction as durable contract parsing.
- One route module owning generation, extraction, recap, propagation, and persistence side effects.

Simplification:

```text
ai/context_projection.rs
ai/prompts/generation.rs
ai/prompts/semantic_claims.rs
ai/prompts/script_patch.rs
ai/proposal_parser.rs
ai/orchestrator.rs
routes/ai_generation.rs
routes/ai_proposals.rs
```

Reasoning and maintainability:

- Every AI workflow has a visible proposal, acceptance, rejection, and event history path.

Performance:

- AI context can be sliced from projections and dependency indexes.
- Avoids broad re-analysis when a single field or script span changes.

## `crates/server/src/ydoc.rs` And WebSocket

Current role:

- Y.Doc manager owns CRDT text keyed by node notes/content.
- WebSocket mixes JSON server events and binary Y.Doc updates.
- UI also receives REST refetches on coarse events.

Blast radius:

- Script editing.
- Notes editing.
- WebSocket protocol.
- Save/load `ydoc_state`.
- Active editor buffers.

Anti-patterns:

- Treating Y.Doc and SQLite as equal canonical stores.
- Keeping durable script locks/provenance only in CRDT attributes.
- Inferring semantic history from CRDT deltas after the fact.
- Mixing durable semantic event history with binary sync protocol.

Simplification:

- SQLite script blocks/spans are canonical.
- Y.Doc, if retained, is only an active buffer transport/cache.
- Accepted Y.Doc changes are converted into explicit script commands/events.
- WebSocket events use typed projection envelopes with object IDs and projection versions.

Reasoning and maintainability:

- One durable history model handles REST, AI, undo/redo, and collaborative edits.

Performance:

- Active text editing can stay efficient without making historical queries depend on CRDT reconstruction.

## `crates/server/src/export.rs`

Current role:

- Builds PDF by collecting Beat nodes and parsing `node.content.content`.

Blast radius:

- Export routes.
- Script parser/formatter.
- ScriptDocument model.

Anti-patterns:

- Continuing export from timeline nodes after script ownership moves.
- Re-parsing raw strings when structured script blocks/spans exist.

Simplification:

```text
ScriptDocument -> ExportProjection -> PDF/Text/Fountain renderers
```

Reasoning and maintainability:

- Export becomes independent of timeline internals.

Performance:

- Export can stream ordered blocks/spans and avoid scanning unrelated timeline nodes.

## `crates/server/src/diffusion`

Current role:

- Diffusion infilling mirrors Y.Doc manager style and operates around node text regions.
- Manager is spawned separately from the main task lifecycle.

Blast radius:

- Script patching.
- AI proposal flow.
- Task lifecycle.
- Python bridge.

Anti-patterns:

- Region rewrite APIs based on raw offsets into node-owned content.
- Diffusion as a side channel outside reviewable script patch proposals.
- Dropped manager handles without a lifecycle owner.

Simplification:

- Treat diffusion output as `ScriptPatchProposal`.
- Anchor ranges to script block/span IDs.
- Put manager ownership under a task/lifecycle supervisor.

Reasoning and maintainability:

- All text-changing AI paths share the same review and provenance model.

Performance:

- Patches can target scoped spans instead of whole node buffers.

## `crates/server/src/vector_store.rs`, `embeddings.rs`, `routes/reference.rs`

Current role:

- RAG/reference support feeds generation prompts.
- Reference content is currently separate from the planned graph asset/reference model.

Blast radius:

- AI context.
- Asset storage.
- Semantic provenance.
- Path/URL validation.

Anti-patterns:

- Including reference chunks in prompts without recording dependencies.
- Accepting paths or URLs outside centralized backend validation.
- Treating RAG content as prompt-only, not provenance-bearing input.

Simplification:

```text
ReferenceAsset
ReferenceChunk
ReferenceDependency
```

- References become graph-linked assets with provenance and dependency edges.

Reasoning and maintainability:

- Script and bible changes can explain which references influenced them.

Performance:

- Search can be filtered by active time range, linked bible nodes, or script segment dependencies.

## `ui/src/lib/types.ts` And `ui/src/lib/api.ts`

Current role:

- Manually mirrors Rust DTOs.
- Exposes entity, node-script, timeline, AI, reference, and export endpoints.

Blast radius:

- Every UI component and store.
- API contract tests.
- Future generated DTOs.

Anti-patterns:

- Manually maintaining Rust and TypeScript command/event/projection contracts.
- Keeping compatibility wrappers for old entity/node-script APIs.
- Fetching broad objects after every event.

Simplification:

```text
types/generated.ts
api/commands.ts
api/projections.ts
api/assets.ts
api/ai-proposals.ts
```

Reasoning and maintainability:

- Command/query separation becomes visible in imports.
- Generated/schema-backed contracts reduce drift.

Performance:

- UI can fetch focused projections instead of full timeline/entity payloads.

## `ui/src/lib/stores`

Current role:

- Stores hold timeline data, story entities/arcs, selected editor node, websocket event handlers, generation state, transient timeline viewport state, and notifications.
- `wsHandlers.ts` refetches broad state on coarse events and mutates cached node content directly.

Blast radius:

- All UI surfaces.
- WebSocket protocol.
- Bevy bridge stores.
- Projection cache ownership.

Anti-patterns:

- Storing canonical graph/script state in Svelte stores.
- Mutating backend-owned cached objects optimistically.
- Letting Svelte, Bevy, and backend each own business selection state.
- Coarse event handlers refetching large state after small updates.

Simplification:

```text
stores/projections/
  timelineProjection.svelte.ts
  scriptProjection.svelte.ts
  bibleProjection.svelte.ts
  changeReviewProjection.svelte.ts

stores/ui/
  viewport.svelte.ts
  panelState.svelte.ts
  transientSelection.svelte.ts

stores/bridges/
  bevyTimelineBridge.svelte.ts
  bevyBibleGraphBridge.svelte.ts
```

Reasoning and maintainability:

- Store names encode ownership: backend projection cache vs transient UI state.

Performance:

- Versioned projection events reduce broad invalidation and rerendering.

## `ui/src/lib/components/editor`

Current role:

- `BeatEditor.svelte` is over 1200 lines and owns notes, hierarchy context, Y.Doc observation, script editing, generation, extraction, linked entities, child planning, raw prompt preview, and consistency suggestions.
- `ScriptPanel.svelte` treats Beat nodes as script sections.
- `ScriptView.svelte` duplicates screenplay parsing and uses generated HTML for highlights.

Blast radius:

- Script editing.
- Timeline context editing.
- AI generation/review.
- Bible extraction/review.
- Accessibility and test strategy.

Anti-patterns:

- Growing `BeatEditor.svelte` into the editor for context, script, graph updates, and AI review.
- Keeping direct script editing tied to Beat node content.
- Continuing client-side duplicate screenplay parsing.
- Ignoring accessibility warnings on clickable non-interactive elements.

Simplification:

```text
TimelineContextEditor.svelte
ScriptDocumentEditor.svelte
ScriptBlock.svelte
ScriptLockControls.svelte
ChangeReviewPanel.svelte
SemanticClaimReview.svelte
ScriptPatchReview.svelte
AiContextPreview.svelte
```

Reasoning and maintainability:

- Each editor surface maps to one projection and one command family.

Performance:

- Editing one script block should not rerender the timeline, full script panel, and bible sidebar.

## `ui/src/lib/components/sidebar/bible`

Current role:

- Fixed entity list/detail UI.
- `EntityDetail.svelte` is almost 600 lines and branches on `EntityDetails`.
- Extraction review applies entity/snapshot results to the old model.

Blast radius:

- Story bible graph UI.
- Schema editor.
- Asset/reference display.
- Relationship editing.

Anti-patterns:

- Expanding fixed category forms for new worldbuilding types.
- Making a generic mega-form that branches on every possible schema.
- Treating extraction review as final mutation instead of proposal review.

Simplification:

```text
BibleNavigator.svelte
BibleNodeDetail.svelte
BiblePartEditor.svelte
BibleFieldEditor.svelte
BibleEdgeList.svelte
BibleSnapshotTimeline.svelte
BibleSchemaEditor.svelte
BibleAssetPanel.svelte
```

Reasoning and maintainability:

- Default and custom schemas use the same UI path.

Performance:

- Detail views load one node/part projection, not the whole entity list.

## `ui/src/lib/components/timeline`

Current role:

- DOM/SVG timeline renderer.
- `Timeline.svelte`, `StoryNodeClip.svelte`, and related components own scroll/zoom, hit testing, drag/resize/split, relationship drawing, labels, and gaps.
- Some interactions use direct document listeners and geometry reads.

Blast radius:

- Bottom timeline stack.
- Timeline store.
- Timeline API calls.
- Relationship layer.
- Character timeline.
- Accessibility command alternatives.

Anti-patterns:

- Maintaining DOM/SVG timeline and Bevy timeline as parallel supported renderers.
- Reusing `timelineState` as both Svelte renderer state and Bevy scene state.
- Letting Bevy commit local state before backend confirmation.
- Leaving critical timeline operations inaccessible outside canvas/Bevy.

Simplification:

```text
BevyTimelineHost.svelte
TimelineToolbar.svelte
TimelineA11yCommandList.svelte
TimelineInspectorPanel.svelte
timelineRenderProjection.ts
```

Reasoning and maintainability:

- Svelte owns command controls and panels; Bevy owns realtime viewport rendering and hit testing.

Performance:

- Dense clips, overlays, relationship curves, emotional graphs, and animation move out of DOM layout.
- Bevy receives compact projection diffs instead of the whole Svelte timeline state.

## `ui/src/lib/components/relationship`

Current role:

- 2D SVG relationship graph derived from fixed entities and character-specific relation fields.

Blast radius:

- Relationship panel.
- Bible graph projection.
- Bevy graph renderer.
- Selection/detail integration.

Anti-patterns:

- Keeping 2D SVG and Bevy graph as supported graph views.
- Rendering all canonical graph data rather than a bounded projection.
- Treating graph visualization as the only way to inspect/edit facts.

Simplification:

```text
BevyBibleGraphHost.svelte
BibleGraphFilters.svelte
BibleGraphInspector.svelte
BibleGraphA11yCommandList.svelte
```

Reasoning and maintainability:

- Graph visualization stays an exploration surface. Structured editors own mutations.

Performance:

- Neighborhood render projections keep large worlds tractable.

## `ui/src/lib/components/layout`

Current role:

- `AppShell.svelte` composes sidebar, editor, timeline stack, relationship panel, undo/redo, save, export, and command shortcuts.

Blast radius:

- Global shell state.
- Bevy host mounting.
- Projection loading.
- Undo/redo and review panel placement.

Anti-patterns:

- Moving orchestration state machines into `AppShell`.
- Letting the shell coordinate propagation or renderer bridge lifecycle.

Simplification:

- Keep shell as layout only.
- Use focused stores and backend projections for workflow state.
- Mount Bevy hosts through isolated bridge components.

Reasoning and maintainability:

- Layout changes stay separate from domain workflow changes.

Performance:

- Shell rerenders stay cheap when heavy projections live in focused child surfaces.

## Tests And Verification

Current role:

- Existing tests are mostly unit-level in core modules, validation, diffusion, and a few UI stores/API areas.
- There is no current test surface for event history, projection rebuild, script document ownership, Bevy bridge contracts, or graph schema invariants.

Blast radius:

- New tests are required across core, server integration, UI, bridge, and performance layers.

Anti-patterns:

- Preserving old compatibility tests for models intentionally deleted by the plan.
- Asserting implementation hops instead of command/projection behavior.
- Relying on typecheck as a substitute for vertical slice acceptance.

Simplification:

```text
core unit tests: validated types, graph invariants, script patching
server integration tests: SQLite transactions, replay/recovery, command idempotency
frontend tests: command forms, accessibility, projection rendering
bridge tests: Bevy/Y.Doc/WebSocket message validation and teardown
acceptance tests: minimum end-to-end vertical slices
benchmarks: projection rebuild and graph/dependency queries
```

Reasoning and maintainability:

- Tests become executable documentation for the new architecture.

Performance:

- Benchmarks catch graph/query/projection regressions before Bevy and AI amplify them.

## Recommended Sequencing

1. Create command/event/projection contracts first.
2. Add persistence/history foundation with idempotent transactional writes.
3. Replace the story bible model.
4. Replace script ownership.
5. Replace AI mutation paths with proposal paths.
6. Add Bevy render projections and hosts.
7. Delete DOM/SVG timeline and 2D relationship graph after Bevy target interactions are covered.

Do not start with Bevy. The renderer needs stable projections and backend command ownership first.

## Highest-Risk Current Anti-Patterns

- `NodeContent` says Y.Doc is the text source of truth, while the new plan requires backend SQLite command/event/revision state.
- `persistence.rs` stores important facts as JSON and rewrites whole tables.
- AI generation and extraction mutate node content and bible entities directly.
- Snapshot undo clones full `Project` state instead of tracking event/object revisions.
- Svelte stores and components mutate cached backend objects.
- DOM/SVG timeline components are large and own interaction semantics that will move to Bevy.
- TypeScript wire contracts are manual duplicates of Rust types.

## Highest-Value Simplification

The highest-value simplification is to make each user-visible surface consume one projection and emit one command family:

```text
Timeline viewport -> TimelineRenderProjection -> timeline commands
Script editor -> ScriptEditorProjection -> script commands
Bible sidebar -> BibleNodeProjection -> bible commands
3D graph -> BibleRenderGraph -> select/inspect/filter commands
AI review -> ChangeReviewProjection -> accept/reject/edit proposal commands
Export -> ExportProjection -> export command
```

That structure makes state ownership explicit, keeps Bevy and Svelte out of canonical state, and gives the backend a single history model for undo, redo, review, recovery, and propagation.
