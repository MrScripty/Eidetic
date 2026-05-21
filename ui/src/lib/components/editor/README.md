# ui/src/lib/components/editor

## Purpose
This directory contains the main beat and script viewing workflow, including AI generation context and projection-backed script display.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `BeatEditor.svelte` | Primary node editing surface for notes, generation, and context panels. |
| `BeatChildContext.svelte` | Context panel for child nodes, including parent, siblings, and adjacent parent-level nodes. |
| `BeatEditorHeader.svelte` | Header controls for lock state and AI generation. |
| `BeatNotesPanel.svelte` | Notes editor, generation status, and prompt preview container. |
| `BeatPlanningActions.svelte` | Parent-node child-planning action controls. |
| `AiPromptPreview.svelte` | Raw AI prompt context preview. |
| `ScriptPanel.svelte` | Container for script-editing surfaces. |
| `ScriptView.svelte` | Read-only screenplay rendering. |

## Problem
The app needs focused editing surfaces where timeline selection, AI generation, and projection-backed script review stay coordinated.

## Constraints
- Editor interactions depend on shared stores and backend events.
- Editor Svelte components stay below the preferred size threshold documented in `ADR-001`.
- Keyboard and accessibility behavior must remain intact across split points.

## Decision
Keep `BeatEditor.svelte` as the orchestration entrypoint and split header, context, planning actions, notes, and prompt preview into focused components. Remove unused legacy planning components once durable backend child plans own planning state.

## Alternatives Rejected
- Splitting the editor during the standards pass: rejected because behavior correctness and accessibility fixes had higher priority.

## Invariants
- Timeline selection remains the single source of truth for the active editor node.
- AI generation state remains transient frontend state; durable script text is read from script document projections.
- Future decomposition preserves current user-facing editor workflows.

## Revisit Triggers
- A change touches both prompt-context rendering and script editing in the same component.
- Another editor mode needs only a subset of the current `BeatEditor.svelte` responsibilities.

## Dependencies
**Internal:** `ui/src/lib/stores`, `ui/src/lib/api.ts`.
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
