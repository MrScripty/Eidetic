# crates/bevy_timeline/src

## Purpose

This directory contains the native Bevy timeline renderer, including viewport
math, scene rebuilds, native input, command emission, and optional Winit window
integration.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `lib.rs` | Renderer exports and module wiring. |
| `app.rs` | Headless renderer app facade. |
| `scene.rs` | Timeline scene rebuild logic. |
| `native_render.rs` | Native window setup and lifecycle systems. |
| `native_input.rs` | Native input-to-command behavior. |
| `native_command.rs` | Validated command emission helpers. |
| `tests/` | Focused renderer and native behavior tests. |

## Problem

The desktop app needs a native timeline view that can render backend timeline
projections and emit validated timeline commands.

## Constraints

- The crate is a leaf renderer crate.
- Native window tests must be separable from display-required smoke checks.
- Timeline command validation must use projection data, not UI guesses.

## Decision

Keep timeline renderer behavior in a dedicated Bevy crate with a headless app
facade for tests and native window helpers for the Tauri host.

## Alternatives Rejected

- Moving timeline rendering into backend services: rejected because rendering
  and input are host concerns.
- Emitting unchecked UI commands: rejected because renderer commands must remain
  bounded by projection state.

## Invariants

- Renderer commands are validated before they cross back to backend services.
- Viewport and playhead changes stay within projection bounds.
- Bevy dependencies do not leak into `eidetic-core`.

## Revisit Triggers

- Timeline and graph renderer hosts share a common lifecycle abstraction.
- The renderer becomes a reusable package.
- Native timeline editing expands beyond current command intents.

## Dependencies

**Internal:** `eidetic-core`.
**External:** `bevy`, `serde`, `thiserror`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```rust
use eidetic_bevy_timeline::TimelineRendererApp;

let renderer = TimelineRendererApp::new();
assert!(renderer.drain_commands().is_empty());
```

## API Consumer Contract

- Tauri hosts consume the renderer app and native window helpers.
- The renderer accepts backend projection DTOs and emits backend command DTOs.

## Structured Producer Contract

- Scene stats and command drains are structured runtime payloads consumed by
  desktop host code and tests.
