# crates/bevy_bible_graph/src

## Purpose

This directory contains the native Bevy story-bible graph renderer, including
projection-to-visual mapping, scene rebuilds, hit testing, command emission, and
optional Winit window integration.

## Contents

| File | Description |
| ---- | ----------- |
| `lib.rs` | Renderer app facade and public exports. |
| `visual.rs` | Projection-to-visual DTO mapping. |
| `visual_3d.rs` | 3D visual snapshot helpers. |
| `scene.rs` | Bevy scene rebuild logic. |
| `native_render.rs` | Native window, input, camera, text editor, and command systems. |
| `category.rs` | Category color/style helpers. |
| `lib_tests.rs` | Renderer app and native system tests. |

## Problem

The desktop app needs an inspectable native graph renderer without moving graph
projection policy out of backend/core contracts.

## Constraints

- The crate is a leaf renderer crate.
- Native window creation must be separable from headless logic tests.
- Backend projection DTOs remain the source of rendered state.

## Decision

Keep renderer state and Bevy systems together in a leaf crate and expose a
small app facade plus native window helpers to the Tauri host.

## Alternatives Rejected

- Rendering the graph entirely in Svelte: rejected because native graph
  interaction and 3D rendering are renderer-owned concerns.
- Moving graph projection into Bevy systems: rejected because backend services
  own read-model construction.

## Invariants

- Renderer commands are validated against the active projection.
- Headless tests must not require display creation.
- Bevy dependencies do not leak into `eidetic-core`.

## Revisit Triggers

- The renderer becomes a reusable package.
- Window lifecycle is unified with the timeline renderer.
- Native text editing grows beyond renderer-local behavior.

## Dependencies

**Internal:** `eidetic-core`.
**External:** `bevy`, `serde`, `thiserror`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```rust
use eidetic_bevy_bible_graph::BibleGraphRendererApp;

let renderer = BibleGraphRendererApp::new();
assert_eq!(renderer.projection_node_count(), 0);
```

## API Consumer Contract

- Tauri hosts consume the renderer app and native window helpers.
- The renderer accepts backend projection DTOs and emits backend command DTOs.

## Structured Producer Contract

- Visual snapshots are disposable render projections and must not become
  canonical graph state.
