# src-tauri/src/bevy_graph_host

## Purpose

This directory owns the desktop host for the native Bevy story-bible graph
renderer as it migrates toward the unified Bevy workspace renderer.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `host.rs` | Host facade used by desktop state and commands. |
| `owner.rs` | Window owner and thread lifecycle coordination. |
| `supervisor.rs` | Renderer supervisor state machine. |
| `window_thread.rs` | Worker-thread runner for native windows. |
| `platform_strategy/` | Platform capability and threading decisions. |
| `tests.rs` | Host and lifecycle tests. |

## Problem

Native renderer windows need explicit desktop lifecycle ownership without
coupling Bevy runtime policy into the backend server crate or Svelte UI.

## Constraints

- Bevy/Winit window creation can be platform-sensitive.
- Renderer commands and projections must remain backend-derived DTOs.
- Window threads must be stopped and joined deterministically.

## Decision

Keep the graph renderer host in a dedicated Tauri-side module that bridges
backend projections into native renderer ownership. During the workspace
renderer migration, this host can accept a combined graph/timeline workspace
projection while preserving the existing graph renderer lifecycle and command
drain boundary.

## Alternatives Rejected

- Owning Bevy windows from backend services: rejected because backend services
  should remain host-neutral.
- Driving native rendering directly from Svelte: rejected because Bevy owns the
  native window lifecycle.

## Invariants

- The owner/supervisor owns all renderer thread lifecycle transitions.
- Platform strategy decisions stay outside graph projection logic.
- Backend projection DTOs remain the source of rendered graph and timeline
  workspace state.

## Revisit Triggers

- Timeline and graph hosts converge on one lifecycle abstraction.
- Windows or macOS support changes threading requirements.
- Renderer commands become schema-generated.

## Dependencies

**Internal:** `eidetic-bevy-bible-graph`, `eidetic-server`, `crate::renderer_window`.
**External:** `tauri`, `tokio`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```rust
// Desktop state owns the host and command adapters call through the facade.
```

## API Consumer Contract

- Tauri commands can request renderer status, projection updates, and command
  drains through the host facade.
- Failures must preserve lifecycle state for UI status indicators.

## Structured Producer Contract

- Renderer status payloads are consumed by Svelte controls.
- Command drains produce backend command DTOs; adding command variants requires
  backend and frontend updates.
