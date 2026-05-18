# Eidetic Bevy Timeline

Leaf Bevy integration for the timeline renderer.

This crate consumes backend-owned timeline render projections from `eidetic-core`
and emits validated renderer commands. It does not own persistent project state,
does not write timeline data, and must not become a dependency of `eidetic-core`
or `eidetic-server`.

Current scope:

- Keep Bevy dependencies isolated from domain and server crates.
- Receive `TimelineRenderProjection` snapshots.
- Validate selectable clip/node IDs before emitting commands.

Future scope:

- Browser or desktop host lifecycle.
- Track and clip visual entities.
- Pointer, keyboard, and accessibility command flows.
- Backend-confirmed move, resize, split, and relationship commands.
