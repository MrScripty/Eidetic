# Story Bible 3D Graph View Plan

This note captures the desired 3D visual graph view for Eidetic's story bible.

Reference implementation for visual direction and interaction feel:

```text
/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/whip-docs/
```

The relevant reference area is:

```text
src/lib/graph-v0/
```

The reference app uses Three.js. Eidetic should not copy that renderer choice. Use it as a reference for the 3D graph's spatial layout, focus behavior, selection feel, and graph readability, then implement the Eidetic version in Bevy with Eidetic's visual language.

## Goal

The story bible should have an optional 3D graph view for exploring worldbuilding structure.

The graph should help users see:

- canonical bible sections,
- parent/child hierarchy,
- relationship edges,
- clusters of related world elements,
- characters and their relationship neighborhoods,
- organizations and cultures,
- locations and contained objects/events,
- rules, motifs, themes, and set pieces connected to story nodes.

This should complement the normal sidebar/tree/detail editor. It should not replace structured editing forms.

## Current Status And Milestone Boundary

The current floating Bevy bible graph window proves native renderer lifecycle,
projection handoff, basic labels/colors, click selection, and simple panning.
It is not the target 3D graph experience yet. The current renderer still uses a
2D camera and sprite/text primitives, so it should be treated as Milestone 8
renderer-host infrastructure rather than a completed graph product.

The next graph milestone is the native 3D bible graph experience:

- replace the 2D/sprite proof with a true 3D Bevy scene,
- render structural and semantic edges clearly,
- make labels, selection, highlighting, and navigation usable,
- keep Svelte as the durable add/edit/remove/detail surface,
- make the graph useful before agent harness/tooling depends on it.

## Rendering Direction

Use Bevy for the 3D bible graph view.

The graph should be rendered by the same Bevy direction being considered for
the main realtime timeline renderer. The `whip-docs` Three graph is a visual/
interaction reference only. The production display target is an app-managed
floating native Bevy window, not a WebView-embedded child surface and not a
renderer sidecar that owns business logic.

Svelte should own:

- graph workspace controls, launch/focus/status UI, and a secondary semantic outline,
- filters and mode controls,
- selected-node detail panes,
- backend command dispatch through typed desktop/backend commands,
- floating renderer lifecycle requests and status display.

Bevy should own:

- 3D scene creation,
- camera controls,
- node/edge mesh lifecycle,
- labels,
- hit testing,
- selection visualization,
- graph neighborhood highlighting,
- local focus modes,
- animation and transitions.

Bevy should not own durable graph facts, accepted proposals, generation
selection, history, or saved project state. It emits typed command intents and
waits for backend-confirmed projections.

## Architecture Pattern From Reference App

The reference app separates graph behavior into pure helpers and an imperative scene class:

```text
types.ts
adapters.ts
layouts.ts
selectionIndex.ts
selection.ts
neighborhood.ts
sceneVisibility.ts
ThreeDirectoryGraphScene.ts
```

Eidetic should follow the same separation conceptually, but adapt it to Bevy/Rust ownership:

```text
story-bible-graph/
  render_graph.rs
  adapters.rs
  layouts.rs
  selection_index.rs
  neighborhood.rs
  visibility.rs
  scene.rs
  systems.rs
```

Key principles to keep:

- backend-owned bible graph state remains canonical,
- render graph adapters normalize backend/project data into a render-facing graph,
- layout math is pure and testable without Bevy,
- selection indexing is pure and rebuilt from graph/view state,
- the Bevy graph scene owns renderer resources/entities and cleans them up explicitly,
- Svelte does not directly create or mutate Bevy entities,
- selection/highlight changes restyle existing Bevy entities when possible,
- full geometry rebuilds are reserved for graph/layout changes.

The production boundary is native Rust Bevy under the desktop runtime. Do not
target a WASM bridge, WebView child-surface embedding path, or split-process
renderer sidecar as the production graph view. A split-process renderer
transport would require a separate standards review before being introduced.
The desktop composition root owns renderer window lifecycle. Svelte can request
launch/focus/close and render status projections, but it must not own renderer
threads, command queues, durable graph selections, or projection subscription
state.

## Render Graph Projection

The 3D graph should render a projection of the bible graph, not the persistence model directly.

Possible render-facing shape:

```text
BibleRenderGraph
  rootNodeId
  nodes
  edges
```

