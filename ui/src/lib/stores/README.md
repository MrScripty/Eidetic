# ui/src/lib/stores

## Purpose
This directory contains the shared reactive frontend state used to coordinate the timeline shell, editor, sidebar panels, notifications, and websocket-driven updates.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `editor.svelte.ts` | Editor-local UI state and generation/consistency helpers. |
| `timeline.svelte.ts` | Timeline selection, traversal, and view-state coordination. |
| `story.svelte.ts` / `bible.svelte.ts` | Story-entity and bible-facing state. |
| `aiStatus.svelte.ts` | Shared AI-status polling ownership. |
| `wsHandlers.ts` | Websocket event handlers that fan server events into stores. |

## Problem
Multiple UI surfaces need shared state and event coordination without turning the route tree into a prop-drilling graph.

## Constraints
- Store state must remain aligned with backend payload semantics.
- Polling and websocket ownership must stay single-owner to avoid duplicate work.
- Tests now cover API error handling and shared AI-status polling behavior from this boundary.

## Decision
Use focused Svelte state modules per feature area and keep websocket/polling orchestration here instead of burying it in component lifecycles.

## Alternatives Rejected
- Centralizing every UI field in one store file: rejected because it would collapse unrelated lifecycles into one mutable surface.

## Invariants
- Shared polling and websocket flows retain explicit ownership and cleanup semantics.
- Stores remain the source of transient UI coordination; components react to them.
- Backend contract changes are reflected here before individual components fork around them.

## Revisit Triggers
- Another realtime channel or polling workflow appears without a clear current owner.
- A store starts mixing durable project data and ephemeral UI-only state without clear boundaries.

## Dependencies
**Internal:** `ui/src/lib/api.ts`, `ui/src/lib/ws.ts`, `ui/src/lib/types.ts`.
**External:** Svelte 5, Yjs.

## Related ADRs
- `ADR-001` decomposition baseline for oversized frontend modules.

## Usage Examples
```ts
import { editorState } from '$lib/stores/editor.svelte.js';

editorState.generationError = null;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: stores are an internal frontend coordination layer.
- Revisit trigger: store modules become a published state-management package.

## Structured Producer Contract
- Store field names mirror server and UI expectations for selection, status, and generation lifecycle state.
- Changes to shared store fields must land with dependent component and websocket updates in the same change.
