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

## Rendering Direction

Use Bevy for the 3D bible graph view.

The graph should be rendered by the same Bevy direction being considered for the main realtime timeline renderer. The `whip-docs` Three graph is a visual/interaction reference only.

Svelte should own:

- the panel/shell around the graph,
- filters and mode controls,
- selected-node detail panes,
- command dispatch,
- lifecycle and bridge management for the Bevy graph view.

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

Implementation language and boundary are still open. The graph helpers can live in Rust if Bevy runs natively, or be exposed through a WASM/IPC bridge if the app shell remains Svelte-first.

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

## Standards Compliance Gates

Implementation must follow the standards plan in `docs/refactors/eidetic-projection-architecture/final-plan.md`.

Specific graph-view requirements:

- The 3D graph is a projection consumer, not a canonical graph store.
- Bevy may own transient renderer state such as camera, hover, animation, local layout simulation, and selection preview.
- Persisted graph facts, relationships, snapshots, asset refs, and user-adjusted layout data must be saved only through backend commands.
- Direct graph editing in Bevy, if added, must emit commands and wait for backend-confirmed projection updates.
- Renderer bridge payloads must be validated at every boundary and use explicit contract shapes.
- Bevy dependencies must stay in a leaf crate/package and out of `eidetic-core`.
- Layout math and projection adapters should be pure and testable without Bevy.
- Large graph rendering must use bounded projections/neighborhoods rather than sending the full canonical graph by default.
- Canvas/Bevy graph actions need keyboard-accessible Svelte command alternatives for selection, inspection, filtering, focus, and opening details.
- Renderer lifecycle must clean up subscriptions and renderer resources deterministically.
- The old 2D SVG relationship graph must be removed as a supported graph view once the Bevy bible graph covers target interactions.

## Open Questions

- Should the Bevy 3D graph and Bevy timeline share one renderer/runtime, or remain separate views?
- Should the graph use real force simulation, deterministic layout, or both?
- Should user-adjusted node positions be persisted?
- Should the graph support custom visual themes per project?
- How should media assets appear in 3D: icons, cards, billboards, or linked nodes?
- Should the graph allow direct editing, or only selection plus editor-panel editing?
- Should very large bibles use progressive loading or detail levels?

## Migration Notes

Recommended path:

No backwards compatibility with the current 2D relationship graph is required. It can remain a temporary reference during development, but the target graph view should be built directly around the composable bible graph.

1. Define `BibleRenderGraph` DTOs.
2. Add pure adapter from composable bible graph to render graph.
3. Add deterministic layout helpers and tests.
4. Add selection/neighborhood indexes and tests.
5. Build a Bevy graph scene/plugin that consumes `BibleRenderGraph`.
6. Mount/bridge the Bevy graph view from a Svelte panel or shared Bevy viewport.
7. Add selection and detail-panel integration.
8. Add filtering by canonical section, node type, and edge kind.
9. Add timeline cross-linking and active-at-playhead filtering.
10. Remove the old 2D relationship graph once the 3D graph view is active.
