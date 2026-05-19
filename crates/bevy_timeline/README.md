# Eidetic Bevy Timeline

Leaf Bevy integration for the timeline renderer.

This crate consumes backend-owned timeline render projections from `eidetic-core`
and emits validated renderer commands. It does not own persistent project state,
does not write timeline data, and must not become a dependency of `eidetic-core`
or `eidetic-server`.

Current scope:

- Keep Bevy dependencies isolated from domain and server crates.
- Receive `TimelineRenderProjection` snapshots.
- Rebuild read-only Bevy ECS entities for tracks, clips, arc tags, and relationships.
- Own transient pan and zoom viewport state derived from projection duration.
- Own transient playhead state bounded by projection duration.
- Validate selectable clip/node IDs before emitting commands.
- Hit-test read-only clips by track and timeline time for selection commands.
- Preserve backend-projected timeline relationships as disposable ECS entities for future curve rendering.
- Derive disposable relationship curve control points from timeline render projections.
- Emit validated node range command requests for backend-confirmed move/resize.
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
- Track and clip visual entities.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed move, resize, split, and relationship commands.
