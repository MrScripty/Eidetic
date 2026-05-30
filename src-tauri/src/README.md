# src-tauri/src

## Purpose

This directory owns the Tauri desktop shell for Eidetic: command adapters,
projection adapters, desktop event bridges, renderer-window hosts, and binary
entry points over backend services.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `main.rs` | Desktop application entry point and Tauri builder wiring. |
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
