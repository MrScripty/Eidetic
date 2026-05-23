# Architecture Blast Radius

This report compares the current codebase with the planning direction across story tracks, script generation, story bible graph, history, persistence, and Bevy rendering.

The important finding is that the plan should be treated as a replacement of the core mental model, not an additive layer.

No backwards compatibility is required. Old systems that no longer serve the target model should be removed, not preserved through adapter layers. Temporary bridge code is acceptable only when it directly supports an implementation step and has a planned deletion point.

Hard ownership requirements:

- There is exactly one source of truth for persistent project state.
- The source of truth is backend-owned and standards-compliant.
- Svelte and Bevy consume backend-confirmed projections and submit commands; they do not own canonical state.
- Bevy replaces the DOM/SVG timeline renderer. The target system must not keep a DOM/SVG timeline as a parallel or fallback renderer.

Current model:

```text
Project
  timeline.nodes[*].content owns script/outline text
  bible.entities owns story bible facts
  UndoStack owns short-lived cloned Project snapshots
  Svelte renders timeline and bible graph directly from REST payloads
```

Target model:

```text
Project
  timeline nodes are timed context chunks
  ScriptDocument owns generated screenplay text
  BibleGraph owns world/story/production facts
  ChangeEvent/ObjectRevision owns history
  read projections feed Svelte and Bevy renderers
```

The main risk is dual truth. If the new graph, script artifact, or event history is added beside the old `Entity`, `StoryNode.content.content`, Y.Doc node content, and snapshot undo stack, the system will become harder to reason about than it is now. Because compatibility is not a constraint, the preferred fix is deletion or replacement of the old ownership path.

## Current Architecture Touchpoints

### Core Timeline

Current files:

```text
crates/core/src/timeline/node.rs
crates/core/src/timeline/mod.rs
crates/core/src/timeline/track.rs
crates/core/src/template.rs
```

Current behavior:

- `StoryLevel` already supports `Premise -> Act -> Sequence -> Scene -> Beat`.
- `Timeline` stores a flat `Vec<StoryNode>` and uses `parent_id` for hierarchy.
- `StoryNode` owns `NodeContent`, including `notes`, `content`, `status`, and `scene_recap`.
- `locked` lives at node level.
- Templates currently create Premise, Acts, and Scenes directly, skipping Sequence in practice.

Plan impact:

- Keep the timed hierarchy and flat node list. This part is structurally compatible.
- Change node text meaning. `StoryNode.content.content` should stop being final screenplay text.
- Split node content into planning/context fields only.
- Move generated script to `ScriptDocument` / `ScriptSegment`.

Blast radius:

- `StoryNode`, `NodeContent`, and all helpers that call `node.content.content`.
- Timeline APIs that expose or mutate node script.
- AI generation request builders.
- Export, script panel, edit reaction, entity extraction, recap generation.

Anti-patterns to avoid:

- Keeping `StoryNode.content.content` as "sometimes outline, sometimes screenplay, sometimes generated artifact".
- Adding `ScriptDocument` while still using Beat nodes as the real script source.
- Letting node-level `locked` stand in for script-span locks. Node locks are too coarse.

Simplification opportunity:

- Make `StoryNode` narrow:

```text
StoryNode
  id
  parent_id
  level
  sort_order
  time_range
  title
  context_notes
  constraints
  status for context generation only
```

- Move all final screenplay lifecycle state into script tables.
- Keep timeline traversal logic where it is; it is already simple and useful.

## Script Model

Current files:

```text
crates/core/src/script/*
crates/server/src/routes/script.rs
crates/server/src/routes/ai.rs
crates/server/src/export.rs
ui/src/lib/components/editor/ScriptPanel.svelte
ui/src/lib/components/editor/ScriptView.svelte
ui/src/lib/components/editor/BeatEditor.svelte
```

Current behavior:

- Script text is stored on selected timeline nodes.
- `ScriptPanel.svelte` reads Beat-level nodes and treats each Beat as a script section.
- `ScriptView.svelte` parses raw text client-side for screenplay display.
- `routes/script.rs` replaces a node's entire `content.content` field.
- AI generation writes output to node content and mirrors it to Y.Doc.
- Export gathers Beat nodes and parses their `content.content`.

Plan impact:

- Add first-class `ScriptDocument`, `ScriptSegment`, `ScriptBlock`, and eventually protected spans.
- Script display should render script segments ordered by time, not Beat nodes.
- Manual script edits become structured script revisions and semantic claims.
- Regeneration becomes patch-based and span-aware.

Blast radius:

- The existing `/nodes/{id}/script` API should be replaced or narrowed to context notes only.
- `ScriptPanel.svelte` should be replaced with a script-document viewer/editor.
- `BeatEditor.svelte` should stop being both context editor and script editor.
- Export should consume `ScriptDocument`, not timeline Beat nodes.
- AI generation should create or patch script segments instead of overwriting node content.

Anti-patterns to avoid:

- A single raw text field for a full segment if the app needs locks, user-authored spans, and AI patches.
- Keeping both client and Rust screenplay parsers as independent sources of formatting truth.
- Letting a regeneration endpoint blindly replace a segment that contains user-authored spans.

Simplification opportunity:

- Keep the screenplay parser, but make it a projection from `ScriptBlock`s when possible.
- Use one canonical script write API:

```text
propose_script_patch
accept_script_patch
reject_script_patch
edit_script_block
lock_script_range
```

- Let the UI show formatted screenplay from a read model:

```text
ScriptDocumentView
  segments
  blocks
  spans
  provenance
  lock_state
  stale_state
```

## Story Bible

Current files:

```text
crates/core/src/story/bible.rs
crates/server/src/routes/story.rs
crates/server/src/routes/ai.rs
crates/server/src/prompt_format.rs
ui/src/lib/types.ts
ui/src/lib/api.ts
ui/src/lib/stores/story.svelte.ts
ui/src/lib/stores/bible.svelte.ts
ui/src/lib/components/sidebar/bible/*
ui/src/lib/components/relationship/*
```

Current behavior:

- `StoryBible` is `Vec<Entity>`.
- Categories are fixed: Character, Location, Prop, Theme, Event.
- `EntityDetails` is a fixed Rust enum.
- Snapshots are attached to entities.
- Snapshot overrides include fixed fields plus a small custom key/value escape hatch.
- Entity relations are simple labeled edges between entities.
- UI and API are strongly coupled to entity categories and detail variants.

Plan impact:

- Replace fixed `Entity` model with composable graph:

```text
BibleNode
BiblePart
BiblePartField
BibleEdge
BibleSnapshot
SemanticClaim
SemanticDependency
```

- The bible becomes a graph of world/story/production objects rather than a category enum.
- User-defined types and fields become schema rows.
- Semantic relations and story-world relations must be distinct.

Blast radius:

- The entire story bible domain model.
- Every `/bible/entities` route.
- Entity extraction and commit logic.
- Prompt formatting for bible context.
- Relationship graph UI.
- Entity detail, entity card, development timeline, extraction review.
- `RelationshipType::EntityDrives { entity_id }` needs to reference graph nodes or semantic dependencies.

Anti-patterns to avoid:

- Keeping `Entity` as the "real" bible object and adding generic graph nodes beside it.
- Building a dynamic UI by switching on old fixed categories.
- Storing custom fields as JSON blobs.
- Mixing story-world edges with semantic/provenance edges in the same relation type without a clear namespace.

