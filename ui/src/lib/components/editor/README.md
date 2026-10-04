# ui/src/lib/components/editor

## Purpose
This directory contains the main beat and script viewing workflow, including AI generation context and projection-backed script display.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `BeatEditor.svelte` | Primary node editing surface for notes, generation, and context panels. |
| `contextRequestLifecycle.ts` | Prompt request invalidation, loading cleanup and stale-response guards. |
| `contextRequestLifecycle.test.ts` | Empty notes/selection invalidation and delayed response ordering. |
| `BeatChildContext.svelte` | Context panel for child nodes, including parent, siblings, and adjacent parent-level nodes. |
| `BeatEditorHeader.svelte` | Header controls for lock state and AI generation. |
| `BeatNotesPanel.svelte` | Notes editor, generation status, and prompt preview container. |
| `BeatPlanningActions.svelte` | Parent-node child-planning action controls. |
| `AiPromptPreview.svelte` | Raw AI prompt context preview. |
| `ScriptPanel.svelte` | Container for script-editing surfaces. |
| `ScriptBlockComposer.svelte` | Selected-context first-block and append writing surface. |
| `scriptBlockCreationDraft.svelte.ts` | Transient captured-context draft, retry identity and explicit placement refresh controller. |
| `ScriptBlockComposer.ssr.test.ts` | Rendering and authoring discoverability fixtures. |
| `scriptBlockCreationDraft.svelte.test.ts` | Refused/ambiguous save, exact text and captured-context frontend fixtures. |
| `scriptBlockCreationRetry.svelte.test.ts` | Lost acknowledgement, exact retry, guarded discard/retarget and definite native placement refusal through actual frontend command/store functions with an invoke fixture. |
| `ScriptBlockComposer.retry.ssr.test.ts` | Uncertain-save read-only controls, exact retry action and definite-refusal recovery rendering fixtures. |
| `ScriptBlockEditor.svelte` | Session-owned per-block edit/save/cancel surface with exact uncertain retry and explicit reload. |
| `scriptBlockEditDraft.svelte.ts` | Captured-revision text draft, immutable pending edit and definite-refusal recovery controller. |
| `scriptBlockEditLifetime.ssr.test.ts` | Fresh workspace consumers, independent Unicode drafts, delayed acknowledgement, exact command replay, refusal/reload and retired-session fixture regressions. |
| `ScriptImpactNotice.svelte` | Read-only Needs review notice with changed/deleted input explanations and historical excerpts. |
| `ScriptImpactReview.svelte` | Targeted provider preview and existing propagation proposal text review with explicit accept/reject actions. |
| `scriptImpactNotice.ts` | Human-readable labels for the typed input review causes. |
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
- Existing-block edits survive Script/Graph/Split navigation with exact text and
  their captured block revision. Canonical updates and selection changes never
  reset a live draft. Uncertain saves expose Retry same save and disable editing,
  cancel and reload until reconciliation or a known definite native refusal.
  Reload failures keep the draft; successful explicit reload discards it.
- Script exposes Write screenplay even before a document exists. The session-owned
  composer draft captures its selected clip and document when writing begins, preserves
  exact refused drafts, and never retargets them on selection change. A placement
  refresh requires an explicit action for the original clip.
- The composer consumes the project-session creation draft. Script view removal
  and replacement retain an uncertain save's exact submitted payload and command
  ID; project activation resets its owner. Exact retry reconciles that request; editing text/kind, discard,
  restart and placement refresh remain disabled until acknowledgement or the
  known native placement-changed refusal. Mutable draft fields never replace the
  submission during retry. Native `bad_request` provenance and the exact known
  pre-recording message are required for placement recovery; lookalike text,
  internal errors and untyped failures remain uncertain.
- Block creation and existing-block editing share canonical projection updates.
- Review targets the block identified by the generation impact. Display current
  and exact proposed text; accept/reject explicitly and refresh the canonical
  script projection afterward. Manual editor drafts remain session-owned and intact.
  A refusal keeps the proposal available for rejection or a fresh preview.
- Timeline selection remains the single source of truth for the active editor node.
- AI generation state remains transient frontend state; durable script text is read from script document projections.
- Needs review is supplied by the backend projection and rendered alongside
  authored text. It provides no automatic edit, proposal acceptance or dismissal
  command. Deleted-source excerpts retain consumed evidence for human review.
- Manual screenplay drafts remain local until canonical save succeeds. Refused
  edits retain the draft and expose explicit discard/reload. Script revision
  invalidation refreshes prompt preview; earlier preview responses cannot replace
  a newer request for the same selected node.
- Clearing notes or selection invalidates the pending context request and clears
  loading immediately. Older successes, failures and finally callbacks cannot
  restore stale context or clear the loading state of a newer request.
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