```text
BibleRenderNode
  id
  type_id
  title
  parent_id
  child_ids
  canonical_section
  tags
  importance
  summary
```

```text
BibleRenderEdge
  id
  kind
  from_node_id
  to_node_id
  weight
  direction
  confidence
  visible_at_detail
```

The adapter should be responsible for deciding what enters the render graph from:

- `bible_nodes`,
- `bible_edges`,
- `bible_parts`,
- canonical section roots,
- timeline node references,
- story arcs,
- asset references when relevant.

## Layout Modes

The graph should support multiple layout modes.

Recommended first layouts:

- `canonical-tree`: canonical roots as major branches.
- `radial-tree`: parent/child hierarchy radiating from selected root.
- `weighted-radial-tree`: larger/important subtrees get more space.
- `layered-grid`: canonical sections or hierarchy depth placed in layers.
- `relationship-neighborhood`: selected node centered with related nodes around it.

The reference app already has deterministic radial and layered-grid layout helpers. Eidetic can adapt that style but should use bible semantics rather than directory/file semantics.

The goal is a similar spatial experience, not identical code or colors.

## Visual Encoding

The graph should use Eidetic's color scheme and visual language, not the `whip-docs` palette.

Node visuals should communicate bible type and importance:

- canonical sections: large anchor nodes,
- characters: distinct color/material,
- locations: spatial/grounded color/material,
- organizations: grouped or faceted nodes,
- cultures: pattern or ring treatment,
- objects: smaller artifact nodes,
- world rules: constraint/lock-like visual language,
- motifs/themes: translucent or symbolic nodes,
- set pieces/events: timeline-accented nodes.

Edges should communicate relationship kind:

- parent/child containment,
- membership,
- ownership,
- conflict,
- alliance,
- causality,
- symbolism,
- location,
- constraint.

Edge visibility should be filterable by detail level so the scene does not become unreadable.

Visual tone should match Eidetic:

- dark neutral editor surface,
- restrained high-contrast accents,
- existing story arc colors where graph nodes/edges relate to arcs,
- bible category colors for node type accents,
- subdued distant nodes and edges,
- clear selected/hover/focus highlighting.

The initial usable graph must show structure even for a new or sparse bible.
Canonical roots and parent/child edges should render as a meaningful scaffold
so the graph does not appear as disconnected unexplained nodes before semantic
relationships are added.

## Interaction

Expected interactions:

- click node to select,
- click edge to select relationship,
- hover to preview title/summary,
- orbit/pan/zoom camera,
- frame selected node,
- focus on selected node's neighborhood,
- show first- and second-degree relationships,
- fade distant graph regions,
- toggle canonical sections,
- filter by node type,
- filter by edge kind,
- open selected node in the normal bible editor,
- create child node from selected node,
- create edge between selected nodes.

Keyboard interactions to consider:

- `.` frames selected node,
- arrow keys navigate parent/child/siblings,
- `Tab` enters or exits a focused neighborhood mode,
- `Esc` clears focus/selection.

Initial durable editing should stay in Svelte:

- add graph nodes,
- edit node fields and snapshots,
- add or remove graph edges,
- delete/archive nodes where supported,
- accept or reject AI proposals,
- inspect selected nodes, edges, context layers, and influence paths.

Direct Bevy editing can be added later only as backend command intents that are
validated and reflected back through projection updates.

## Selection And Hit Testing

Use the reference app's selection approach as a model:

- render an ID-map selection pass/buffer where practical,
- sample IDs on click,
- use depth/distance to choose the best hit,
- fall back to Bevy ray picking if ID-map selection misses or is not implemented,
- keep selection indexes separate from Bevy entity state.

The selection index should support:

- incident edges,
- adjacent nodes,
- first- and second-level neighborhoods,
- graph distance from selected node,
- visible-edge filtering so hidden edges do not affect highlight state.

Selection/highlight behavior should follow the reference graph's useful
patterns:

- selected node is visually primary,
- incident edges are highlighted,
- adjacent nodes remain prominent,
- second-level nodes can receive labels,
- distant unrelated graph regions are dimmed,
- edge and node selection both feed the same right-panel detail projection.

## Relationship To Composable Bible Model

The 3D graph view should consume the proposed composable bible graph:

```text
StoryBible
  schemas
  canon
  nodes
  parts
  edges
  snapshots
  assets
```

