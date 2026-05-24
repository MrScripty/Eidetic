# src-tauri

## Purpose
This directory contains the Tauri desktop shell for Eidetic. It packages the
Svelte projection UI and exposes backend-owned Rust services through Tauri
commands and events.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `Cargo.toml` | Desktop crate manifest. Tauri dependencies are isolated here. |
| `tauri.conf.json` | Desktop window, frontend asset, and build command configuration. |
| `capabilities/` | Tauri permission configuration for desktop IPC access. |
| `src/lib.rs` | Tauri command/event registration and backend runtime composition. |
| `src/main.rs` | Native desktop binary entrypoint and `--smoke` startup probe. |
| `src/bin/native_renderer_smoke.rs` | Diagnostic-only Bevy/winit native graph renderer preflight binary. |
| `src/ai_commands.rs` | Tauri commands for AI status, config, context-preview, child-plan generation, and streaming script generation service access. |
| `src/bevy_graph_host/` | Desktop-managed lifecycle owner and focused host adapter for the Bevy bible graph renderer leaf crate. |
| `src/desktop_events.rs` | Backend `ServerEvent` to Tauri event bridge. |
| `src/export_commands.rs` | Tauri commands for export service access. |
| `src/graph_renderer_commands.rs` | Tauri commands for native Bevy graph renderer status and validated command draining. |
| `src/model_commands.rs` | Tauri commands for Pumas model-library projection reads. |
| `src/reference_commands.rs` | Tauri commands for reference document list/upload/delete service access. |

## Invariants
- Tauri is the desktop transport boundary only. Canonical state remains owned by
  backend services in `eidetic-server`.
- Tauri commands call backend services directly and return validated
  command/projection results.
- Tauri events are transport projections of backend-owned events. They must not
  introduce a second frontend-owned event source of truth.
- The desktop shell calls backend lifecycle shutdown when the window is
  destroyed so long-running backend tasks do not remain detached.
- The `--smoke` binary path initializes the backend runtime, emits JSON health,
  shuts down supervised backend tasks, and exits without opening a window.
- `eidetic-native-renderer-smoke --report-only` emits a JSON preflight record
  for the diagnostic Bevy/winit graph renderer path without opening a window.
  It records platform, backend, threading mode, window config, command, and
  observed report-only result. This diagnostic must not mark production
  renderer-window support as verified.
- The desktop crate may depend on Tauri; `eidetic-core`, renderer crates, and
  backend services must not depend on Tauri.
- Bevy renderer hosts live in this desktop crate. They may consume backend
  projections and emit validated transient renderer commands, but they must not
  write durable project state or make `eidetic-server` depend on Bevy.
- `projection_bible_render_graph` mirrors the backend-owned graph projection
  into the Bevy renderer owner when desktop state is available. Svelte still
  receives the same projection envelope, so renderer failures are logged instead
  of turning the read projection into a UI-blocking error.
- Native renderer IPC exposes status and command-drain reads only. Drained
  renderer commands are validated transient interaction intents, not durable
  bible graph mutations.
- Native renderer window threads are desktop-owned lifecycle resources. They
  must expose bounded close/join behavior and report completion or panic through
  backend status instead of detaching unmanaged Bevy event loops.
- Production Bevy rendering uses app-managed floating renderer windows, not
  WebView child-surface embedding. Svelte may launch, focus, close, and display
  status for a renderer window, but desktop Rust owns renderer lifecycle,
  command queues, projection subscription, and teardown.
- Raw OS/window handles are not part of the default desktop dependency surface.
  If a future native runner requires platform handles, the dependency belongs in
  the thin desktop platform module that owns the handle and must be covered by
  the renderer-runner safety and verification plan before becoming production.
- Renderer host state must not be stored in `tauri::State` unless the owner is
  `Send + Sync`; Bevy `App` is not. Native render-window integration needs a
  dedicated desktop renderer owner instead of storing `App` in global managed
  state. `DesktopBibleGraphRendererOwner` is the managed boundary; the Bevy
  renderer itself lives on its owned thread.

## API Consumer Contract
- Svelte invokes desktop commands by name through Tauri IPC.
- Svelte listens for backend refresh events on `eidetic://server-event`; the
  payload shape is `{ event: ServerEvent }`, where `ServerEvent` keeps the
  backend snake-case `type` discriminator.
- Command errors are serialized as `{ kind, message }` and should be treated as
  transport-safe projections of backend service errors.

## Structured Producer Contract
- This crate produces no durable project state. It composes backend services,
  packages UI assets, and emits/returns transport DTOs derived from backend
  service outputs.
