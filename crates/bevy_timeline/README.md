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
- Preserve projected clip level, lock, content status, and deterministic
  renderer colors in native clip visuals.
- Highlight the backend-projected selected clip in native visuals without
  storing selection as renderer-owned durable state.
- Render disposable native relationship connector visuals from backend
  projection relationships and endpoint clip positions.
- Render disposable native affect overlay bars from backend-projected affect
  samples, valence, and intensity.
- Rebuild read-only Bevy ECS entities for backend-projected affect overlays.
- Own transient pan and zoom viewport state derived from projection duration.
- Own transient playhead state bounded by projection duration.
- Validate selectable clip/node IDs before emitting commands.
- Hit-test read-only clips by track and timeline time for selection commands.
- Hit-test read-only clips from validated viewport pixel coordinates using the
  current transient viewport, without storing durable renderer layout.
- Preserve backend-projected timeline relationships as disposable ECS entities for future curve rendering.
- Derive and expose disposable relationship curve control points from timeline render projections.
- Emit validated node range command requests for backend-confirmed move/resize.
- Emit validated split command requests with backend-required replacement node IDs.
- Expose native Rust renderer state for the desktop host boundary.
- Expose a feature-gated native window control API for the future desktop host:
  `TimelineNativeWindowRunnerConfig`, `TimelineNativeWindowControlHandle`,
  controlled Bevy app configuration, and minimal native window runners. This
  API owns only renderer-local lifecycle signaling for ready, visible, show,
  hide, and close state.
- Render a disposable native playhead visual from bounded renderer-local
  playhead state and the current transient viewport.
- Nudge native playhead state through Bevy keyboard input while keeping the
  position clamped to the backend projection duration.
- Emit native node range command requests only after validating the node and
  requested range against the active backend projection.
- Emit native selected-clip nudge requests from backend-projected selection
  through validated node range commands.
- Emit native selected-clip resize requests from backend-projected selection
  through validated node range commands.
- Emit native delete command requests only for nodes present in the active
  backend projection.
- Emit native selected-delete command requests from backend-projected selection
  only, without renderer-owned durable selected node state.
- Emit native split command requests only when the split point is inside the
  projected clip and replacement node IDs are available.
- Emit native selected-split command requests from backend-projected selection
  and renderer-local playhead state only.
- Emit native create command requests only when parent references and requested
  ranges are valid for the active backend projection.
- Keep native command validation in `native_command.rs` so the native renderer
  window module stays focused on lifecycle, input, and visual projection work.
- Keep native styling decisions in `native_style.rs` so color and size mapping
  stays separate from Bevy window lifecycle and input handling.
- Keep native keyboard and pointer systems in `native_input.rs` so command
  intent handling reads backend projections and transient renderer viewport
  state without accumulating inside the window lifecycle module.

Dependency review:

- Bevy is isolated to this leaf crate and is not a dependency of `eidetic-core`
  or `eidetic-server`.
- `bevy` is pinned to 0.18.1 and declared with `default-features = false` and
  only the `std` feature for default builds because this crate currently uses
  ECS/resource types and does not render windows, assets, text, audio, or UI
  unless the explicit native renderer feature is selected.
- `cargo tree -p eidetic-bevy-timeline --depth 2` shows Bevy remains under this
  leaf crate, with `eidetic-core`, `serde`, `thiserror`, `serde_json`, and
  `uuid` as the only other direct dependency families.
- Browser/WASM interop dependencies are intentionally absent. Eidetic's
  production renderer path is native desktop host integration through Tauri and
  Bevy, not browser canvas or wasm-bindgen.
- The current normal dependency tree has 110 unique crates. That is acceptable
  only because Bevy is leaf-scoped and because render/window/text/UI/asset
  features are still disabled.
- The `native_render` feature gates the reviewed Bevy render/window stack for
  future desktop timeline renderer-window work. It enables `2d_bevy_render`,
  `bevy_window`, and `bevy_winit` plus Linux `wayland`/`x11` window backends,
  and is intentionally off by default so projection-only tests and server
  builds do not pay for native rendering.
- Native renderer-window control is limited to renderer-local lifecycle
  signaling. `TimelineNativeWindowControlHandle` lets the desktop host request
  close/show/hide and observe ready/visible state without giving this leaf crate
  access to Tauri, SQLite, or durable project state.
- Asset/text/UI/audio features remain out of scope for the timeline renderer
  window until a concrete timeline-rendering requirement proves they are needed.
- A guard test fails if native render features move into default builds or if
  text/UI features are added without a separate dependency-review slice.
- Adding more Bevy render/window/asset/text/input features requires a new
  dependency review, a commit that explains the transitive dependency cost, and
  proof that the feature remains out of `eidetic-core` and `eidetic-server`.

Future scope:

- Desktop host lifecycle.
- Track and clip visual entities.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed move, resize, split, and relationship commands.