The view should not require every `BiblePart` to become a visible node. Parts usually appear as detail content for a selected node. A part should become a visible graph node only when it represents a meaningful world object or relationship target.

Default projection:

- `BibleNode` records become visible nodes.
- `BibleEdge` records become visible edges.
- canonical roots become anchor nodes.
- selected node parts appear in detail/sidebar UI.
- asset references appear as badges, thumbnails, or optional linked nodes.

## Relationship To Timeline

The 3D bible graph should be separate from the Bevy timeline renderer, but both should share the same story model.

Possible cross-links:

- select a bible node and show timeline nodes that reference it,
- select a timeline node and show its bible neighborhood,
- show temporal snapshots for a selected bible node,
- filter the graph to entities active at the current playhead time,
- show story arcs as graph overlays or colored edge/node accents.

The graph should support context/influence views:

- selecting a timeline clip highlights directly used bible nodes and edges,
- inherited premise/act/sequence context appears as softer highlights,
- selected context layers explain which parent context was distilled into the
  current clip,
- selected influence paths show why a graph fact affected generation or review.

## Standards Compliance Gates

Implementation must follow the standards plan in `docs/refactors/eidetic-projection-architecture/final-plan.md`.

Specific graph-view requirements:

- The 3D graph is a projection consumer, not a canonical graph store.
- Bevy may own transient renderer state such as camera, hover, animation, local layout simulation, and selection preview.
- Persisted graph facts, relationships, snapshots, asset refs, and user-adjusted layout data must be saved only through backend commands.
- Direct graph editing in Bevy, if added, must emit commands and wait for backend-confirmed projection updates.
- Renderer bridge payloads must be validated at every boundary and use explicit contract shapes.
- The floating Bevy graph window is a renderer-only projection surface. It may
  own camera, hover, animation, local layout simulation, and disposable ECS
  resources, but not durable graph facts, history, selection that affects
  generation, or persistence.
- Floating renderer lifecycle must have one desktop owner, bounded command
  queues, deterministic shutdown, panic reporting, checked dimension handling,
  and typed status for unsupported platform capabilities.
- Any platform-specific window behavior must live behind desktop runtime
  strategy modules, not in graph domain services, Svelte stores, or
  renderer-independent projection adapters.
- Bevy dependencies must stay in a leaf crate/package and out of `eidetic-core`.
- Layout math and projection adapters should be pure and testable without Bevy.
- Large graph rendering must use bounded projections/neighborhoods rather than sending the full canonical graph by default.
- Canvas/Bevy graph actions need keyboard-accessible Svelte command alternatives for selection, inspection, filtering, focus, and opening details.
- Renderer lifecycle must clean up subscriptions and renderer resources deterministically.
- The old 2D SVG relationship graph must be removed as a supported graph view once the Bevy bible graph covers target interactions.
- Normal graph selection must not mutate graph projection scope. Projection
  requests that include `selected_node_id` are reserved for explicit
  focus-neighborhood actions or backend-selected generation context, not default
  click selection.
- Category classification, canonical root coverage, and graph colors must be
  centralized in projection/contract data or a shared pure helper. Svelte and
  Bevy must not maintain divergent hard-coded category maps.
- Renderer geometry helpers must be pure and shared by rendering and picking so
  visible edges and hit testing cannot drift.
- Frequent projection updates must reuse renderer assets where practical. The
  native renderer must not keep creating unbounded mesh/material assets for
  every refresh.
- Projection construction should separate graph querying, scope selection,
  layout derivation, and DTO assembly so each step remains testable and easy to
  reason about.
- Frontend stores and Svelte components must remain projection/view-model
  consumers. They may own transient UI state such as open panels, draft form
  values, hover, focus, and camera command intent, but they must not own graph
  facts, graph selection that affects generation, persisted layout,
  command outcomes, or durable renderer capability state.
- Graph add/edit/remove flows must not use optimistic updates. The UI submits a
  backend command, waits for the backend-confirmed projection/event, then
  refreshes or receives the projected graph.
- Graph workspace synchronization should be event-driven through the existing
  backend/Tauri event bridge. Any polling or command-drain loop must be scoped
  to a single lifecycle owner, bounded, documented, and stopped deterministically.
- Every Tauri, renderer-command, and projection payload that crosses a process
  or runtime boundary must be parsed or validated at the boundary before domain
  code or renderer code trusts it.
