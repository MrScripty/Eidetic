# ui/src/lib/stores

## Purpose

This directory contains the shared reactive frontend state used to coordinate the timeline shell, editor, sidebar panels, notifications, and backend event-driven updates.

## Contents

| File/Folder                               | Description                                                                          |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| `editor.svelte.ts`                        | Editor-local UI state and generation-progress helpers.                               |
| `timeline.svelte.ts`                      | Timeline viewport, playhead, tool, and drag interaction state.                       |
| `project.svelte.ts`                       | Active project session metadata.                                                     |
| `bibleGraphNodeProjection.svelte.ts`      | Focused cache/action layer for backend-owned bible graph node and field projections. |
| `bible.svelte.ts`                         | Typed transient graph selection state.                                               |
| `bibleRenderGraphProjection.svelte.ts`    | Focused cache layer for backend-owned bible render graph projections.                |
| `contextStackProjection.svelte.ts`        | Focused cache layer for backend-owned selected timeline context stack projections.   |
| `graphRendererCommands.ts`                | Applies validated transient Bevy graph renderer commands to UI selection state.      |
| `graphRendererWindow.svelte.ts`           | Backend-projected renderer-window status for display and lifecycle controls.         |
| `workspaceMode.svelte.ts`                 | Transient workspace layout mode for script, graph, and split views.                  |
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
| `serverEventHandlers.ts`                  | Backend event handlers that fan Tauri server events into stores.                     |

## Problem

Multiple UI surfaces need shared state and event coordination without turning the route tree into a prop-drilling graph.

## Constraints

- Store state must remain aligned with backend payload semantics.
- Polling and backend event ownership must stay single-owner to avoid duplicate work.
- Tests now cover API error handling and shared AI-status polling behavior from this boundary.

## Decision

Use focused Svelte state modules per feature area and keep backend event and polling orchestration here instead of burying it in component lifecycles.

Projection stores are the only allowed frontend caches for backend-owned
durable state. Transient stores may coordinate selection, hover, focus,
scrolling, zoom, local drafts, pending/error flags, and gesture state. Legacy
ownership paths listed below must be removed before the Bevy timeline renderer
continues.

## Store Ownership Audit