Simplification opportunity:

- Delete category-specific entity update routes and replace with generic graph commands:

```text
create_bible_node
update_bible_node
create_bible_part
set_bible_field
create_bible_edge
set_bible_snapshot_field
attach_asset
```

- Keep default schemas for film-writing concepts, but implement them through the same schema system as user extensions.
- Generate category-like UI from schema definitions instead of hard-coded Svelte branches.

## Persistence

Current files:

```text
crates/server/src/persistence.rs
crates/server/src/state.rs
```

Current behavior:

- Project persistence is SQLite.
- Save rewrites the project into relational tables after clearing existing rows.
- Several fields are stored as JSON text: episode segments, node content, beat type, arc type, entity details, snapshot overrides, relationship type.
- Y.Doc state is stored as one BLOB.
- Runtime state is `Arc<Mutex<Option<Project>>>`.
- Undo is a stack of cloned `Project` snapshots.

Plan impact:

- SQLite remains the right storage engine.
- Canonical graph/script/history data should be relational rows.
- Object revisions should be delta-based and append-only.
- Current project state should be queryable without reconstructing from full snapshots.
- Undo/redo should operate on accepted change events and object revisions.

Blast radius:

- `SCHEMA_SQL` needs a major replacement.
- `save_project_sync` and `load_project_sync` should stop round-tripping the whole in-memory project as the main write path.
- `clear_all_tables` rewrite behavior conflicts with append-only history.
- `UndoStack` becomes obsolete for persistent history.
- `ydoc_state` must be scoped carefully if Y.Doc remains involved.

Anti-patterns to avoid:

- Putting the event log inside the in-memory `Project` and then serializing it wholesale.
- Combining append-only history with the current "clear all and rewrite" save strategy.
- Keeping current JSON fields for facts that need indexing and partial updates.
- Making every read reconstruct state by replaying the entire event log.

Simplification opportunity:

- Separate write model from read projections:

```text
canonical tables:
  bible_nodes
  bible_part_fields
  script_segments
  change_events
  object_revisions
  semantic_claims
  semantic_dependencies

read projections:
  timeline_view
  script_document_view
  bible_graph_view
  ai_context_view
  bevy_timeline_view
  bevy_bible_graph_view
```

- Keep current-state tables as canonical for fast reads, plus append-only revisions for history.
- Do not make event replay the only way to load the normal editor state.
- Use transaction boundaries around user-visible change proposals and accept/reject operations.

## History, Review, Undo/Redo

Current files:

```text
crates/server/src/state.rs
crates/core/src/ai/consistency.rs
crates/server/src/routes/ai.rs
ui/src/lib/components/editor/DiffView.svelte
```

Current behavior:

- Undo/redo is volatile and snapshot-based.
- Consistency reaction has no true before state; `previous_script` is currently empty.
- AI consistency suggestions target downstream node text snippets.
- Suggestions are not first-class events.

Plan impact:

- Every meaningful change becomes a `ChangeEvent`.
- AI work should produce proposed events before accepted events.
- Before/after diffs are computed from object revisions.
- Undo/redo reverses or replays accepted object revisions.
- Users can stop propagation, edit a proposed bible update, or rerun with new instructions.

Blast radius:

- All mutation routes need to emit change events.
- AI routes need proposal/accept/reject phases.
- The UI needs a review surface for change chains.
- WebSocket events need to identify changed objects and history events, not just "BibleChanged" or "NodeUpdated".

Anti-patterns to avoid:

- Treating AI propagation as one large mutation.
- Auto-committing AI bible changes after extraction without a reviewable semantic claim.
- Using generated natural language explanations as the only provenance.
- Implementing undo by restoring full SQLite backups.

Simplification opportunity:

- Introduce a small command/event vocabulary before adding advanced UI:

```text
Command
  EditScriptBlock
  ProposeSemanticClaim
  AcceptSemanticClaim
  SetBibleField
  ProposeScriptPatch
  AcceptScriptPatch
  UndoEvent
  RedoEvent

Event
  UserScriptEdited
  SemanticClaimProposed
  BibleFieldChanged
  ScriptPatchProposed
  ScriptPatchAccepted
```

- Keep proposed events separate from accepted events so review is first-class.

## AI Context And Propagation

Current files:

```text
crates/core/src/ai/*
crates/server/src/routes/ai.rs
crates/server/src/prompt_format.rs
crates/server/src/ai_backends/*
crates/server/src/vector_store.rs
crates/server/src/embeddings.rs
```

Current behavior:

- Generation context is built from target node, ancestors, siblings, bible entities, recaps, and RAG chunks.
- Generation output writes back to the node.
- Auto-extraction can commit entities/snapshots from generated text.
- Consistency reaction looks at downstream nodes by sibling order and causal relationships.

Plan impact:

- AI context should be assembled from active timed context chunks plus script/bible revision state.
- Entity extraction becomes semantic claim extraction.
- Propagation should query semantic dependencies, not only downstream siblings.
- AI output should be patches/proposals, not immediate writes.

Blast radius:

- `GenerateRequest` must stop carrying full `StoryNode` as the script target.
- Prompt formatting needs `ScriptSegmentContext` and `BibleGraphContext` projections.
- Existing extraction prompt should become a claim extraction prompt with scope and confidence.
- `downstream_node_ids` should be replaced by dependency queries.

Anti-patterns to avoid:

- Prompt builders reaching directly into full project structs.
- AI routes mutating project state after parsing model output without producing reviewable proposals.
- Using entity name matching as the primary dependency model.

Simplification opportunity:

- Build AI context from explicit query results:

```text
GenerationContext
  target_time_range
  active_timeline_context
  active_bible_facts
  relevant_script_segments
  protected_spans
  dependency_revisions
  references
```

- Keep context construction deterministic and testable.
- Use AI only for interpretation/generation, not for deciding persistence side effects directly.

## Y.Doc / Collaborative Text

Current files:

```text
crates/server/src/ydoc.rs
ui/src/lib/yjs.ts
ui/src/lib/ws.ts
ui/src/lib/components/editor/BeatEditor.svelte
```

Current behavior:

- Y.Doc schema is node keyed:

```text
nodes.{node_id}.notes
nodes.{node_id}.content
project_text.premise
```

- Node content is mirrored into Rust `Project` and persisted as a Y.Doc BLOB.
- Y.Doc provides attributed spans by author, but not currently script-block semantics.

Plan impact:

- If Y.Doc remains, it should manage collaborative text buffers for script documents/blocks, not node script content.
- Script spans, locks, and provenance need stable IDs outside raw CRDT text.
- Y.Doc updates need to become change events or feed change events.

Blast radius:

- Y.Doc schema.
- WebSocket sync assumptions.
- BeatEditor subscription logic.
- Save/load Y.Doc state.
- Script locking and patch application.

Anti-patterns to avoid:

- Keeping Y.Doc as the hidden source of truth while SQLite stores script revisions as another source of truth.
- Trying to infer durable semantic history from raw CRDT updates after the fact.
- Storing locked spans only as Y.Text formatting attributes.

Simplification opportunity:

- Decide one clear ownership model:

```text
Option A:
  SQLite script blocks/spans are canonical.
  Y.Doc is an editing transport/cache for currently open text.

Rejected option:
  Y.Doc is canonical for text.
  SQLite stores only object IDs, semantic annotations, and revision anchors.
```