- Rust implementation slices must keep pure layout/category/geometry helpers
  synchronous and framework-free. Async belongs at Tauri/database/event bridge
  shells, with blocking work isolated through owned blocking boundaries.
- Rust production paths must return typed errors for recoverable failures and
  avoid `unwrap`/`expect` in request, lifecycle, renderer host, and background
  bridge paths.
- Platform-specific renderer window behavior must compile through thin strategy
  modules for Linux, Windows, and macOS targets where practical. Graph domain,
  projection, and Svelte code must not contain inline platform branching.
- New or expanded modules should respect decomposition thresholds: keep UI
  components under the component-size target, split renderer files when they
  accumulate unrelated camera, geometry, material, lifecycle, and input logic,
  and add/update `README.md` files for non-obvious source directories.

Milestone 9 must not be considered complete until the graph is usable as a 3D
surface. Renderer lifecycle, projection delivery, and a 2D native proof are
necessary prerequisites, not the final graph.

## Usable MVP

The first product-complete slice should prove:

1. A new project opens with a meaningful canonical bible scaffold.
2. The Bevy graph uses a true 3D camera, 3D node geometry, and 3D edge geometry.
3. Structural parent/child edges and explicit semantic bible edges are both visible.
4. The user can add two nodes and one edge in Svelte and see them appear in the graph.
5. Clicking a node selects it and opens the normal bible detail/edit surface
   without collapsing the graph projection. Focused-neighborhood filtering is
   an explicit action, not the default click behavior.
6. Clicking an edge selects the relationship and shows edge detail.
7. Selected nodes highlight incident edges, adjacent nodes, and nearby labels
   while keeping non-selected graph context visible.
8. Node titles are readable by default for MVP-sized projections. Label-density
   culling can be added later, but the initial graph must not hide most titles.
9. Edges align in 3D and connect consistently to node centers or node surfaces.
10. Canonical roots, characters, places, objects, themes, events, and
    other/custom nodes have visibly distinct color identities. Selection and
    dimming must preserve category identity instead of flattening everything to
    one highlight color.
11. Orbit, pan, zoom, frame selected, reset/fit view, and clear selection are
    available through discoverable UI controls, not only hidden keyboard input.
12. Search/category/edge filters reshape backend projections rather than
    mutating renderer-local durable state.
13. Selecting a timeline clip or moving the playhead can highlight active
    graph/context influence.
14. Graph-local add/edit/remove controls exist for nodes and edges. These
    controls submit backend commands and refresh backend-confirmed projections;
    Svelte and Bevy do not mutate durable graph state locally.
15. Renderer window open/focus/close/recover behavior is obvious enough that a
    user can reopen the graph after closing or losing the floating Bevy window.

## Viewer Completion Requirements

Milestone 9 completion requires the 3D graph to function as a usable
viewer/editor, not just a render target.

### Selection And Scope

- Node selection and edge selection are transient UI state backed by projection
  requests/events; they do not own canonical graph facts.
- Selecting a node highlights the node, incident edges, adjacent nodes, and
  related labels while preserving the current graph scope.
- Graph workspace request builders must keep normal selected-node UI state out
  of the render projection request.
- Focused neighborhood mode is explicit. It may update the backend projection
  request with `selected_node_id`, but only when the user chooses a focus action.
- Focused neighborhood state must be modeled separately from selected-node
  inspection state so clearing focus does not also clear the selected detail
  panel unless the user asks for that behavior.
- Clear selection restores the visible graph context without requiring a graph
  reload workaround.

### Labels And Readability

- Every visible node in MVP-sized projections has a readable title.
- Selected, hovered, adjacent, and canonical root labels are always visible.
- Later density control may hide far-away or low-priority labels, but it must be
  deterministic renderer presentation state, not durable graph state.
- Label billboards must face the camera and avoid being placed so close to nodes
  that text overlaps the sphere.

### Edge Geometry

- Edge endpoints are computed in 3D using both XY and Z.
- Edges either connect center-to-center or are shortened to node surfaces using
  the current node radii; the behavior must be consistent and tested.
- Semantic edges and structural parent/child edges remain visually distinct.
- Edge picking uses the same 3D endpoint data that rendering uses.
- Edge rendering must not use 2D-only length/rotation math. A pure 3D segment
  helper should return the mesh length, midpoint, orientation, and selectable
  segment used by both rendering and hit testing.

