# ui/src/lib

## Purpose

This directory holds the shared frontend surface for the Eidetic UI: typed API contracts, client-side stores, reusable components, and cross-cutting helpers that let the Svelte app render the current project state without duplicating server schema knowledge.

## Contents

| File/Folder                | Description                                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------------------ |
| `types.ts`                 | Shared TypeScript mirrors of core timeline, story, and UI layout contracts.                      |
| `bibleGraphSchemaTypes.ts` | Focused TypeScript mirrors for backend-owned bible graph schema projection shapes.               |
| `api.ts`                   | Frontend request helpers for project, timeline, story, and export operations.                    |
| `desktopTransport.ts`      | Tauri IPC detection and command invocation helpers for desktop-hosted frontend code.             |
| `commandApi.ts`            | Browser-side command helper barrel for backend-owned commands and versioned command projections. |
| `timelineCommandApi.ts`    | Timeline-specific command helpers that prefer Tauri IPC and fall back to legacy HTTP adapters.   |
| `commandTransport.ts`      | Shared command IDs and legacy HTTP command transport used while Tauri migration is in progress.  |
| `serverEventClient.ts`     | Backend event client for Tauri desktop event transport.                                         |
| `commandApi.test.ts`       | Tests for command helper request shape and backend error handling.                               |
| `projectionApi.ts`         | Browser-side read helpers for focused backend projections.                                       |
| `projectionApi.test.ts`    | Tests for projection helper query shape and backend error handling.                              |
| `stores/`                  | Reactive Svelte state used to coordinate the UI around backend-driven data.                      |
| `components/`              | Feature UI modules for layout, timeline editing, sidebars, and relationship views.               |

## Problem

The UI needs one place where backend-backed shapes, local UI constants, and shared rendering behavior stay consistent. Without a common `lib/` boundary, the app would drift into per-component contract copies and fragile ad hoc wiring.

## Constraints

- Backend responses remain the source of truth for project content.
- Timeline rendering depends on stable shared geometry constants across multiple components.
- The UI mixes persistent project data with transient interaction state, so the boundary must distinguish the two clearly.

## Decision

Keep shared UI contracts, stores, and feature components under `ui/src/lib` and route layout-sensitive constants through `types.ts` so rendering code uses one source of truth for timeline sizing and related chrome.

## Alternatives Rejected

- Putting timeline geometry constants directly inside individual Svelte components: rejected because it would make synchronized layout changes error-prone.
- Splitting each store next to every consumer component: rejected because the timeline/editor shell shares state across multiple panels.

## Invariants

- Backend-owned project data enters the UI through typed HTTP or Tauri IPC contracts instead of free-form objects.
- Shared timeline geometry values are defined once and reused by all dependent components.
- Stores own transient UI coordination; components render from store state rather than manual DOM mutation.

## Revisit Triggers

- A second frontend client needs a slimmer shared contract package.
- Timeline layout constants become large enough to justify a dedicated layout module.
- The app introduces SSR or multiple entrypoints that need different store composition roots.

## Dependencies

**Internal:** `ui/src/routes`, `ui/src/app.html`, Rust backend APIs exposed through `api.ts`, `commandApi.ts`, `projectionApi.ts`, and Tauri commands in `desktopTransport.ts`.
**External:** Svelte 5, SvelteKit, Vite.

## Related ADRs

- None identified as of 2026-03-08.
- Reason: the current UI structure changes are local to the frontend module boundary.
- Revisit trigger: a future change alters frontend/backend contract ownership or splits the timeline UI into separate packages.

## Usage Examples

```ts
import { mainTimelinePanelHeightPx } from '$lib/types.js';
import { timelineState } from '$lib/stores/timeline.svelte.js';
import { refreshTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';

const fixedTimelineHeight = mainTimelinePanelHeightPx();

async function openTimeline() {
  timelineState.scrollX = 0;
  await refreshTimelineRenderProjection();
}
```

## API Consumer Contract

- Internal consumers import typed shapes and helpers from `$lib/*`.
- Store consumers should treat backend-backed entities as read-through state and mutate them through API/store actions, not local object surgery.
- Command helpers return backend projections and must not patch persistent stores optimistically.
- Desktop command helpers return backend projections through Tauri IPC and must fall back only while Milestone 7 keeps legacy HTTP parity paths.
- Projection helpers are read-only and return backend-owned versioned read models.
- Layout consumers should reuse exported constants/helpers instead of re-declaring pixel budgets in component-local CSS.
- Compatibility is maintained by updating this directory README or an ADR whenever shared contracts materially change.

## Structured Producer Contract

- `types.ts` exports stable field names that mirror backend timeline/story payloads used throughout the UI.
- UI layout helper exports define default semantics for fixed panel sizing; consumers should treat them as the canonical budget for the main timeline shell.
- When a shared shape or layout helper changes, dependent components must be updated in the same change to preserve visual and type consistency.