The rejected option violates the backend-owned single-source-of-truth requirement and makes history, undo/redo, and AI propagation harder to verify. SQLite script blocks/spans should remain canonical if Y.Doc is retained.

## Frontend State And UI

Current files:

```text
ui/src/lib/types.ts
ui/src/lib/api.ts
ui/src/lib/stores/*
ui/src/lib/components/editor/*
ui/src/lib/components/sidebar/bible/*
ui/src/lib/components/timeline/*
ui/src/lib/components/relationship/*
ui/src/lib/components/layout/*
```

Current behavior:

- Frontend types mirror current Rust DTOs.
- `storyState.entities` and `timelineState.timeline` are the main read stores.
- `BeatEditor.svelte` is a large mixed surface: node notes, script display/editing, generation, extraction, linked entities, consistency suggestions.
- `EntityDetail.svelte` is category-specific and already called out as too large in its README.
- `ScriptPanel.svelte` is Beat-node based.

Plan impact:

- Frontend type model must be replaced with graph/script/history DTOs.
- Stores should separate command state, read projections, and renderer view state.
- Editor surfaces need to split by object:

```text
TimelineContextEditor
ScriptDocumentEditor
BibleGraphPanel
ChangeReviewPanel
PropagationInspector
```

Blast radius:

- Nearly all UI feature components.
- API wrapper.
- WebSocket handlers.
- Selection state.
- Existing bible sidebar.
- Existing relationship graph.

Anti-patterns to avoid:

- Expanding `BeatEditor.svelte` into the new script editor and propagation inspector.
- Expanding `EntityDetail.svelte` into a dynamic schema editor.
- Passing full graph objects through every component instead of using focused view models.
- Letting Bevy and Svelte both own selection/hover/drag state.

Simplification opportunity:

- Create view-model stores that mirror backend projections:

```text
timelineViewState
scriptDocumentState
bibleGraphState
changeReviewState
bevyTimelineBridgeState
bevyBibleGraphBridgeState
```

- Keep Svelte command surfaces declarative.
- Let Bevy own only realtime scene interaction and emit high-level commands.

## Bevy Timeline Renderer

Current files to replace or shrink:

```text
ui/src/lib/components/timeline/Timeline.svelte
ui/src/lib/components/timeline/LevelTrack.svelte
ui/src/lib/components/timeline/StoryNodeClip.svelte
ui/src/lib/components/timeline/RelationshipLayer.svelte
ui/src/lib/components/timeline/RelationshipArc.svelte
ui/src/lib/components/timeline/TimeRuler.svelte
ui/src/lib/components/timeline/StructureBar.svelte
ui/src/lib/components/timeline/CharacterTimeline.svelte
ui/src/lib/stores/timeline.svelte.ts
```

Current behavior:

- DOM/CSS renders tracks and clips.
- SVG renders relationship arcs.
- Svelte store owns zoom, scroll, playhead, selection, snapping, drag state.

Plan impact:

- Bevy should own the floating timeline renderer window, hit testing,
  drag/resize, overlays, arcs, and dense realtime visuals.
- Svelte should own shell panels and command forms.

Blast radius:

- Build system and asset pipeline.
- Desktop floating renderer window strategy.
- Timeline interaction contract.
- UI test strategy.
- Accessibility command alternatives for operations that move into canvas/WebGPU/WebGL.

Anti-patterns to avoid:

- Keeping DOM clip rendering active while Bevy also renders clips.
- Letting Bevy directly mutate project state.
- Sending full project data to Bevy every frame.
- Coupling Bevy systems to REST response shapes.

Simplification opportunity:

- Define a compact `TimelineRenderModel`:

```text
tracks
clips
relationships
arcs
markers
overlays
selection
viewport
```

- Bevy consumes render models and emits commands:

```text
SelectNode
MoveNode
ResizeNode
SplitNode
CreateRelationship
SetPlayhead
SetViewport
```

- Backend/Svelte remain responsible for validation and persistence.

## Bevy 3D Bible Graph

Current files to replace or shrink:

```text
ui/src/lib/components/relationship/RelationshipGraph.svelte
ui/src/lib/components/relationship/RelationshipPanel.svelte
```

Current behavior:

- 2D SVG graph from entity relations.
- Nodes and edges are simple Svelte/SVG objects.

Plan impact:

- Bevy renders a 3D graph projection of bible nodes, fields, edges, semantic relationships, and dependency neighborhoods.
- Svelte owns filters, details, schema editing, and change review.

Blast radius:

- Current relationship panel.
- Story bible selection state.
- Graph layout code.
- Renderer integration.

Anti-patterns to avoid:

- Rendering the entire canonical graph when a projection/neighborhood would be enough.
- Mixing graph editing operations with camera/selection systems.
- Making 3D graph layout the only way to inspect facts.

Simplification opportunity:

- Use explicit graph projections:

```text
BibleGraphProjection
  visible_nodes
  visible_edges
  clusters
  selection
  labels
  semantic_dependency_overlay
```

- Keep canonical graph queries in backend/domain code.
- Bevy only receives enough data to render and interact.

## Codebase Section Impact Matrix

This section iterates the current codebase by implementation area and records the specific effect of the plan. It is intentionally more concrete than the architectural sections above.

### `crates/core/src/project`

Current role:

- `Project` aggregates `Timeline`, `Vec<StoryArc>`, `StoryBible`, and references.
- It currently makes `StoryBible { entities }` and timeline-owned text part of the central project shape.

Plan effect:

- `Project` should stop being a large mutable object that all routes clone, mutate, and rewrite.
- It should become either a lightweight project metadata aggregate or a read projection over canonical tables.
- Core project state should reference canonical submodels:

```text
ProjectMetadata
TimelineModel
BibleGraph
ScriptWorkspace
ChangeHistory
ReferenceLibrary
```

Anti-patterns to avoid:

- Expanding `Project` with `BibleGraph`, `ScriptDocument`, and `ChangeHistory` while leaving old `StoryBible` and node-owned script in place.
- Treating `Project` as the persistence unit for append-only history.

Simplification opportunity:

- Make `Project` a boundary DTO/read model for UI/bootstrap.
- Move mutations into command handlers that operate on focused repositories/services.

Reasoning and maintainability:

- A smaller project aggregate makes ownership clearer.
- It prevents every feature from depending on the whole world state.

Performance:

- Avoids cloning and serializing large project graphs for every mutation or undo entry.

### `crates/core/src/timeline`

Current role:

- Owns tracks, hierarchy, node timing, relationships, arc tags, and node content.
- Node traversal helpers are useful and already close to the target model.

Plan effect:

- Keep `StoryLevel`, `TimeRange`, track hierarchy, parent-child validation, and timeline relationship mechanics.
- Remove final screenplay ownership from `NodeContent`.
- Reframe nodes as timed context chunks with notes/constraints/status only.

Anti-patterns to avoid:

- Using `Beat` as the only script-bearing level.
- Keeping `ContentStatus::HasContent` as a proxy for generated screenplay state.
- Keeping `node.locked` as the mechanism for script edit protection.

Simplification opportunity:

- Replace `NodeContent` with a context-specific structure:

```text
TimelineContext
  notes
  instructions
  constraints
  recap
  generation_state_for_context_only
```

- Keep timeline mutations ignorant of script text and bible field changes.

Reasoning and maintainability:

- Timeline code becomes responsible for time and hierarchy only.
- Script behavior becomes testable without timeline mutation side effects.

Performance:

- Bevy timeline projections can be produced from compact timeline rows without shipping script text.

### `crates/core/src/story`

Current role:

- Owns story arcs, fixed `StoryBible`, fixed `EntityCategory`, fixed `EntityDetails`, snapshots, entity relations, and prompt text formatting helpers.

Plan effect:

- Replace fixed entity model with a composable graph domain.
- Keep story arcs, but make references to story-world objects point to `BibleNodeId` or semantic dependency IDs instead of `EntityId`.
- Split graph concepts by responsibility:

```text
bible/schema.rs
bible/node.rs
bible/part.rs
bible/field.rs
bible/edge.rs
bible/snapshot.rs
bible/semantic_claim.rs
bible/dependency.rs
bible/projection.rs
```

Anti-patterns to avoid:

- Recreating `EntityCategory` as a new hard-coded enum with more variants.
- Adding "custom fields" while keeping canonical details in Rust enum variants.
- Having `to_prompt_text` methods on graph nodes that directly decide AI context policy.

Simplification opportunity:

- Default screenwriting concepts should be seed schemas, not special Rust code paths.
- Prompt-facing text should be produced by an AI context projection service.

Reasoning and maintainability:

- Adding new user categories no longer requires changing Rust enums, TypeScript unions, Svelte forms, and persistence JSON together.

Performance:

- Field-level rows allow direct indexes for queries like "all rainy locations in this sequence" or "all script segments depending on this field".

### `crates/core/src/script`

Current role:

- Contains screenplay element parsing/formatting helpers.
- It does not currently own the generated script artifact.

Plan effect:

- Promote this area into the canonical script domain:

```text
ScriptDocument
ScriptSegment
ScriptBlock
ScriptSpan
ScriptPatch
ScriptLock
ScriptProjection
```

- Keep parser/formatter logic, but make it serve import/export/display projections rather than being the storage model.

Anti-patterns to avoid:

- Treating raw screenplay text as the only canonical representation while also needing span locks, provenance, and partial regeneration.
- Maintaining divergent screenplay parsing logic in Rust and Svelte.

Simplification opportunity:

- Centralize screenplay parsing/formatting in Rust and expose rendered/projection DTOs to the UI.
- Use script block IDs as stable anchors for edits, locks, semantic claims, and regeneration patches.

Reasoning and maintainability:

- Script behavior becomes local to one domain area instead of spread across Beat editor, AI routes, export, and Y.Doc.

Performance:

- Patch operations can update affected blocks/spans rather than rewriting whole Beat text blobs.

### `crates/core/src/ai`

Current role:

- Builds generation requests from full `Project`, `StoryNode`, siblings, entity bible context, and surrounding node content.
- Consistency logic currently has weak before/after state and uses downstream node heuristics.

Plan effect:

- Replace project-reaching context builders with projection-based context builders.
- `GenerateRequest` should carry explicit generation inputs rather than full `StoryNode`s.
- Consistency becomes semantic propagation over dependencies and claims.

Anti-patterns to avoid:

- Prompt builders querying arbitrary project internals.
- AI deciding persistence side effects directly.
- Using name matching and sibling order as the primary impact model.

Simplification opportunity:

- Define narrow AI DTOs:

```text
GenerationContextProjection
SemanticEditAnalysisInput
SemanticClaimProposal
ScriptPatchProposal
PropagationPlan
```

- Keep deterministic context packing testable without model calls.

Reasoning and maintainability:

- AI becomes a consumer/producer of proposals, not the owner of app state.

Performance:

- AI context can be assembled from indexed projections and dependency rows rather than scanning the full project.

### `crates/server/src/persistence.rs`

Current role:

- Owns schema creation, save, load, JSON migrations, v1/v2/v3 compatibility, clear-and-rewrite persistence, entity serialization, node serialization, and Y.Doc state persistence.
- It is already above a healthy responsibility threshold.

Plan effect:

- Split into focused repositories/adapters.
- Remove legacy compatibility and clear-and-rewrite behavior for canonical state.
- Add transactional command persistence and projection rebuild support.

Anti-patterns to avoid:

- Adding new graph/script/history tables into the same monolithic file.
- Continuing to serialize important canonical fields into JSON because it is convenient.
- Replaying event history for every normal read instead of maintaining current-state tables/projections.

Simplification opportunity:

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

Reasoning and maintainability:

- Each repository owns one set of tables and tests.
- Schema changes can be reviewed against one domain at a time.

Performance:

- Focused queries and indexes become easier to tune.
- Avoids whole-project rewrites on every small edit.

### `crates/server/src/state.rs`

Current role:

- Owns app state, undo stack, project mutex, AI config, generating set, save channel, Y.Doc channels, and broadcast events.

Plan effect:

- Replace `UndoStack` with persistent `ChangeEvent` undo/redo commands.
- Replace broad `Project` mutex mutation with command handlers and repositories.
- Introduce lifecycle managers for spawned/background work.

Anti-patterns to avoid:

- Keeping `Arc<Mutex<Option<Project>>>` as the central mutation path for new canonical data.
- Spawning background tasks without tracked handles and shutdown ownership.
- Storing process-local undo/redo for persistent history.

Simplification opportunity:

```text
AppState
  command_bus
  projection_broadcaster
  task_supervisor
  ai_generation_manager
  propagation_manager
  project_repository
```

Reasoning and maintainability:

- Runtime state becomes composition/root wiring rather than business state.

Performance:

- Smaller locks and DB transactions reduce contention around long AI and save operations.

### `crates/server/src/routes`

Current role:

- REST handlers directly validate some input, call `snapshot_for_undo`, mutate `Project`, emit coarse events, and trigger saves.
- Routes are organized by feature but still own too much business logic.

Plan effect:

- Routes should become thin boundary adapters:

```text
deserialize raw payload
validate/parse into command
submit command
return projection or accepted event id
```

- Existing entity/script routes should be deleted/replaced.

Anti-patterns to avoid:

- Inline validation logic duplicated across handlers.
- Route handlers performing graph/script mutation logic.
- Returning ad hoc `serde_json::json!` shapes for long-lived contracts.

Simplification opportunity:

```text
routes/commands.rs
routes/projections.rs
routes/assets.rs
routes/ai.rs as proposal endpoints only
```

Reasoning and maintainability:

- Command handlers become testable without HTTP.
- Wire contracts become explicit and easier to keep aligned with TypeScript.

Performance:

- Routes can return focused projections rather than whole project/timeline/entity lists after each mutation.

### `crates/server/src/routes/ai.rs` And `prompt_format.rs`

Current role:

- AI routes generate content, batch-generate, react to edits, extract/commit entities, generate recaps, and mutate project/Y.Doc state.
- Prompt formatting mixes story model knowledge and output instructions.

Plan effect:

- Split AI orchestration, prompt building, proposal parsing, and command committing.
- AI outputs should produce proposal objects first.

Anti-patterns to avoid:

- A model response directly committing bible/script state.
- One route module owning generation, extraction, recap, propagation, and persistence side effects.
- Prompt builders tied to old entity/node structs.

Simplification opportunity:

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

- Each AI workflow has a clear state transition and review boundary.

Performance:

- Projection-based context avoids large prompt assembly by scanning broad state.

### `crates/server/src/ydoc.rs`