### Color And Materials

- Category colors are explicit for:
  - canonical roots,
  - characters,
  - places/locations,
  - objects/props,
  - cultures,
  - events,
  - themes,
  - rules,
  - references,
  - other/custom nodes.
- Selection/highlight/dim transforms are material treatments layered over the
  category color. They must not erase category color identity.
- Unknown colors must not silently collapse important categories into a generic
  fallback.
- `prop`, `object`, and canonical object/root names must resolve to the same
  object/prop category. Cultures, rules, and references must not fall into
  "other" only because an older UI category list omitted them.

### Navigation And Recovery

- Graph workspace exposes visible controls for:
  - open renderer,
  - focus renderer,
  - close renderer,
  - clear selection,
  - focus neighborhood.
- The Bevy graph viewport must expose a typed camera-control API for workflow
  and agent-driven presentation actions:
  - fit graph,
  - frame node or edge,
  - frame active influence/context path,
  - reset camera,
  - navigate to node or neighborhood.
- Bevy supports mouse navigation for normal use: orbit, pan, and zoom.
- Keyboard navigation can remain, but the UI must show available actions through
  controls or concise labels.
- Renderer closed/error state should include a clear action to reopen or recover.
- Camera commands are transient renderer presentation commands. They may be
  issued by backend workflows, agent harnesses, or optional UI surfaces, but
  they must flow through a typed backend-owned renderer command boundary and
  must not mutate durable graph facts.
- Every Svelte control must use semantic controls with accessible names,
  visible focus states, and keyboard activation. Icon-only controls need
  `aria-label` or equivalent hidden text.
- Embedded graph controls must be verified against parent graph gestures so
  orbit, pan, zoom, drag, focus, and Escape behavior do not steal interaction
  from buttons, inputs, selects, sliders, or other controls.

### Graph Editing Workflow

- Graph workspace provides obvious node creation controls scoped by category or
  selected canonical root.
- Selected nodes can be edited through the normal backend-owned node detail
  surface from the graph workspace.
- Nodes can be removed through a confirmed backend command when deletion support
  exists. If delete is not yet implemented, the plan must record that blocker
  instead of presenting delete as available.
- Edges can be added, edited, and removed through backend commands and refreshed
  projections.
- Edge add workflows should use selectable source/target nodes or graph
  selections, not require users to manually type opaque node IDs.
- Creating or editing graph data must update the Bevy graph after the backend
  confirms the command and emits/returns the new projection.

### Renderer Performance

- The renderer may full-rebuild MVP projections, but reusable sphere/cylinder
  meshes and category/material palettes should be cached instead of recreated
  for every projection refresh.
- Per-projection updates should update transforms, visibility, and material
  handles before allocating new assets.
- Large graph support remains bounded by projection requests first; renderer
  optimizations are not a reason to send the entire bible graph by default.
- Performance-oriented caching is renderer-local presentation state only. It
  must not become a second source of graph truth.
- Do not make performance claims without measurement. If a slice claims
  improved refresh or interaction performance, add a repeatable benchmark,
  smoke measurement, or profiling note appropriate to the risk.

### Projection And Layout Structure

- Backend graph queries should return graph facts and influence facts.
- Scope selection should decide which nodes/edges are included for default,
  focused-root, explicit-neighborhood, search, and active-playhead cases.
- Layout derivation should be a pure helper with deterministic tests.
- DTO assembly should combine facts, layout, labels, categories, relationships,
  and influence metadata without embedding unrelated query or renderer concerns.
- Use correct-by-construction types for graph scope modes, renderer command
  kinds, camera command kinds, category identifiers, and node/edge IDs when
  invalid combinations would cross module or runtime boundaries.
- Prefer named enums over boolean parameters for modes such as default graph,
  focused root, focused neighborhood, selected context, and active playhead.
- Projection helpers must not depend on Bevy, Svelte, Tauri, or platform
  modules.

### Lifecycle And Concurrency

- The floating renderer host remains the single owner for native renderer
  lifecycle. Feature modules may request open/focus/close/status through typed
  commands but must not start independent windows, tasks, or timers.
- Background bridge tasks must have an owner that can stop them. Panics,
  cancellation, and closed channels must be surfaced through the existing
  diagnostics/status path.
