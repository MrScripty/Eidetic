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

Use Svelte for the surrounding application shell, panels, forms, and
inspectors, but move the main NLE/timeline visual surface to a realtime
renderer.

Bevy is the target renderer for the main timeline visual surface. The timeline
should not be implemented as a 2D overlay outside the scene. It should be real
3D Bevy geometry inside the same workspace renderer as the bible graph. Its
initial presentation mode can make it look like a flat panel by anchoring it to
the camera, but the same entities must be able to animate into a world-relative
3D NLE surface later.

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

- main timeline visual surface as 3D scene geometry,
- track and clip rendering,
- realtime interaction hit-testing,
- drag/resize/trim/split interactions,
- relationship arcs and graph overlays,
- arc visualization overlays,
- valence/arousal or other emotional graph overlays,
- dense markers and annotations,
- zoom/pan camera behavior,
- realtime visual effects and previews.

The target Bevy app is a workspace renderer rather than a separate graph window
plus separate timeline window. The graph scene, timeline scene, shared camera,
presentation controller, and input router live under one native renderer
lifecycle while continuing to consume backend-owned projections.

The server/core remains authoritative for project state. Bevy renders a
projection, then dispatches command intents through the backend-owned desktop
command/event path. The projection is a versioned read model that can be
discarded and rebuilt from backend state at any time.

## Timeline Scene Model

The Bevy scene should render a projection of timeline state, not replace the domain model.

Possible ECS components:

