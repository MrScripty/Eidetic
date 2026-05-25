# Eidetic Bevy Bible Graph

Leaf Bevy integration for the story-bible graph renderer.

This crate consumes backend-owned `BibleRenderGraphProjection` snapshots from
`eidetic-core` and emits validated renderer commands. It does not own durable
story-bible graph state, does not write project data, and must not become a
dependency of `eidetic-core` or `eidetic-server`.

Current scope:

- Keep Bevy dependencies isolated from domain and server crates.
- Receive `BibleRenderGraphProjection` snapshots.
- Rebuild read-only Bevy ECS entities for graph nodes, edges, and influence
  highlights.
- Reject projection snapshots above the documented prototype full-rebuild
  envelope of 500 nodes and 1,000 edges. Larger primary graph views require
  keyed entity diffing before this crate may consume them.
- Derive disposable visual primitives for nodes and edges, including positions,
  radii, colors, widths, and highlight flags, so native render systems do not
  re-derive graph styling in Svelte or desktop command code.
- Derive renderer-neutral 3D visual snapshots for Milestone 9, including
  selected/highlighted/dimmed node states, bounded-label visibility, semantic
  graph edges, and structural parent/child edges derived from backend
  projections. These snapshots are disposable renderer inputs, not durable
  graph facts.
- Expose projection-provided neighborhood indexes for host-side graph highlighting.
- Validate selectable/inspectable graph node, edge, and influence IDs before
  emitting commands.
- Expose native Rust renderer state for the desktop host boundary.

Dependency review:

- Bevy is isolated to this leaf crate and is not a dependency of `eidetic-core`
  or `eidetic-server`.
- `bevy` is pinned to 0.18.1 and declared with `default-features = false` and only the `std`
  feature because this crate currently uses ECS/resource types and does not
  render windows, assets, text, audio, or UI.
- `cargo tree -p eidetic-bevy-bible-graph --depth 3` shows Bevy remains under
  this leaf crate, with `eidetic-core`, `serde`, and `thiserror` as the only
  other direct dependency families. The current Bevy subtree is app/ECS/input/
  math/time/transform/reflection support, not render/window/asset/text/audio.
- `cargo tree -p eidetic-bevy-bible-graph -e features --depth 3` without
  optional features shows the only enabled Bevy feature is `std`. This keeps the
  renderer crate usable as a projection-driven ECS bridge while the desktop
  renderer-window boundary is proven.
- Browser/WASM interop dependencies are intentionally absent. Eidetic's
  production renderer path is native desktop host integration through Tauri and
  Bevy, not browser canvas or wasm-bindgen.
- Native visual rendering must stay behind reviewed slices. Enabling Bevy
  render/window features is justified only for app-managed floating renderer
  windows owned by the desktop host, not WebView child-surface embedding. The
  first render slices should prove a minimal background/grid scene before graph
  nodes/edges are visualized.
- The `native_render` feature gates the reviewed Bevy render/window stack for
  desktop renderer-window work. It enables `2d_bevy_render`, `bevy_light`,
  `bevy_pbr`, `bevy_window`, and `bevy_winit` plus Linux `wayland`/`x11`
  window backends, and is intentionally off by default so projection-only tests
  and server builds do not pay for native rendering.
- Native renderer-window setup starts with a borderless scene resource, Eidetic
  graph colors, clear color, one marked `Camera3d`, and a renderer-local light.
  Graph nodes are renderer-local `Mesh3d` spheres and graph edges are
  renderer-local `Mesh3d` cuboids with `StandardMaterial` colors derived from
  the backend projection. Node labels are renderer-local text billboards that
  follow the native graph camera. The plugin does not own durable graph data;
  the desktop host owns renderer-window lifecycle.
- Native camera movement is renderer-local only: keyboard pan and zoom change
  the Bevy camera transform without mutating graph selection, layout, or
  backend-owned project data.
- Native renderer-window control is limited to renderer-local lifecycle
  signaling. `BibleGraphNativeWindowControlHandle` lets the desktop host request
  close without giving this leaf crate access to Tauri, SQLite, or durable
  project state.
- Desktop hosts enable `native_render` explicitly and start the renderer through
  `new_renderer_window()` so native readiness can be reported without letting the
  renderer own durable project state.
- Asset/text/UI/audio features remain out of scope for the bible graph renderer window
  until there is a concrete graph-rendering requirement that cannot be met with
  primitive meshes, materials, and Svelte-side semantic text/detail panels.
- `eidetic-desktop` may own OS/Tauri window handles and renderer window lifecycle.
  `eidetic-bevy-bible-graph` must continue to own only renderer-local scene
  state and validated commands; it must not learn Tauri, SQLite, or project
  persistence APIs.

Future scope:

- Native desktop host lifecycle is owned by `eidetic-desktop`; the leaf
  renderer remains responsible only for projection consumption, ECS state, and
  validated renderer commands.
- Force-layout and richer interaction state built from the renderer-neutral 3D
  snapshot boundary.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed graph mutation commands.