- Command queues and event drains must remain bounded. Overflow behavior must
  be explicit and observable.
- Shared mutable state crossing threads must stay behind one owner or one lock
  per logically consistent state group. Do not split related renderer lifecycle
  fields across independent locks.
- Shutdown and close paths must be idempotent and deterministic.

### Boundary Contracts

- Rust serde attributes and TypeScript receiver types must be updated together
  in the same implementation slice for every renderer command, graph projection,
  category, and status payload change.
- Add serialization round-trip tests for new or changed tagged enums, string
  enums, optional/default fields, bounded numeric fields, and branded IDs.
- Boundary decoders should reject malformed commands, dimensions, IDs, and
  unsupported modes before they reach graph domain code or Bevy systems.
- Cross-process messages should carry stable action/type names and enough safe
  diagnostic context to identify the failed command, projection request, or
  renderer operation.

## Codebase Impact Corrections

The current implementation already has the renderer host and projection plumbing,
but the next slices must correct these code-level issues before Milestone 9 can
be called complete:

- `GraphWorkspacePanel.svelte` currently passes selected node state into
  `graphWorkspaceProjectionRequest`, and `bibleRenderGraphProjection.svelte.ts`
  turns that into `selected_node_id`. This couples click selection to backend
  graph scope and must be split.
- `bible_render_graph_query.rs` and `bible_render_graph_filter.rs` currently
  treat `selected_node_id` as a neighborhood query input. That behavior should
  remain only for explicit focus-neighborhood requests.
- `native_render.rs` stores Z coordinates for edges but computes edge length and
  rotation in XY only. Replace this with shared 3D segment math.
- `visual_3d.rs` hides most labels and replaces category color with one
  highlight color. It needs default MVP labels plus layered material state.
- `bibleGraphCategories.ts` covers fewer categories than the canonical bible
  roots. Category data should be derived from shared projection/contract data or
  kept in one tested helper used consistently by UI and renderer adapters.
- `BibleGraphEdgeEditor.svelte` requires manual target IDs. Graph-local edge
  creation should use selected nodes or a selectable node picker.
- Bible graph node/edge delete commands do not appear to exist yet. The UI must
  not present delete controls until backend commands and history projections are
  implemented.
- Camera control cannot be modeled as Svelte-owned local state. The current
  graph renderer command bridge is renderer-to-UI for selection/inspection
  commands; Milestone 9 still needs a typed backend-owned camera command API so
  backend workflows and agents can direct the Bevy viewport to fit the graph,
  frame important nodes/edges/influence paths, reset the camera, or navigate to
  a graph scope while preserving Bevy-local interactive navigation state.
- `native_render.rs` currently creates new mesh/material assets during each
  projection rebuild. Add renderer-local reusable assets before relying on
  frequent playhead/context refreshes.
- Existing graph/renderer files are already near or past decomposition review
  thresholds. Any slice that grows `native_render.rs`, graph workspace
  components, projection stores, or command bridges should first extract focused
  helpers/modules for geometry, materials, camera commands, category mapping,
  lifecycle/status, and request building rather than continuing to add mixed
  responsibilities to one file.

## Standards Compliance Checklist

Before implementing each remaining Milestone 9 slice, confirm:

- Backend remains the single source of truth for graph facts, graph mutations,
  persisted layout, history, and generation-affecting selection.
- Frontend and Bevy own only transient UI/renderer state and submit backend
  commands for durable changes.
- No optimistic updates are introduced for graph data. UI updates after command
  confirmation and backend projection refresh/event delivery.
- Runtime boundaries validate payloads and keep Rust/TypeScript contract shapes
  aligned in the same slice.
- Platform-specific behavior stays behind renderer strategy modules, not graph
  domain, projection, or Svelte code.
- Async/background work has one lifecycle owner, bounded queues, deterministic
  shutdown, and surfaced errors.
- Pure graph scope, category, layout, and 3D geometry helpers stay independent
  of Bevy, Tauri, Svelte, database, and platform modules.
- UI controls are semantic, labeled, keyboard-accessible, and tested against
  parent graph gestures.
- Decomposition thresholds are reviewed before expanding large renderer,
  projection, or component files.
- Verification includes the thinnest useful vertical slice through backend
  command/projection, Tauri bridge, Svelte control, and Bevy projection
  rendering before broader horizontal refactors.

### Acceptance Tests

