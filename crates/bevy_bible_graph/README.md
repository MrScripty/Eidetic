# Eidetic Bevy Bible Graph

Leaf Bevy integration for the story-bible graph renderer.

This crate consumes backend-owned `BibleRenderGraphProjection` snapshots from
`eidetic-core` and emits validated renderer commands. It does not own durable
story-bible graph state, does not write project data, and must not become a
dependency of `eidetic-core` or `eidetic-server`.

Current scope:

- Keep Bevy dependencies isolated from domain and server crates.
- Receive `BibleRenderGraphProjection` snapshots.
- Rebuild read-only Bevy ECS entities for graph nodes and edges.
- Validate selectable/inspectable graph node IDs before emitting commands.
- Expose a wasm-bindgen bridge for browser hosts.

Dependency review:

- Bevy is isolated to this leaf crate and is not a dependency of `eidetic-core`
  or `eidetic-server`.
- `bevy` is declared with `default-features = false` and only the `std`
  feature because this crate currently uses ECS/resource types and does not
  render windows, assets, text, audio, or UI.
- Browser interop dependencies are target-scoped to `wasm32`.
- Adding Bevy render/window/asset/text/input features requires a new dependency
  review and a commit that explains the transitive dependency cost.

Future scope:

- Browser canvas or desktop host lifecycle.
- Camera, force-layout, and interaction state.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed graph mutation commands.
