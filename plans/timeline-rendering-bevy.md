# Timeline Rendering Plan

This note captures the rendering direction for the NLE-style timeline.

## Current Renderer

The current timeline is rendered in Svelte using DOM/CSS layout with SVG overlays:

- DOM rows for tracks.
- DOM clips for story nodes.
- CSS absolute positioning for time ranges.
- SVG paths for relationship arcs.
- DOM/SVG marker overlays for character progression.

This is workable for a simple editor, but it is not the desired long-term rendering model.

## Direction Change

The main timeline should not be rendered as ordinary Svelte DOM.

The timeline is expected to update frequently and support visually complex realtime behavior that typical DOM timeline rendering will not handle well enough.

Use Svelte for the surrounding application shell, panels, forms, and inspectors, but move the main NLE/timeline viewport to a realtime renderer.

Bevy is the target renderer for the main timeline viewport.

Hard requirements:

- The backend is the only source of truth for persistent project state.
- Bevy receives backend-confirmed timeline render projections and emits validated commands.
- Bevy may own transient interaction state such as hover, drag previews, camera, selection preview, and animation state.
- Bevy must not own canonical timeline, script, bible, or history state.
- The DOM/SVG timeline renderer is replaced, not preserved as a fallback renderer.

## Why Consider Bevy

Bevy gives us:

- an ECS model that fits many timeline primitives,
- realtime rendering and animation,
- efficient update of many visual entities,
- camera/viewport abstractions for pan, zoom, minimap, and focus,
- custom drawing for clips, curves, overlays, graph paths, and heatmaps,
- Rust-side sharing with the existing domain model,
- a path toward richer visualizations without fighting DOM layout.

## Proposed Ownership Split

Svelte should own:

- app shell,
- sidebars,
- forms,
- text editors,
- story bible panels,
- AI controls,
- menus and modals,
- high-level command dispatch.

Bevy should own:

- main timeline viewport,
- track and clip rendering,
- realtime interaction hit-testing,
- drag/resize/trim/split interactions,
- relationship arcs and graph overlays,
- arc visualization overlays,
- valence/arousal or other emotional graph overlays,
- dense markers and annotations,
- zoom/pan camera behavior,
- realtime visual effects and previews.

The server/core remains authoritative for project state. Bevy renders a client-side projection, then dispatches commands back to the API or a future local command bridge. The projection is a versioned read model that can be discarded and rebuilt from backend state at any time.

## Timeline Scene Model

The Bevy scene should render a projection of timeline state, not replace the domain model.

Possible ECS components:

```text
TimelineViewport
TimelineCamera
TrackEntity
ClipEntity
ClipTimeRange
ClipHierarchyLevel
ClipArcTags
ClipSelectionState
RelationshipEdge
RelationshipCurve
Marker
OverlayLayer
HitTarget
```

The underlying data remains:

```text
Timeline
  tracks
  nodes
  node_arcs
  relationships
  structure
```

Bevy receives snapshots/diffs of that state and renders it into an interactive scene.

## Visual Layers

Expected render layers:

```text
Background grid
Time ruler
Structure bands
Track lanes
Clips
Clip text/icons
Arc color overlays
Relationship curves
Entity markers
Character/world markers
Valence/arousal overlays
Selection/hover/drag previews
Tool overlays
Playhead
```

Layering should be explicit so complex overlays can be toggled and composed without rewriting the base timeline renderer.

## Arcs

Story arcs should not remain limited to a single clip background color.

Bevy should support richer arc rendering, such as:

- primary clip color,
- multiple arc stripes or bands on a clip,
- arc lanes or ribbons,
- cross-track arc paths,
- convergence highlights,
- coverage heatmaps,
- filtered arc-only views.

Arc data should continue to come from `Timeline.node_arcs` and project `StoryArc` definitions.

## Valence/Arousal

Valence/arousal is not currently implemented, but the renderer should leave room for it.

Possible display forms:

- curve overlay above or below tracks,
- per-node emotional markers,
- heatmap across the timeline,
- character-specific emotional trajectory lane,
- relationship emotional trajectory overlay.

This should be model-backed, not purely visual. Likely data sources:

