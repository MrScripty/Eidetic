# ui/src/lib/components/editor

## Purpose
This directory contains the main beat and script editing workflow, including AI generation context, consistency suggestions, and extraction-related UI.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `BeatEditor.svelte` | Primary node editing surface for notes, script, generation, and context panels. |
| `BeatPlanEditor.svelte` | Child-beat planning editor. |
| `ScriptPanel.svelte` | Container for script-editing surfaces. |
| `ScriptView.svelte` | Read-only screenplay rendering. |
| `DiffView.svelte` | Suggestion diff/acceptance rendering. |

## Problem
The app needs one editing surface where timeline selection, AI generation, script refinement, and consistency review all converge.

## Constraints
- Editor interactions depend on shared stores and websocket events.
- `BeatEditor.svelte` exceeds the preferred size threshold documented in `ADR-001`.
- Keyboard and accessibility behavior must remain intact across split points.

## Decision
Keep the current editor entrypoints stable and track `BeatEditor.svelte` for a follow-up split into context, extraction, and script/generation subpanels.

## Alternatives Rejected
- Splitting the editor during the standards pass: rejected because behavior correctness and accessibility fixes had higher priority.

## Invariants
- Timeline selection remains the single source of truth for the active editor node.
- Script editing and AI generation keep sharing the same editor state store.
- Future decomposition preserves current user-facing editor workflows.

## Revisit Triggers
- A change touches both prompt-context rendering and script editing in the same component.
- Another editor mode needs only a subset of the current `BeatEditor.svelte` responsibilities.

## Dependencies
**Internal:** `ui/src/lib/stores`, `ui/src/lib/api.ts`, `ui/src/lib/yjs.ts`.
**External:** Svelte.

## Related ADRs
- `ADR-001` decomposition baseline for `BeatEditor.svelte`.

## Usage Examples
```svelte
<BeatEditor />
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: these are internal UI components.
- Revisit trigger: editor panels become plugin points or external package exports.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: editor components consume server/store contracts rather than publishing them.
- Revisit trigger: the editor starts emitting reusable saved templates or machine-consumed schemas.
