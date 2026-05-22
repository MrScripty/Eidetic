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
- `cargo tree -p eidetic-bevy-bible-graph --depth 2` shows Bevy remains under
  this leaf crate, with `eidetic-core`, `serde`, and `thiserror` as the only
  other direct dependency families.
- Browser/WASM interop dependencies are intentionally absent. Eidetic's
  production renderer path is native desktop host integration through Tauri and
  Bevy, not browser canvas or wasm-bindgen.
- Adding Bevy render/window/asset/text/input features requires a new dependency
  review and a commit that explains the transitive dependency cost.

Future scope:

- Native desktop host lifecycle is owned by `eidetic-desktop`; the leaf
  renderer remains responsible only for projection consumption, ECS state, and
  validated renderer commands.
- Camera, force-layout, and interaction state.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed graph mutation commands.