- node parts or annotations,
- story-bible snapshots,
- character/relationship state,
- AI analysis output,
- manually edited emotional beats.

## Integration Options

Production target:

- Tauri owns the standalone desktop application shell.
- Svelte remains the WebView UI shell for non-renderer panels and accessible
  command alternatives.
- Bevy runs as a native desktop renderer, not as a browser/WASM canvas.

Native integration choices may still vary by implementation slice:

1. Native Bevy viewport embedded/inset beside the Tauri WebView.
2. Split-process native Bevy renderer communicating over bounded local IPC.
3. Temporary separate Bevy window for early smoke validation only, not a final
   product UX.

Rejected production path:

- Bevy compiled to WASM and rendered into a Svelte canvas. Eidetic is a
  desktop application, and the renderer should use the native Bevy path.

## Interaction Contract

The renderer should emit high-level commands, not mutate persistent state directly:

```text
select_node(node_id)
move_node(node_id, start_ms, end_ms)
resize_node(node_id, start_ms, end_ms)
split_node(node_id, at_ms)
create_relationship(from_node_id, to_node_id, type)
set_playhead(time_ms)
set_viewport(start_ms, end_ms)
```

The app/server applies the command, persists it, and sends back updated timeline state.

## Standards Compliance Gates

Implementation must follow the standards plan in `docs/refactors/eidetic-projection-architecture/final-plan.md`.

Specific renderer requirements:

- Bevy is a presentation/interaction surface only. It consumes versioned projections and emits validated commands.
- Bevy must not own canonical timeline, script, bible, propagation, selection, or history state.
- Selection that affects business logic must be backend-confirmed or submitted as command input.
- No optimistic updates for backend-owned state. Bevy can show transient previews while dragging, but committed state changes only after backend confirmation.
- Bevy, native renderer, Tauri, or IPC dependencies must stay in a leaf
  crate/package and out of `eidetic-core`.
- Renderer bridge payloads are trust boundaries and must be validated on receipt.
- Renderer lifecycle must define initialization, teardown, event unsubscribe, cancellation, panic/error handling, and queue overflow behavior.
- Queues/events between backend, Svelte, and Bevy must be bounded.
- Canvas/Bevy critical actions require keyboard-accessible Svelte command alternatives.
- UI tests must cover keyboard alternatives, focus paths, pointer capture/release, parent gesture conflicts, and projection-to-command flow.
- The DOM/SVG timeline must be removed as a supported runtime path once Bevy covers target interactions.

## Open Questions

- Should the production native Bevy host be embedded/inset in the Tauri window
  or run as a split-process native renderer?
- How should text rendering be handled for dense clip labels?
- Should timeline hit-testing live entirely in Bevy?
- How should Bevy receive state updates: full snapshots, diffs, or command/event streams?
- How do accessibility and keyboard navigation work when the main timeline is not DOM?
- Should screenshots/export views use the Bevy renderer or a separate report/export pipeline?

## Implementation Notes

No backwards compatibility with the current DOM timeline is required. The existing Svelte timeline can be used as a behavioral reference while building the target renderer, but the target design does not preserve its component structure, data assumptions, or runtime fallback path.

Recommended path:

1. Add the Tauri desktop shell and move runtime startup/shutdown under the
   desktop lifecycle.
2. Remove the existing wasm-bindgen renderer bridges after native host
   contracts exist.
3. Upgrade renderer planning and crates to Bevy 0.18.1 with a fresh dependency
   review.
4. Define any missing renderer-facing timeline projection DTOs around context
   chunks, overlays, and script-generation coverage.
5. Prototype a native Bevy timeline viewport with read-only tracks/context
   chunks.
6. Add pan/zoom/playhead behavior.
7. Add selection and hit-testing.
8. Add move/resize/split command dispatch for context chunks.
9. Add relationship curves and arc overlays.
10. Add script coverage/staleness overlays.
11. Add advanced overlays such as valence/arousal after backend-owned affect
    contracts exist.
12. Add keyboard-accessible Svelte command alternatives for timeline operations
    that cannot be directly represented in the Bevy viewport.
13. Remove the DOM/SVG timeline renderer once the Bevy renderer is active.
