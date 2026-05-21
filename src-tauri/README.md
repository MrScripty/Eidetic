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
| `src/main.rs` | Native desktop binary entrypoint. |
| `src/desktop_events.rs` | Backend `ServerEvent` to Tauri event bridge. |

## Invariants
- Tauri is the desktop transport boundary only. Canonical state remains owned by
  backend services in `eidetic-server`.
- Tauri commands call backend services directly and return validated
  command/projection results.
- Tauri events are transport projections of backend-owned events. They must not
  introduce a second frontend-owned event source of truth.
- The desktop crate may depend on Tauri; `eidetic-core`, renderer crates, and
  backend services must not depend on Tauri.

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
