# ui/src/lib/stores

## Purpose

This directory contains the shared reactive frontend state used to coordinate the timeline shell, editor, sidebar panels, notifications, and websocket-driven updates.

## Contents

| File/Folder                               | Description                                                                          |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| `editor.svelte.ts`                        | Editor-local UI state and generation-progress helpers.                               |
| `timeline.svelte.ts`                      | Timeline viewport, playhead, tool, and drag interaction state.                       |
| `project.svelte.ts`                       | Legacy broad project mirror pending replacement with project/session metadata.       |
| `bibleGraphNodeProjection.svelte.ts`      | Focused cache/action layer for backend-owned bible graph node and field projections. |
| `bible.svelte.ts`                         | Bible panel selection state.                                                         |
| `bibleRenderGraphProjection.svelte.ts`    | Focused cache layer for backend-owned bible render graph projections.                |
| `bibleGraphSchemaProjection.svelte.ts`    | Focused cache layer for backend-owned bible graph schema projections.                |
| `objectFieldProjection.svelte.ts`         | Focused cache/action layer for backend-owned object-field projections.               |
| `scriptDocumentProjection.svelte.ts`      | Focused cache/action layer for backend-owned script document projections.            |
| `semanticProposalProjection.svelte.ts`    | Focused cache/action layer for semantic bible reference proposals.                   |
| `propagationProposalProjection.svelte.ts` | Focused cache/action layer for semantic propagation proposals.                       |
| `changeReviewProjection.svelte.ts`        | Focused cache layer for backend-owned change history review projections.             |
| `storyArcProjection.svelte.ts`            | Focused cache/action layer for backend-owned story arc projections.                  |
| `aiStatus.svelte.ts`                      | Shared AI-status polling ownership.                                                  |
| `shortcuts.svelte.ts`                     | Keyboard shortcut registry and dispatch helpers.                                     |
| `notifications.svelte.ts`                 | Toast notification queue state.                                                      |
| `wsHandlers.ts`                           | Websocket event handlers that fan server events into stores.                         |

## Problem

Multiple UI surfaces need shared state and event coordination without turning the route tree into a prop-drilling graph.

## Constraints

- Store state must remain aligned with backend payload semantics.
- Polling and websocket ownership must stay single-owner to avoid duplicate work.
- Tests now cover API error handling and shared AI-status polling behavior from this boundary.

## Decision

Use focused Svelte state modules per feature area and keep websocket/polling orchestration here instead of burying it in component lifecycles.

Projection stores are the only allowed frontend caches for backend-owned
durable state. Transient stores may coordinate selection, hover, focus,
scrolling, zoom, local drafts, pending/error flags, and gesture state. Legacy
ownership paths listed below must be removed before the Bevy timeline renderer
continues.

## Store Ownership Audit

| Store                                     | Classification                            | Current status                                                                                                                            | Milestone 6 action                                                                                            |
| ----------------------------------------- | ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `aiStatus.svelte.ts`                      | Projection/status cache                   | Owns the last-known AI backend status and the single polling lifecycle.                                                                   | Keep; add stale-response guards if config-driven overlapping refreshes become possible.                       |
| `bible.svelte.ts`                         | Transient UI state                        | Stores selected bible graph node ID only.                                                                                                 | Keep; document as frontend-owned selection.                                                                   |
| `bibleGraphNodeProjection.svelte.ts`      | Projection cache and command bridge       | Caches backend projection envelopes and replaces cache from command responses.                                                            | Keep; later share common projection command helpers if useful.                                                |
| `bibleGraphSchemaProjection.svelte.ts`    | Projection cache                          | Caches backend schema projection.                                                                                                         | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `bibleRenderGraphProjection.svelte.ts`    | Projection cache                          | Caches backend render graph projection.                                                                                                   | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `changeReviewProjection.svelte.ts`        | Projection cache                          | Caches backend change review projection.                                                                                                  | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `editor.svelte.ts`                        | Transient UI state                        | Stores selected timeline node IDs, selected level, and generation-progress state only.                                                    | Keep durable selected-node data in `selectedNodeEditorProjection.svelte.ts`.                                  |
| `selectedNodeEditorProjection.svelte.ts`  | Backend projection cache                  | Caches the focused backend editor projection keyed by selected timeline node ID.                                                          | Keep as the editor read model; do not patch durable node fields locally.                                      |
| `notifications.svelte.ts`                 | Transient UI state                        | Owns discardable toast messages.                                                                                                          | Keep.                                                                                                         |
| `objectFieldProjection.svelte.ts`         | Projection cache and command bridge       | Caches backend object-field projections and replaces cache from command responses.                                                        | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `project.svelte.ts`                       | Legacy ownership                          | Stores a broad `Project` including durable timeline data.                                                                                 | Replace with lightweight project/session metadata or open-state only.                                         |
| `propagationProposalProjection.svelte.ts` | Projection cache and command bridge       | Caches backend proposal list and replaces cache from command responses.                                                                   | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `scriptDocumentProjection.svelte.ts`      | Projection cache and command bridge       | Caches backend script document projections and replaces cache from command responses.                                                     | Keep.                                                                                                         |
| `semanticProposalProjection.svelte.ts`    | Projection cache and command bridge       | Caches backend semantic proposal list and replaces cache from command responses.                                                          | Keep; add stale-response guards when projection refresh coalescing lands.                                     |
| `shortcuts.svelte.ts`                     | Transient UI infrastructure               | Owns in-memory shortcut registrations.                                                                                                    | Keep; ensure cleanup on component unmount remains deterministic.                                              |
| `storyArcProjection.svelte.ts`            | Projection cache and command bridge       | Caches backend story arc projection and replaces cache from command responses.                                                            | Keep.                                                                                                         |
| `timeline.svelte.ts`                      | Transient UI state                        | Stores viewport, zoom, playhead, active tool, snapping, and connection drag only.                                                         | Keep timeline clips/tracks/arcs in `timelineRenderProjection.svelte.ts`; do not add broad timeline DTO state. |
| `timelineRenderProjection.svelte.ts`      | Projection cache and command bridge       | Desired pattern for timeline commands: command responses replace the projection cache.                                                    | Keep; add stale-response guards/coalescing and shared command helper only after ownership cleanup.            |
| `wsHandlers.ts`                           | Mixed orchestration plus legacy ownership | Correctly refreshes several projections, but timeline and generation events still hydrate/patch broad timeline and selected node objects. | Rewrite as projection refresh/invalidation orchestration only.                                                |

## Alternatives Rejected

- Centralizing every UI field in one store file: rejected because it would collapse unrelated lifecycles into one mutable surface.

## Invariants

- Shared polling and websocket flows retain explicit ownership and cleanup semantics.
- Stores remain the source of transient UI coordination; components react to them.
- Projection stores cache backend envelopes and must not patch broad durable entity state optimistically.
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
import { refreshTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';

await refreshTimelineRenderProjection();
```

## API Consumer Contract

- None identified as of 2026-03-08.
- Reason: stores are an internal frontend coordination layer.
- Revisit trigger: store modules become a published state-management package.

## Structured Producer Contract

- Projection cache stores hold backend `ProjectionEnvelope` payloads as replace-only snapshots. Components may read cached envelopes but must not mutate payload contents.
- Transient stores own local interaction state only. If the backend stores or acts on a field, that field must be changed through a command helper and refreshed through a projection response or invalidation.
- Changes to shared store fields must land with dependent component, websocket, README, and verification updates in the same logical slice.
