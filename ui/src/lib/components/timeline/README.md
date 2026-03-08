# ui/src/lib/components/timeline

## Purpose
This directory contains the interactive timeline workspace: the main fixed-height timeline, per-level track rows, supporting rulers and structure chrome, relationship overlays, and the optional character timeline.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `Timeline.svelte` | Main timeline viewport that coordinates the ruler, relationship lane, track rows, structure bar, horizontal scrollbar, and context menu. |
| `LevelTrack.svelte` | A single story-level row that renders clips, gaps, and track-local interactions. |
| `TimelineToolbar.svelte` | Tool, snapping, zoom, and character timeline controls for the main timeline. |
| `RelationshipLayer.svelte` | Edge overlays drawn above track rows. |
| `CharacterTimeline.svelte` | Optional secondary bottom panel for character progression markers. |
| `TimeRuler.svelte` | Shared top ruler for playhead movement and time labels. |

## Problem
The timeline UI has to render a horizontally scrollable editing surface while keeping multiple vertical layers aligned: labels, relationship space, track rows, structure bar, and scrollbar chrome. It also has to stay usable when the shell constrains it to a fixed panel height.

## Constraints
- Timeline row and chrome heights must stay synchronized across multiple components.
- Horizontal scroll, zoom, playhead movement, and relationship overlays must remain aligned.
- Extra vertical content must overflow inside the timeline panel rather than resizing the shell.

## Decision
Use shared exported layout constants for row and chrome budgets, keep the main timeline panel fixed-height in the shell, and let the timeline content scroll vertically as one aligned label/content viewport when more rows are present than the default budget allows.

## Alternatives Rejected
- Separate independent vertical scroll containers for labels and content: rejected because they would require scroll synchronization and are easy to desynchronize.
- Hardcoding timeline chrome heights directly in each component: rejected because it would cause layout drift across ruler, rows, and structure chrome.

## Invariants
- Label rows and timeline rows share the same total row height.
- The main timeline viewport width is measured from the time-content column, not the fixed label column.
- Wheel zoom and horizontal scroll operate on the shared timeline store; vertical overflow stays inside the timeline content viewport.
- The default shell height is based on the five built-in timeline tracks.
- The timeline renders the clamped shell height it is given; it does not own the user-preferred timeline height state.

## Revisit Triggers
- Track virtualization becomes necessary for very large projects.
- The character timeline needs to share the same scrolling surface as the main timeline.
- Track groups, nested lanes, or new chrome layers change the fixed-height budget model.

## Dependencies
**Internal:** `$lib/stores/timeline.svelte.js`, `$lib/stores/characterTimeline.svelte.js`, `$lib/api.js`, `$lib/types.js`.
**External:** Svelte runtime and browser SVG/CSS features.

## Related ADRs
- None identified as of 2026-03-08.
- Reason: the current work preserves the existing timeline architecture and only changes its shell/layout budgeting.
- Revisit trigger: the timeline rendering model or panel ownership changes across package or process boundaries.

## Usage Examples
```svelte
<div class="timeline-panel" style="height: {mainTimelinePanelHeightPx()}px">
	<Timeline />
</div>
```

## API Consumer Contract
- `Timeline.svelte` is consumed as a full main-panel component and reads timeline state from shared stores.
- `LevelTrack.svelte` expects a `track`, optional `gaps`, and an `onconnectstart` callback from its parent viewport.
- Consumers should not override row/chrome heights locally; use shared constants/helpers from `$lib/types.js`.
- Shell/layout consumers are responsible for preserving preferred timeline height across viewport resize and passing only the rendered height budget down to this directory.
- The optional character timeline is a sibling panel, not part of the main timeline’s scroll surface.

## Structured Producer Contract
- Shared timeline layout helpers in `$lib/types.js` are the canonical producer for row and chrome sizing used by this directory.
- Consumers can rely on `mainTimelinePanelHeightPx()` representing the default five-track shell budget unless the README or helper contract changes.
- If timeline layout semantics change, update this README in the same change so downstream component consumers understand the new height/overflow rules.