| Store                                     | Classification                      | Current status                                                                                                                            | Milestone 6 action                                                                                            |
| ----------------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `affectProposalProjection.svelte.ts`      | Projection cache and command bridge | Caches backend affect proposal projections and replaces cache from command responses with stale-response guards.                          | Keep as proposal projection state; durable affect changes must remain backend-command driven.                 |
| `aiStatus.svelte.ts`                      | Projection/status cache             | Owns the last-known AI backend status and the single polling lifecycle.                                                                   | Keep; add stale-response guards if config-driven overlapping refreshes become possible.                       |
| `bible.svelte.ts`                         | Transient UI state                  | Stores typed graph selection for nodes, edges, influences, context layers, and neighborhoods.                                              | Keep; do not use graph selection as durable graph state.                                                       |
| `bibleGraphNodeProjection.svelte.ts`      | Projection cache and command bridge | Caches backend projection envelopes and replaces cache from command responses with stale-response guards.                                 | Keep; later share common projection command helpers if useful.                                                |
| `bibleGraphSchemaProjection.svelte.ts`    | Projection cache                    | Caches backend schema projection with stale-response guards.                                                                              | Keep.                                                                                                         |
| `bibleRenderGraphProjection.svelte.ts`    | Projection cache                    | Caches backend render graph projection with stale-response guards.                                                                        | Keep.                                                                                                         |
| `changeReviewProjection.svelte.ts`        | Projection cache                    | Caches backend change review projection with stale-response guards.                                                                       | Keep.                                                                                                         |
| `contextStackProjection.svelte.ts`        | Projection cache                    | Caches the backend context stack for the selected timeline node with stale-response guards.                                               | Keep; graph context layer UI should read this cache rather than deriving hierarchy in Svelte.                 |
| `editor.svelte.ts`                        | Transient UI state                  | Stores selected timeline node IDs, selected level, and generation-progress state only.                                                    | Keep durable selected-node data in `selectedNodeEditorProjection.svelte.ts`.                                  |
| `graphRendererCommands.ts`                | Transient command application       | Applies validated renderer selection/inspect commands to typed transient graph selection helpers.                                        | Keep as transient selection adapter; durable graph mutations must still use backend commands.                 |
| `graphRendererWindow.svelte.ts`           | Projection/status cache             | Stores the last backend-projected renderer-window status for display and renderer lifecycle controls.                                     | Keep as status projection only; do not derive durable graph state here.                                       |
| `selectedNodeEditorProjection.svelte.ts`  | Backend projection cache            | Caches the focused backend editor projection keyed by selected timeline node ID with request-id and stale-response guards.                | Keep as the editor read model; do not patch durable node fields locally.                                      |
| `notifications.svelte.ts`                 | Transient UI state                  | Owns discardable toast messages.                                                                                                          | Keep.                                                                                                         |
| `objectFieldProjection.svelte.ts`         | Projection cache and command bridge | Caches backend object-field projections and replaces cache from command responses with stale-response guards.                             | Keep.                                                                                                         |
| `project.svelte.ts`                       | Transient/session metadata          | Stores active project metadata used to gate the shell after backend create/load.                                                          | Keep visible durable project data projection-backed; do not store full `Project` DTOs here.                   |
| `projectSession.ts`                       | Projection refresh orchestration    | Owns project create/load activation ordering, lifecycle clearing, transient-state reset, and initial projection refreshes.                | Keep as the single project activation owner; do not hydrate broad project/timeline DTOs into stores.          |
| `projectionCacheGuards.ts`                | Projection cache infrastructure     | Provides shared version guards for replace-only projection cache writes.                                                                  | Keep as infrastructure; projection stores should use it instead of ad hoc stale-response checks.              |
| `projectionRefreshQueue.ts`               | Projection refresh orchestration    | Coalesces backend event/project-triggered projection refreshes and resolves queued waiters during teardown.                                | Keep as the single refresh coalescing owner; do not start per-component refresh state machines.               |
| `propagationProposalProjection.svelte.ts` | Projection cache and command bridge | Caches backend proposal list and replaces cache from command responses with stale-response guards.                                        | Keep.                                                                                                         |
| `scriptDocumentProjection.svelte.ts`      | Projection cache and command bridge | Caches backend script document projections and replaces cache from command responses with stale-response guards.                          | Keep.                                                                                                         |
| `semanticProposalProjection.svelte.ts`    | Projection cache and command bridge | Caches backend semantic proposal list and replaces cache from command responses with stale-response guards.                               | Keep.                                                                                                         |
| `shortcuts.svelte.ts`                     | Transient UI infrastructure         | Owns in-memory shortcut registrations.                                                                                                    | Keep; ensure cleanup on component unmount remains deterministic.                                              |
| `storyArcProjection.svelte.ts`            | Projection cache and command bridge | Caches backend story arc projection and replaces cache from command responses with stale-response guards.                                 | Keep.                                                                                                         |
| `timeline.svelte.ts`                      | Transient UI state                  | Stores viewport, zoom, playhead, active tool, snapping, and connection drag only.                                                         | Keep timeline clips/tracks/arcs in `timelineRenderProjection.svelte.ts`; do not add broad timeline DTO state. |
| `timelineKeyboardCommands.ts`             | Transient command application       | Maps keyboard shortcuts to backend timeline command helpers without storing durable timeline data.                                         | Keep as a shortcut adapter; command responses must replace projection caches rather than patching clips.      |
| `timelineRenderProjection.svelte.ts`      | Projection cache and command bridge | Desired pattern for timeline commands: command responses replace the projection cache with stale-response guards.                         | Keep; add coalescing and shared command helper only after ownership cleanup.                                  |
| `timelineRendererWindow.svelte.ts`        | Projection/status cache             | Stores the last backend-projected timeline renderer-window status for display and lifecycle controls.                                      | Keep as status projection only; do not store renderer-owned timeline data here.                               |
| `serverEventHandlers.ts`                  | Projection refresh orchestration    | Routes Tauri backend events into projection refresh requests through `projectionRefreshQueue.ts`; does not hydrate or patch broad durable DTOs. | Keep as orchestration only; add new event handling by requesting focused projection refreshes.                |

## Alternatives Rejected

- Centralizing every UI field in one store file: rejected because it would collapse unrelated lifecycles into one mutable surface.

## Invariants

- Shared polling and backend event flows retain explicit ownership and cleanup semantics.
- Stores remain the source of transient UI coordination; components react to them.
- Projection stores cache backend envelopes and must not patch broad durable entity state optimistically.
- Backend contract changes are reflected here before individual components fork around them.

## Revisit Triggers

- Another realtime channel or polling workflow appears without a clear current owner.
- A store starts mixing durable project data and ephemeral UI-only state without clear boundaries.

## Dependencies

**Internal:** `ui/src/lib/api.ts`, `ui/src/lib/serverEventClient.ts`, `ui/src/lib/types.ts`.
**External:** Svelte 5.

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
- Changes to shared store fields must land with dependent component, backend event, README, and verification updates in the same logical slice.