- Automated tests cover:
  - selection does not add `selected_node_id` to the default graph projection
    request,
  - explicit focus-neighborhood action does add `selected_node_id`,
  - all MVP graph nodes get visible labels,
  - 3D edge endpoint math uses Z and aligns with node radii,
  - category color mapping covers all canonical categories,
  - selected/highlighted/dimmed material transforms preserve category identity,
  - UI and renderer category helpers classify every canonical bible root and
    supported node schema consistently,
  - rendered edge transforms and edge picking are produced from the same 3D
    segment helper,
  - renderer projection refreshes reuse stable mesh/material assets for common
    node and edge primitives,
  - renderer command application updates selection/detail state without owning
    graph facts,
  - add/edit edge and add/edit node flows submit backend commands then refresh
    projections,
  - no optimistic graph-node or graph-edge UI update is visible before backend
    command confirmation,
  - malformed renderer/projection boundary payloads fail with typed errors,
  - TypeScript command/projection types match Rust serde wire shapes for changed
    contracts,
  - renderer lifecycle tasks stop cleanly and queue overflow behavior is
    bounded and observable,
  - keyboard-accessible Svelte controls can perform the same selection,
    inspection, filtering, focus, and open/close actions as the renderer,
  - backend-issued camera commands can fit the graph, frame selected graph
    entities, reset the camera, and navigate to important graph scopes without
    requiring Svelte camera buttons.
- Manual smoke test covers:
  - new project opens canonical scaffold with labels,
  - user can open/focus/close/reopen the Bevy graph,
  - user can add two nodes and one edge from the graph workflow,
  - user can select a node without losing the rest of the graph,
  - user can select an edge and inspect its details,
  - user can navigate with mouse and visible controls,
  - user can reset/fit the camera after getting lost,
  - category colors are visually distinguishable.

## Open Questions

- Should the Bevy 3D graph and Bevy timeline share one floating renderer host
  with separate windows, or remain separate renderer owners behind the same
  command/projection contract?
- Should the graph use real force simulation, deterministic layout, or both?
- Should user-adjusted node positions be persisted?
- Should the graph support custom visual themes per project?
- How should media assets appear in 3D: icons, cards, billboards, or linked nodes?
- Should the graph allow direct editing, or only selection plus editor-panel editing?
- Should very large bibles use progressive loading or detail levels?

## Migration Notes

Recommended path:

No backwards compatibility with the current 2D relationship graph is required. It can remain a temporary reference during development, but the target graph view should be built directly around the composable bible graph.

The current floating 2D Bevy graph window is a renderer-host proof. Replace it
with the 3D graph rather than preserving it as a supported fallback.

Current implementation progress:

- Completed: renderer-neutral 3D visual snapshot boundary.
- Completed: native Bevy graph window uses a `Camera3d`, renderer-local light,
  mesh/material assets, 3D node meshes, 3D edge meshes, label billboards,
  ray-based node picking, pan/zoom camera movement, and derived structural
  parent/child edges from the 3D visual snapshot.
- Completed: semantic edge picking, native ECS selection/highlight/dimmed state,
  backend-derived label visibility, frame-selected camera navigation, and typed
  clear-selection commands.
- Completed: Graph workspace bootstrap ensures backend-owned canonical roots
  before loading render projections, so new projects show the canonical scaffold.
- Completed: native orbit camera navigation and native selected/highlighted/
  dimmed material styling.
- Completed: Graph workspace search and category controls request bounded
  backend render projections rather than filtering durable graph facts locally.
- Completed: native edge-selection commands are verified through the shared
  selection store into projection-derived edge detail.
- Completed: typed backend-owned camera commands can reset/fit/frame/navigate
  the Bevy graph viewport through the renderer owner, native runner, native
  window control, Tauri command boundary, and TypeScript API without making
  Svelte own durable camera or graph facts.
- Completed: Graph workspace node creation now uses the same backend-owned
  category create flow as the Bible tab, including schema validation,
  canonical-root ensure, backend create command, render projection refresh, and
  selection of the confirmed node projection.
- Completed: native renderer rebuilds now reuse renderer-local mesh and
  material assets for identical node radii, edge dimensions, and visual states
  instead of allocating fresh assets on every projection refresh.
- Completed: no separate old 2D bible relationship graph remains as a supported
  visual graph surface. Keep `BibleRenderGraphOutline` as the required
  keyboard-accessible projection alternative, not as a visual graph fallback.