Current role:

- Single-owner Y.Doc manager for node notes/content.
- Provides sync over WebSocket and serialized BLOB persistence.

Plan effect:

- Decide whether Y.Doc remains. If it remains, it should be editing transport/cache for script blocks or notes, not a second canonical script store.
- It needs lifecycle ownership, task handle tracking, and schema replacement.

Anti-patterns to avoid:

- Treating Y.Doc and SQLite as equal canonical stores for the same script text.
- Encoding lock/provenance only in Y.Text attributes.
- Inferring semantic edit history from CRDT deltas after the fact.

Simplification opportunity:

- Prefer SQLite canonical script blocks/spans, with Y.Doc attached only to active editor buffers.
- Convert accepted Y.Doc changes into explicit script commands/events.

Reasoning and maintainability:

- Durable history stays in the same event/revision model as the rest of the app.

Performance:

- Active collaborative editing can remain efficient without forcing every historical query through CRDT state.

### `crates/server/src/export.rs`

Current role:

- Exports screenplay PDF by collecting Beat nodes and parsing `content.content`.

Plan effect:

- Export should consume `ExportProjection` built from `ScriptDocument`.

Anti-patterns to avoid:

- Exporting from timeline nodes after script ownership has moved.
- Re-parsing raw strings when structured blocks/spans already exist.

Simplification opportunity:

- One export path:

```text
ScriptDocument -> ExportProjection -> PDF/Text/Fountain renderers
```

Reasoning and maintainability:

- Export becomes independent of timeline editing internals.

Performance:

- Export can stream ordered blocks and avoid scanning unrelated timeline nodes.

### `crates/server/src/diffusion`

Current role:

- Diffusion infilling currently operates against Y.Doc node content regions.

Plan effect:

- If retained, it must target script document blocks/spans or an active edit buffer with stable script-block anchors.

Anti-patterns to avoid:

- Region rewrite APIs based on raw character offsets into node content after script segments become canonical.
- Running diffusion as a side channel outside reviewable patch/proposal history.

Simplification opportunity:

- Treat diffusion output as a `ScriptPatchProposal`.

Reasoning and maintainability:

- All AI text modification paths share the same review, lock, and provenance model.

Performance:

- Infilling can be scoped to block/span ranges rather than whole nodes.

### `crates/server/src/vector_store.rs`, `embeddings.rs`, `reference.rs`

Current role:

- Reference/RAG support feeds generation prompts.

Plan effect:

- References become assets or reference nodes connected to bible/script/timeline objects.
- RAG should consume a reference projection and attach provenance/dependencies.

Anti-patterns to avoid:

- Reference material being included in prompts without recording dependency/provenance.
- Asset paths or URLs accepted without centralized validation.

Simplification opportunity:

- Align references with the bible asset model:

```text
ReferenceAsset
ReferenceChunk
ReferenceDependency
```

Reasoning and maintainability:

- Generated script can trace which reference materials influenced it.

Performance:

- Embedding search can be filtered by story time, linked nodes, or active project context.

### `ui/src/lib/types.ts` And `ui/src/lib/api.ts`

Current role:

- TypeScript manually mirrors Rust DTOs.
- API wrapper exposes entity/timeline/node-script endpoints.

Plan effect:

- Replace manual old DTOs with command/projection DTOs.
- Prefer generated or schema-backed DTOs for long-lived contracts.

Anti-patterns to avoid:

- Manually maintaining duplicate Rust and TypeScript wire contracts for the new command/event/projection layer.
- Keeping old entity/script functions as compatibility facades.

Simplification opportunity:

```text
api/commands.ts
api/projections.ts
api/assets.ts
api/ai-proposals.ts
types/generated.ts or schema-backed contracts
```

Reasoning and maintainability:

- Clear API grouping reinforces command/query separation.

Performance:

- API calls can fetch focused projections instead of whole entity/timeline payloads.

### `ui/src/lib/stores`

Current role:

- Stores hold timeline, story entities, bible selection, editor state, websocket handling, and project bootstrap state.

Plan effect:

- Split backend-owned projections from transient UI state.
- Selection that affects business logic must be backend-confirmed or represented as command input, not hidden persistent frontend state.

Anti-patterns to avoid:

- Storing canonical graph/script state in Svelte stores and mutating it optimistically.
- Letting Bevy, Svelte, and backend each own selection state.

Simplification opportunity:

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

- Store names and ownership make it clear which data is backend projection cache and which state is transient UI state.

Performance:

- Smaller stores reduce broad reactive invalidation after every backend event.

### `ui/src/lib/components/editor`

Current role:

- `BeatEditor.svelte` owns node notes, Y.Doc sync, script editing, generation, extraction, linked entities, and consistency suggestions.
- `ScriptPanel.svelte` renders all Beat-owned script.
- `DiffView.svelte` handles simple text replacement suggestions.

Plan effect:

- Replace with separate script, context, and review surfaces.

Anti-patterns to avoid:

- Growing `BeatEditor.svelte` into a universal editor for graph/script/AI review.
- Keeping text diff suggestions as the only review UI for semantic graph updates.

Simplification opportunity:

```text
editor/TimelineContextEditor.svelte
editor/ScriptDocumentEditor.svelte
editor/ScriptBlock.svelte
editor/ScriptLockControls.svelte
editor/ChangeReviewPanel.svelte
editor/SemanticClaimReview.svelte
editor/ScriptPatchReview.svelte
```

Reasoning and maintainability:

- Each component maps to one user task and one projection.

Performance:

- Editing one script block should not rerender the whole timeline or bible sidebar.

### `ui/src/lib/components/sidebar/bible`

Current role:

- Fixed entity list/detail UI with category-specific forms and development timeline.

Plan effect:

- Replace with schema-driven graph editing and detail projections.

Anti-patterns to avoid:

- Trying to render custom schemas through branches based on old `EntityCategory`.
- Expanding `EntityDetail.svelte` into a generic mega-form.

Simplification opportunity:

```text
bible/BibleNavigator.svelte
bible/BibleNodeDetail.svelte
bible/BiblePartEditor.svelte
bible/BibleFieldEditor.svelte
bible/BibleEdgeList.svelte
bible/BibleSnapshotTimeline.svelte
bible/BibleSchemaEditor.svelte
```

Reasoning and maintainability:

- Schema-driven UI allows default and user-defined structures through the same code path.

Performance:

- Detail panels can load one node/part projection instead of the whole entity list.

### `ui/src/lib/components/timeline`

Current role:

- Svelte DOM/SVG timeline renderer and interaction owner.

Plan effect:

- Replace main rendering with a backend-owned floating Bevy renderer window and
  Svelte launch/focus/status controls.
- Delete the DOM/SVG clip timeline as a renderer once the Bevy renderer window
  owns the target timeline visual surface.
- Keep Svelte only for toolbar, forms, side panels, and accessibility command alternatives.

Anti-patterns to avoid:

- Maintaining DOM/SVG clips and Bevy clips as parallel renderers.
- Treating the old DOM/SVG timeline as a fallback runtime path after the Bevy timeline is active.
- Reusing `timelineState` as both Bevy scene state and backend projection state.
- Reintroducing embedded WebView child-surface, WASM, local HTTP/WebSocket, or
  split-process renderer-sidecar paths as timeline production paths.

Simplification opportunity:

