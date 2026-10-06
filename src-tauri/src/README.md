# src-tauri/src

## Purpose

This directory owns the Tauri desktop shell for Eidetic: command adapters,
projection adapters, desktop event bridges, renderer-window hosts, and binary
entry points over backend services.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `main.rs` | Desktop application entry point and Tauri builder wiring. |
| `ai_commands.rs` | AI command transport including explicit optional fictional-time context preview. |
| `commands/` | Tauri command adapters over backend service APIs. |
| `projections/` | Tauri projection readers over backend projection services. |
| `bevy_graph_host/` | Native story-bible graph renderer host and lifecycle owner. |
| `bin/` | Desktop helper binaries and smoke-test entry points. |

## Problem

The desktop shell must expose backend-owned behavior to the Svelte UI without
moving persistence, domain policy, or renderer lifecycle decisions into
frontend code.

## Constraints

- Tauri adapters must stay thin over backend services.
- Renderer hosts must own their threads, shutdown, and platform strategy.
- Desktop event bridges must keep payloads compatible with frontend stores.

## Decision

Keep desktop integration in `src-tauri/src` and route behavior through
service-level Rust APIs. The shell owns process/window integration; backend
crates own project state and command semantics.

## Alternatives Rejected

- Reintroducing a loopback HTTP server: rejected because production desktop
  transport is Tauri commands and events.
- Moving command policy into Svelte stores: rejected because backend state is
  authoritative.

## Invariants
- The desktop builder registers `command_script_block_create` alongside existing
  edit commands so an empty screenplay can be authored through the same native
  invoke/event boundary.
- The desktop builder registers the targeted screenplay impact preview command
  alongside the existing propagation review commands; no extra transport or
  automatic acceptance path is introduced.
- AI context preview transports optional `story_time_ms` to backend resolution;
  it never derives fictional time from playhead or selected-clip coordinates.

- Commands validate and delegate; they do not fork backend business rules.
- Projection adapters return backend-owned read models.
- Renderer hosts have explicit lifecycle owners.

## Revisit Triggers

- A second desktop shell needs the same adapters.
- Tauri command payloads become generated from shared schemas.
- Renderer hosts require a shared lifecycle abstraction.

## Dependencies

**Internal:** `eidetic-server`, `eidetic-core`, native renderer crates.
**External:** `tauri`, `tokio`, `serde`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```rust
tauri::Builder::default();
```

## API Consumer Contract

- Frontend code calls these commands through Tauri's invoke/event APIs.
- Command failures must be serialized as user-facing error strings or typed
  payloads already understood by the UI.
- Renderer events must remain compatible with the frontend event client.

## Structured Producer Contract

- Command and projection payloads are JSON-compatible Rust structures consumed
  by Svelte stores.
- Any payload shape change requires synchronized TypeScript contract updates.

The command registry includes `command_bible_graph_edge_label` for authored
relationship labels. Its expected-revision contract reaches the server writer
transaction through the existing Bible IPC/service boundary, keeping label
editing and screenplay dependency invalidation on normal desktop transport.

The Bevy timeline bridge continues submitting legacy range commands with no expected receipt. Exact placement form custody is optional on the shared command and does not alter renderer gesture ownership.