- Completed: active playhead/clip context projections now use
  `active_timeline_ms`. The Graph workspace passes playhead time as projection
  request metadata, and the backend resolves active timeline clips plus their
  context influences before producing the bounded graph projection.
- Reopened: the graph is not product-usable yet. Implemented projection and
  renderer plumbing exists, but the current user-facing graph still fails the
  intended node editor/viewer experience.
- Code review findings to resolve before completion:
  - normal selection is coupled to backend projection scope,
  - edge rendering uses 2D math while picking uses 3D math,
  - label visibility is too restrictive for MVP graph reading,
  - category mapping/coloring is duplicated and incomplete,
  - graph-local add/edit workflows are incomplete and delete commands are not
    available,
  - visible navigation/recovery controls are incomplete,
  - renderer rebuilds allocate fresh assets on each projection refresh,
  - projection construction mixes scope, layout, and DTO assembly.
- Standards review findings to resolve before implementation completion:
  - prevent frontend/renderer ownership of backend facts or generation-affecting
    selection,
  - prohibit optimistic graph data updates,
  - keep graph synchronization event-driven except for the bounded renderer
    command drain owned by the desktop bridge,
  - validate and round-trip-test every changed runtime boundary contract,
  - keep platform-specific behavior inside renderer strategy modules,
  - keep async shells outside pure graph helpers,
  - add lifecycle, queue, shutdown, and panic/error verification for renderer
    background work,
  - add accessibility and parent-gesture conflict checks for graph controls,
  - split large mixed-responsibility renderer/projection/component files as part
    of the relevant implementation slices.
- Remaining usability gaps are tracked in "Viewer Completion Requirements" and
  must be completed before Milestone 9 is considered done.

1. Define `BibleRenderGraph` DTOs.
2. Add pure adapter from composable bible graph to render graph.
3. Add deterministic layout helpers and tests.
4. Add selection/neighborhood indexes and tests.
5. Add true 3D visual primitives: 3D camera, meshes, edge geometry, labels, and
   lighting using Eidetic's visual language.
6. Build a Bevy graph scene/plugin that consumes `BibleRenderGraph`.
7. Connect the Bevy graph scene/plugin to the shared floating renderer host.
8. Split normal graph selection from projection scope. Normal selection updates
   transient UI/detail state only; explicit focus-neighborhood requests update
   backend projection scope.
9. Add the vertical-slice acceptance path for selection/scope: backend
   projection request, Tauri renderer bridge, Svelte control, Bevy render
   refresh, and detail projection must all agree without optimistic updates.
10. Centralize graph category classification/color identity for canonical roots,
   supported schemas, custom schemas, and unknown nodes.
11. Replace 2D edge mesh placement with tested 3D segment geometry shared by
    rendering and picking.
12. Update labels and material transforms so MVP graph nodes are readable and
    highlight/dim states preserve category identity.
13. Add selection, edge selection, highlighting, and detail-panel integration
    without collapsing graph scope on normal selection.
14. Add orbit/pan/zoom, frame selected, reset/fit view, explicit focus
    neighborhood, clear selection, keyboard navigation, and visible controls for
    those actions.
15. Completed: add a typed backend-owned camera command API for the Bevy
    viewport.
    Commands must be transient renderer presentation commands, bounded through
    the existing renderer owner, callable by backend workflows and the future
    agent harness, and verified through Rust/TypeScript contract tests. Svelte
    camera buttons are optional and not required for this milestone.
16. Partially completed: add graph-local backend-command workflows for node and
    edge add/edit, using selectable graph/node controls rather than opaque ID
    entry. Graph workspace node creation now exists; edge creation and node/
    edge detail editing remain through the selected-node inspector. Delete
    remains blocked until backend delete commands exist and must not be
    simulated in frontend state.
17. Refactor projection construction into query, scope, layout, and DTO helpers
    with targeted tests.
18. Completed: add renderer-local mesh/material reuse for frequent projection
    refreshes.
19. Add lifecycle/queue/shutdown/error verification for renderer host,
    projection bridge, and command drain ownership.
20. Add filtering by canonical section, node type, edge kind, search, and
    active playhead/clip context.
21. Add timeline cross-linking and active-at-playhead filtering.
22. Remove the old 2D relationship graph once the 3D graph view is active.