```text
timeline/TimelineRendererWindowControls.svelte
timeline/TimelineToolbar.svelte
timeline/TimelineA11yCommandList.svelte
timeline/TimelineInspectorPanel.svelte
```

Reasoning and maintainability:

- Renderer lifecycle is isolated from business command handling.

Performance:

- Bevy receives compact render diffs/projections, not full Svelte state.

### `ui/src/lib/components/relationship`

Current role:

- 2D SVG graph for fixed entity relations.

Plan effect:

- Replace with a floating Bevy 3D bible graph window plus Svelte filters,
  launch/focus/status controls, detail, and inspection.

Anti-patterns to avoid:

- Treating 3D graph as the only way to edit or inspect facts.
- Rendering the full canonical graph instead of a useful projection/neighborhood.
- Treating the floating Bevy window as a second graph state owner.
- Continuing the superseded embedded WebView child-surface path.

Simplification opportunity:

```text
relationship/BibleGraphRendererWindowControls.svelte
relationship/BibleGraphFilters.svelte
relationship/BibleGraphInspector.svelte
```

Reasoning and maintainability:

- Graph visualization stays read/interaction-focused; forms own edits.

Performance:

- Neighborhood projection keeps graph rendering bounded for large worlds.

### `ui/src/lib/components/layout`

Current role:

- `AppShell` composes sidebar, editor, script panel, right panel, and bottom timeline stack.

Plan effect:

- Layout should compose new projection-driven surfaces and Bevy hosts without becoming the owner of domain workflow state.

Anti-patterns to avoid:

- Moving orchestration logic into `AppShell`.
- Letting the shell coordinate propagation state machines.

Simplification opportunity:

- Keep `AppShell` as layout only.
- Route workflow state through backend projections and focused stores.

Reasoning and maintainability:

- Layout changes remain separate from domain model changes.

Performance:

- Shell rerenders can stay low-cost if heavy projections live in focused child surfaces.

### WebSocket/Event Layer

Current role:

- Broadcasts coarse events like `TimelineChanged`, `NodeUpdated`, `BibleChanged`, generation progress, and Y.Doc binary updates.

Plan effect:

- Add event/projection-specific notifications:

```text
command_accepted
proposal_created
projection_updated
change_event_committed
script_segment_updated
bible_node_updated
render_model_updated
```

Anti-patterns to avoid:

- Emitting only coarse invalidation events that force the frontend to refetch everything.
- Mixing Y.Doc binary sync with durable semantic event history.

Simplification opportunity:

- Use typed event envelopes with explicit object IDs and projection versions.

Reasoning and maintainability:

- Consumers know exactly what changed and can request the right projection.

Performance:

- Reduces broad refetches and unnecessary UI/Bevy projection rebuilds.

### Tests And Verification

Current role:

- Existing tests are mostly unit-level and tied to current entity/node behavior.

Plan effect:

- Replace obsolete tests with command/projection/event tests.
- Add vertical slice tests for each new cross-layer path.

Anti-patterns to avoid:

- Preserving old compatibility tests for models we intend to delete.
- Asserting internal implementation hops instead of command/projection behavior.

Simplification opportunity:

```text
core unit tests: validated types, graph invariants, script patching
server integration tests: SQLite transactions, replay/recovery, command idempotency
frontend tests: command forms, accessibility, projection rendering
acceptance tests: minimum end-to-end slices
benchmarks: projection rebuild and graph/dependency queries
```

Reasoning and maintainability:

- Tests document the new architecture instead of anchoring old behavior.

Performance:

- Benchmarking hot projections catches scaling regressions before Bevy or AI workflows amplify them.

## Performance Implications

Good performance outcomes if implemented cleanly:

- SQLite row-level facts avoid full JSON parsing for common trait reads.
- Dependency queries can target indexed semantic tables.
- Script regeneration can be limited to affected ranges.
- Bevy can handle dense timeline visuals better than DOM/SVG.
- Read projections can avoid sending the whole project to every UI surface.

Performance risks:

- Event replay on every load or every query.
- Sending full bible graph/script document to Bevy on every small edit.
- Storing every text keystroke as a heavyweight event.
- Using row-per-field without proper indexes.
- Maintaining both Y.Doc and SQLite script state without a clear owner.

Recommended indexes:

```text
bible_nodes(parent_id)
bible_nodes(type_id)
bible_parts(node_id, kind)
bible_part_fields(part_id, field_key)
bible_part_fields(field_key, value_text)
bible_edges(from_node_id, kind)
bible_edges(to_node_id, kind)
bible_snapshots(node_id, at_ms)
script_segments(document_id, start_ms, end_ms)
semantic_dependencies(source_kind, source_id)
semantic_dependencies(target_kind, target_id)
semantic_claims(subject_kind, subject_id, predicate)
object_revisions(object_kind, object_id, change_event_id)
change_events(timestamp)
```

## Maintainability Recommendations

### 1. Replace old models instead of bridging

Because backwards compatibility is not required, delete or rewrite the old entity/script ownership paths early. Do not keep compatibility facades for old project schemas, old entity APIs, or old Beat-owned script behavior unless they are explicitly temporary implementation scaffolding.

Do not build:

```text
Entity + BibleNode
Beat.content + ScriptSegment
UndoStack + ChangeEvent history
DOM timeline + Bevy timeline
```

as parallel systems.

When a replacement lands, remove the obsolete path in the same phase:

```text
BibleGraph lands -> remove Entity/EntityDetails as canonical model
ScriptDocument lands -> remove Beat-owned screenplay text
ChangeEvent history lands -> remove Project snapshot undo as canonical undo
Bevy timeline lands -> remove DOM/SVG clip timeline renderer
Bevy bible graph lands -> remove 2D SVG relationship graph as canonical graph view
```

### 2. Introduce a command layer

Current routes mutate `Project` directly. The target model needs commands with explicit side effects.

Recommended shape:

```text
Command handler
  validates intent
  writes canonical rows
  writes change_events/object_revisions
  updates read projections
  emits websocket events
```

This makes AI, REST, undo/redo, and UI actions use the same mutation path.

### 3. Keep core domain independent from renderers

Do not add Bevy to `eidetic-core`.

Keep:

```text
eidetic-core       domain structs, commands, projections
eidetic-server     persistence, AI, HTTP/WebSocket
bevy renderer      realtime view consumption and interaction output
ui                 Svelte shell and forms
```

If a Bevy crate is added, it should depend on projection DTOs, not the whole server.

### 4. Use projections as boundaries

The plan becomes simpler if each surface consumes a purpose-built projection:

```text
AI context projection
Script editor projection
Bible detail projection
Timeline render projection
Bible 3D render projection
Change review projection
Export projection
```

This avoids leaking storage schema or full domain objects into every UI component.

### 5. Stage AI changes by default

AI should produce proposed semantic claims and proposed patches. Accepted proposals become project state. This prevents the AI from silently making bible/script changes that are difficult to unwind.

## Standards Compliance Constraints

This architecture must comply with the standards in:

```text
/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/Coding-Standards/
```

The standards-compliant execution plan for implementation now lives at:

```text
docs/refactors/eidetic-projection-architecture/final-plan.md
```

The implementation should treat the following as design constraints, not optional cleanup.

### Layering And Ownership

Required standards:

- `CODING-STANDARDS.md`
- `ARCHITECTURE-PATTERNS.md`

Implications:

- Keep dependencies pointing inward:

```text
Svelte / Bevy presentation -> application commands/projections -> domain -> infrastructure adapters
```

- `eidetic-core` should contain domain types, validated commands, projections, and pure services.
- `eidetic-server` should own runtime wiring, HTTP/WebSocket, AI backend calls, SQLite adapters, asset filesystem adapters, and task lifecycle.
- Bevy must be a renderer/interaction surface, not a state authority.
- Frontend and Bevy may own transient UI state only: hover, camera, drag preview, local form drafts, scroll/viewport, animation state.
- Backend remains the single source of truth for persistent project data and business decisions.
- Do not use optimistic updates for backend-owned state. UI should send commands and update after backend-confirmed projection changes.
- A projection may be cached in Svelte or Bevy only as a read model with a version. It must be replaceable from backend state at any time.

### File Size And Component Decomposition

Required standards:

- `CODING-STANDARDS.md`
- `FRONTEND-STANDARDS.md`

Implementation gates:

- Files over 500 lines require decomposition review.
- UI components over 250 lines require decomposition review.
- Modules/services with roughly more than 7 public functions or 3 responsibilities require decomposition review.
- Current oversized areas should be replaced with smaller target surfaces rather than expanded:

```text
crates/server/src/persistence.rs
crates/server/src/routes/ai.rs
crates/server/src/ydoc.rs
ui/src/lib/components/editor/BeatEditor.svelte
ui/src/lib/components/sidebar/bible/EntityDetail.svelte
```

Target UI decomposition should keep separate ownership for:

```text
TimelineContextEditor
ScriptDocumentEditor
BibleGraphPanel
BibleSchemaEditor
ChangeReviewPanel
PropagationInspector
TimelineRendererWindowControls
BibleGraphRendererWindowControls
```

### Command Layer And Correct-By-Construction Types

Required standards:

- `languages/rust/RUST-API-STANDARDS.md`
- `SECURITY-STANDARDS.md`

Implications:

- Parse external input once at the boundary into validated Rust types.
- Use newtypes/enums for IDs, object kinds, event states, lock states, time ranges, value types, command kinds, proposal states, and asset reference roles.
- Avoid stringly typed internal APIs for command names, object kinds, graph relation kinds, or lock states.
- Public fallible APIs should return typed errors, not `Result<T, String>`.
- Use private constructors when callers must go through validation.

Examples:

```text
Raw HTTP payload -> ValidatedCommand -> DomainCommandHandler
Raw path/URL -> ValidatedAssetRef
Raw field value -> BibleFieldValue
Raw time range -> TimeRange
Raw event id -> ChangeEventId
```

### Persistence And Event History

Required standards:

- `TESTING-STANDARDS.md`
- `SECURITY-STANDARDS.md`
- `languages/rust/RUST-SECURITY-STANDARDS.md`

Implications:

- SQLite writes for accepted commands must be transactional.
- Multi-step durable operations must be cancellation-safe, idempotent, or compensating.
- Event history and projections need replay/recovery/idempotency tests.
- Append-only history must not be implemented with the current clear-and-rewrite save strategy.
- Projections must have consistency checks after recovery.
- Queues for propagation, WebSocket updates, Bevy bridge events, or AI work must be bounded, with documented overflow behavior.
- User-supplied paths for project files, assets, imports, exports, and reference materials must be validated with a centralized path validator.
- URL references must be parsed and scheme-validated at the backend boundary.

Required persistence tests:

```text
per-field bible update writes only affected rows
script patch preserves locked/user-authored spans
event undo reverses only that event's object revisions
event redo reapplies only if base revision still matches
projection rebuild from persisted state matches current-state tables
duplicate command ids are idempotent
partial failure does not leave accepted event without required revisions
```

### Async, Tasks, And Concurrency

Required standards:

- `CONCURRENCY-STANDARDS.md`
- `languages/rust/RUST-ASYNC-STANDARDS.md`

Implications:

- Keep domain/core logic synchronous unless concurrent I/O is part of the contract.
- Async belongs at I/O/runtime boundaries: HTTP, WebSocket, SQLite blocking adapters, AI streams, filesystem, Bevy bridge, background workers.
- Every `tokio::spawn` must be owned by a lifecycle manager with tracked handles, cancellation, shutdown, panic logging, and draining.
- Do not discard spawned task handles.
- Do not hold `parking_lot::Mutex` guards across `.await`.
- Related mutable state should have one owner. Do not split a state machine across Svelte, Bevy, and server.

Target lifecycle owners:

```text
AiGenerationManager
PropagationManager
ProjectionRebuildManager
YDocBridgeManager, if Y.Doc remains
BevyBridgeManager
WebSocketSessionManager
```

Each owner should document:

```text
start condition
stop condition
cancellation path
queue capacity
overflow/backpressure behavior
shutdown behavior
panic/error handling
```

### Frontend And Accessibility

Required standards:

- `FRONTEND-STANDARDS.md`
- `ACCESSIBILITY-STANDARDS.md`

Implications:

- Svelte UI remains declarative except isolated Bevy/canvas/WebGL host components.
- Bevy host components must document direct DOM/canvas access and clean up lifecycle subscriptions deterministically.
- Every non-canvas command must use semantic HTML controls: buttons for actions, anchors for navigation, labels for inputs.
- Icon-only controls need accessible names.
- Canvas/Bevy interactions need keyboard-accessible command equivalents for critical actions.
- Timeline and graph navigation need focusable alternatives for selection, inspect, move, resize, lock, accept/reject proposal, undo/redo, and open detail.
- Embedded controls inside draggable/canvas containers need smoke checks for pointer capture/release, focus/blur, keyboard access, and parent gesture conflicts.

Accessibility cannot be deferred until after the Bevy renderer. The renderer bridge contract should include accessibility-facing commands and selected/focused object projection from the start.

### Interop And Wire Contracts

Required standards:

- `INTEROP-STANDARDS.md`
- `languages/rust/RUST-INTEROP-STANDARDS.md`
- `LANGUAGE-BINDINGS-STANDARDS.md`, if Bevy or other native views cross a generated binding boundary later

Implications:

- HTTP, WebSocket, Y.Doc bridge, and Bevy bridge payloads are trust boundaries.
- Validate messages at every process/language/runtime boundary.
- Use explicit serde attributes for public wire DTOs.
- Avoid duplicated TypeScript/Rust contract drift. Prefer a shared schema or generated DTOs for long-lived contracts.
- Add serialization round-trip tests for public command, event, and projection DTOs.
- Document thread requirements for Bevy bridge calls and callbacks.
- Event subscriptions across Svelte, WebSocket, Y.Doc, and Bevy must unsubscribe on teardown.

### Dependency Placement

Required standards:

- `DEPENDENCY-STANDARDS.md`
- `languages/rust/RUST-DEPENDENCY-STANDARDS.md`

Implications:

- Bevy is a heavy dependency and must not be added to `eidetic-core`.
- Bevy should live in a leaf crate or frontend-owned package that consumes projection DTOs.
- If Bevy adds 100+ transitive dependencies, it needs written justification and/or feature gating.
- Optional heavyweight features should be behind explicit Cargo features.
- Each crate/package must declare dependencies it directly uses.
- Feature contracts need `cargo check --workspace --all-features` and `cargo check --workspace --no-default-features`.

Recommended crate/package shape:

```text
eidetic-core          lean domain/contracts/projections
eidetic-server        runtime composition and infrastructure
eidetic-bevy-view     optional/leaf Bevy renderer, if native Rust renderer is added
ui                    Svelte shell and TypeScript host
```

### Documentation And Planning Artifacts

Required standards:

- `DOCUMENTATION-STANDARDS.md`
- `PLAN-STANDARDS.md`

Implications:

- The current `plans/` directory is a project-local planning area from this discovery session. Standards-compliant implementation planning should move or mirror final work plans under:

```text
docs/plans/<plan-slug>/
docs/refactors/<refactor-slug>/
docs/reports/<report-slug>/
```

- Large refactor execution plans must include objective, scope, milestones, task lists, verification per milestone, risks, re-plan triggers, and completion criteria.
- Directories under `src/` or equivalent source roots need README coverage when touched.
- README updates should include real contracts, invariants, alternatives rejected, dependencies, and revisit triggers.
- Public contract changes should have ADRs or README decision updates.

### Verification Gates

Required standards:

- `TESTING-STANDARDS.md`
- `TOOLING-STANDARDS.md`
- `languages/rust/RUST-TOOLING-STANDARDS.md`

Baseline verification for implementation slices:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --doc
cargo check --workspace --all-features
cargo check --workspace --no-default-features, once public features exist
npm run lint
npm run typecheck
npm run test
```

Cross-layer work must include a vertical slice acceptance test before broad horizontal expansion.

Required acceptance slices:

```text
create bible graph node -> set field -> query projection -> UI/API receives updated view
manual script edit -> semantic claim proposal -> accept bible field change -> affected segment marked stale
accepted script patch -> locked span preserved -> export projection includes final screenplay
undo accepted bible change -> graph projection reverts -> dependent script segment status updates
rebuild projections from persisted database after restart
Bevy timeline render model receives projection and emits a validated command
Bevy bible graph render model receives projection and emits selection/inspect command
```

Performance-sensitive graph queries, projection rebuilds, and renderer projection serialization should get Criterion benchmarks before making performance claims or locking regression budgets.

### Tooling And Lint Policy

Required standards:

- `TOOLING-STANDARDS.md`
- `languages/rust/RUST-TOOLING-STANDARDS.md`

Implications:

- Add or maintain workspace lint policy.
- Deny warnings in CI.
- Keep TypeScript strict/type-aware lint scoped to source files.
- Add persisted artifact validation hooks for schemas, projection fixtures, command fixtures, and sample events.
- If new generated contracts are introduced, generation must be deterministic and verified.

### Cross-Platform And Asset Handling

Required standards:

- `CROSS-PLATFORM-STANDARDS.md`
- `languages/rust/RUST-CROSS-PLATFORM-STANDARDS.md`
- `SECURITY-STANDARDS.md`

Implications:

- Use platform path APIs; never hardcode separators.
- Test path handling with spaces in paths.
- Validate asset paths under project asset roots.
- Keep platform-specific renderer or filesystem behavior behind platform modules/factories.
- If Bevy/native renderer support differs by platform, document supported platforms and graceful degradation behavior.

## Suggested Implementation Order

### Phase 1: Stabilize the target contracts

- Define new core structs for bible graph, script document, and change events.
- Define command/event vocabulary.
- Define projection DTOs.
- Decide Y.Doc ownership model.
- Mark obsolete canonical models for removal: `Entity`, Beat-owned screenplay text, snapshot undo, entity routes, and DOM/SVG renderers.
- Use and maintain the standards-compliant execution plan at `docs/refactors/eidetic-projection-architecture/final-plan.md` before code implementation begins.
- Add or update ADR/README contracts for command layer, projection boundaries, persistence ownership, and renderer ownership.
- Confirm workspace lint/test/tooling baseline before broad refactor work.

### Phase 2: Replace persistence foundation

- Add SQLite tables for graph/script/history.
- Stop using clear-and-rewrite saves for new data.
- Keep current-state tables plus append-only revision rows.
- Add tests for per-field update, before/after diff, undo/redo of one event.
- Remove persistence paths for old canonical bible/script state once replacement tables are active.
- Add replay/recovery/idempotency tests before adding higher-level AI propagation.
- Add centralized asset path and URL validators before asset/reference graph fields are exposed.

### Phase 3: Replace story bible

- Replace `Entity` routes and UI with graph routes and graph projections.
- Implement default schemas through the same system as user-defined schemas.
- Move extraction to semantic claim proposal.
- Delete category-specific `EntityDetails` editing and entity snapshot APIs after graph equivalents exist.

### Phase 4: Replace script ownership

- Add `ScriptDocument` and `ScriptSegment` persistence.
- Replace Beat-owned script panel with script-document editor/viewer.
- Move generation output into script segments.
- Add lock/protected-span behavior.
- Delete `/nodes/{id}/script` as a screenplay write path after script document commands exist.

### Phase 5: Replace propagation

- Replace current consistency reaction with semantic dependency analysis.
- Add proposed bible changes and proposed script patches.
- Add change review UI and undo/redo event operations.
- Remove cloned-project undo as the canonical undo/redo mechanism.

### Phase 6: Add Bevy renderers

- Start with read-only render projections.
- Add the shared floating renderer window host before graph/timeline native
  rendering.
- Retire or quarantine the superseded embedded WebView child-surface path before
  extending renderer behavior.
- Add selection and renderer status/focus sync through backend-owned command
  and projection paths.
- Add timeline editing commands.
- Add graph interaction commands.
- Remove the old DOM/SVG timeline renderer once Bevy projections cover the target timeline interactions.
- Do not keep a DOM/SVG timeline fallback after the Bevy timeline is active.
- Replace the 2D SVG relationship graph with the Bevy bible graph once graph projections cover the target interactions.
- Keep Bevy dependency isolated in a leaf crate/package with written dependency-cost justification.
- Keep platform-specific window behavior behind desktop runtime strategy
  modules, with typed status for unsupported capabilities.
- Add keyboard-accessible Svelte command alternatives and pointer/focus smoke checks for Bevy surfaces before shipping Bevy as the only timeline renderer.

## Main Design Risks

1. Dual truth between timeline node content, script document, and Y.Doc.
2. Dual truth between fixed entities and composable bible graph.
3. Undo/redo split between cloned project snapshots and event revisions.
4. Bevy becoming a second application state manager rather than a renderer.
5. JSON creeping back into canonical facts because it is quicker in the short term.
6. AI routes mutating graph/script state without user-reviewable proposal steps.
7. Large Svelte components growing further instead of being replaced by smaller surfaces.
8. Standards-compliance work being deferred until after implementation, causing large rewrites for testing, docs, accessibility, task lifecycle, or dependency placement.
9. Heavy Bevy/native dependencies entering `eidetic-core` or shared contracts instead of remaining isolated at a leaf/runtime boundary.
10. Event/projection recovery paths going untested, making undo/redo and review history unreliable after restart.

## Highest-Value Simplification

The strongest simplification is to make the system projection-driven.

Canonical data:

```text
timeline context chunks
bible graph rows
script document rows
change events and revisions
semantic dependencies
assets/references
```

Everything else is a projection:

```text
what the AI sees
what the screenplay editor sees
what the timeline renderer sees
what the 3D graph sees
what the change review panel sees
what export sees
```

This keeps the domain model stable while allowing Svelte, Bevy, AI prompts, and export to evolve independently.
