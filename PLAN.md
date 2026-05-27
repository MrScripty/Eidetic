# Eidetic Plan

This file is an index for the active planning documents. The old browser-first
Axum/WebSocket/WASM plan has been retired. Eidetic is now planned as a
standalone Tauri desktop application with backend-owned SQLite state, Svelte as
a projection consumer, and app-managed floating Bevy renderer windows for the
bible graph and timeline.

## Source Of Truth

- Active implementation plan:
  `docs/refactors/eidetic-projection-architecture/final-plan.md`
- Supporting planning notes:
  - `plans/story-tracks.md`
  - `plans/story-bible-worldbuilding.md`
  - `plans/script-generation-model.md`
  - `plans/story-bible-3d-graph-view.md`
  - `plans/timeline-rendering-bevy.md`
  - `plans/architecture-blast-radius.md`

## Current Direction

- Backend-owned SQLite command/event/revision state is the only durable source
  of truth.
- Svelte owns projection caches, local drafts, focus, filters, and accessible
  command surfaces only.
- Bevy graph and timeline crates are leaf renderers. They consume versioned
  backend projections and emit validated command requests only.
- Tauri is the desktop composition root. It owns command/event transport,
  renderer window lifecycle, startup/shutdown, and task ownership.
- The production application does not use Axum, local HTTP/WebSocket runtime
  paths, browser-open startup, or WASM renderer bridges.

## Milestone Status

- Milestones 1-7: Completed. These established backend-owned contracts,
  SQLite history/projections, projection-only frontend ownership, and the Tauri
  desktop shell while removing the old Axum/WASM/browser runtime path.
- Milestone 8: Completed for graph/context projection delivery and the
  floating native renderer ownership foundation.
- Milestone 9: Completed for the current native 3D bible graph viewer/editor
  scope. Future graph refinements should be planned as new milestones instead
  of reopening Milestone 9.
- Milestone 10: Completed for the backend-owned agent harness and graph tool
  foundation.
- Milestone 11: Completed for affect model contracts, persistence,
  proposals, prompt integration, and timeline overlay projections.
- Milestone 12: In progress. This is the remaining major active milestone:
  finish the Bevy timeline renderer, keep Svelte accessibility alternatives,
  then remove the DOM/SVG timeline only after Bevy covers the target
  interactions.

## Retired Direction

The following are no longer active plan items:

- compiling Eidetic core or renderers to WASM as a product path
- serving the app through an Axum local web server
- using browser WebSocket delivery for production desktop state
- using a browser tab as the application shell
- embedding Bevy into a WebView child surface
- keeping DOM/SVG timeline rendering as a runtime fallback after Bevy timeline
  coverage is complete