```text
WorkspaceRenderer
WorkspaceCamera
TimelinePresentationMode
TimelineRoot
TimelineViewport
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

`TimelinePresentationMode` should explicitly support:

```text
CameraAnchoredPanel
WorldAnchoredTimeline
Transitioning { from, to, progress }
```

In `CameraAnchoredPanel`, the timeline root transform is derived from the active
camera each frame so the timeline appears as a stable flat panel in the view.
In `WorldAnchoredTimeline`, the timeline root uses a normal world transform so
future graph/timeline visualizations can draw 3D edges to clips and place
objects inside clips. `Transitioning` interpolates the same timeline root and
material/depth policy between those modes; it must not swap to a different
renderer or duplicate timeline entities.

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
- Bevy runs as an app-managed floating native workspace renderer window, not as
  a browser/WASM canvas, not as a WebView-embedded child surface, and not as a
  renderer sidecar that owns business logic.
- The workspace renderer contains both the bible graph scene and the timeline
  scene. The timeline is rendered as real 3D geometry with a camera-anchored
  panel presentation mode first, not as a Svelte overlay, DOM layer, or separate
  timeline window.
- The floating renderer host is owned by the desktop runtime/composition root.
  Svelte may launch, focus, close, and display status for the renderer window,
  but must not own renderer lifecycle, durable timeline state, command queues,
  or projection subscriptions.
- The timeline scene must reuse the desktop-owned native renderer runner proven
  for the bible graph by moving into the shared workspace renderer. The runner
  boundary separates Tauri command/reply handling from the long-lived
  Bevy/winit event loop through bounded channels, so an open renderer window
  cannot block status, focus, close, or projection request commands.
- The desktop renderer host owns the active projection subscription. Svelte may
  request workspace/focus/filter changes and render accessible alternatives,
  but it must not push independent timeline or graph projections into Bevy in
  parallel with backend event refresh.

Rejected production paths:

- Bevy compiled to WASM and rendered into a Svelte canvas.
- Native Bevy child surfaces embedded/inset inside the Tauri WebView.
- Split-process native Bevy renderer IPC as the production renderer path.
- A separate production Bevy timeline window after the unified workspace
  renderer covers timeline rendering.

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
- Bevy, native renderer, and Tauri command/event dependencies must stay in a
  leaf crate/package and out of `eidetic-core`.
- Embedded WebView child-surface code, WASM bridges, local HTTP/WebSocket
  fallbacks, and split-process renderer-sidecar paths are not supported
  production timeline paths.
- Renderer window lifecycle must have one owner, tracked tasks/threads, bounded
  command queues, deterministic shutdown, panic reporting, and checked
  dimension arithmetic before allocation or hit testing.
- Platform-specific renderer/window behavior must live behind desktop runtime
  strategy modules, not in timeline business logic, projection adapters, or
  Svelte stores.
- Renderer bridge payloads are trust boundaries and must be validated on receipt.
- Renderer lifecycle must define initialization, teardown, event unsubscribe, cancellation, panic/error handling, and queue overflow behavior.
- Queues/events between backend, Svelte, and Bevy must be bounded.
- Projection delivery for a renderer window must have one active writer owned
  by the desktop host. Route mirroring, event refresh, and Svelte set-projection
  calls must not remain parallel writers for the same renderer state.
- Native runner work must not run Bevy/winit's blocking event loop inside a
  synchronous Tauri request handler or owner loop. Status and close commands
  must remain responsive while the renderer event loop is alive.
- Raw OS/window-handle dependencies are absent by default. If they are required,
  they belong only in the desktop runner/platform module and need an explicit
  safety and verification plan before the renderer becomes production.
- Canvas/Bevy critical actions require keyboard-accessible Svelte command alternatives.
- UI tests must cover keyboard alternatives, focus paths, pointer capture/release, parent gesture conflicts, and projection-to-command flow.
- The DOM/SVG timeline must be removed as a supported runtime path once Bevy covers target interactions.

## Open Questions

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
4. Prove the desktop-owned native renderer runner with the bible graph's
   minimal Bevy window gate before adding timeline visuals.
5. Consolidate renderer projection delivery into the desktop host so Svelte
   controls request focus/filter/open state but do not push renderer projections
   as a second writer.
6. Define any missing renderer-facing timeline projection DTOs around context
   chunks, overlays, and script-generation coverage.
7. Define the unified workspace renderer contract that combines bible graph,
   timeline, active context, playhead, and selection projections without moving
   durable state into Bevy.
8. Move timeline scene rendering into the workspace renderer as real 3D
   geometry under a `TimelineRoot`.
9. Add the camera-anchored panel presentation mode so the 3D timeline initially
   appears as a stable flat panel facing the camera.
10. Add the world-anchored timeline presentation mode and transition controller
   so the same timeline entities can animate into a world-relative 3D NLE.
11. Route workspace input through a priority hit-test router where timeline
   panel hits are handled before graph-world hits when the panel is visually on
   top.
12. Preserve existing pan/zoom/playhead, selection, hit-testing, move/resize/
   split/delete/create, relationship, and affect command paths through
   backend-confirmed projections.
13. Add script coverage/staleness overlays.
14. Add advanced overlays such as valence/arousal after backend-owned affect
    contracts exist.
15. Add keyboard-accessible Svelte command alternatives for timeline operations
    that cannot be directly represented in the Bevy viewport.
16. Remove the separate timeline renderer host once the workspace renderer
    covers timeline rendering.
17. Remove the DOM/SVG timeline renderer once the workspace renderer is active
    and accessibility alternatives are covered.

Current implementation progress:

- Completed: the desktop-owned floating Bevy timeline renderer can open, close,
  receive backend-confirmed timeline render projections, render tracks/clips/
  relationships/affect overlays/playhead, and emit validated selection,
  move/resize, split, delete, create-child, and create-relationship command
  intents through the Tauri/backend command bridge.
- Completed: native Bevy playhead keyboard navigation emits a validated
  transient `SetPlayhead` command through the renderer command queue. The
  desktop bridge records the backend-owned transient playhead, emits a
  `timeline_playhead_changed` event, refreshes active graph/timeline renderer
  projections, and the Svelte projection handlers update `timelineState`
  from the backend event instead of deriving graph context from unsynchronized
  Bevy-local state. Renderer-issued playhead commands are clamped against the
  current backend timeline projection before the event is emitted.
- Completed: the temporary Svelte timeline ruler and playhead now request
  playhead movement through a typed desktop command. Dragging uses local preview
  state only; committed `timelineState.playheadMs` updates still come from the
  backend-owned `timeline_playhead_changed` event. The desktop command clamps
  requested playhead positions against the current backend timeline projection
  instead of trusting frontend duration constants.
- Open: the temporary DOM/SVG timeline still exists as a behavioral reference
  and accessible command surface until Bevy covers the target interactions. Do
  not expand it as a parallel renderer or fallback source of timeline truth.
- Open: the existing separate native timeline renderer should be treated as
  transitional implementation work. The target product renderer is the unified
  Bevy workspace renderer with the timeline scene inside the graph/workspace
  window.
- Completed: the graph renderer crate has a first workspace projection boundary
  that can receive graph projection data and optional timeline projection
  metadata without changing current graph behavior.
- Completed: desktop graph renderer projection delivery now seeds and refreshes
  the workspace renderer with backend-owned graph and timeline projections
  together.
