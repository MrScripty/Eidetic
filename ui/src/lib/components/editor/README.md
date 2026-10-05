# ui/src/lib/components/editor

## Purpose
This directory contains the main beat and script viewing workflow, including AI generation context and projection-backed script display.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `createSelectedTimelineChild.ts` | Existing canonical create-child command and guarded selection after acknowledgement. |
| `createSelectedTimelineChild.svelte.test.ts` | Delayed selection, same-ID session reset and unmounted editor acknowledgement regressions. |
| `BeatEditor.svelte` | Primary node editing surface for notes, generation, and context panels. |
| `contextRequestLifecycle.ts` | Prompt request invalidation, loading cleanup and stale-response guards. |
| `contextRequestLifecycle.test.ts` | Empty notes/selection invalidation and delayed response ordering. |
| `BeatChildContext.svelte` | Context panel for child nodes, including parent, siblings, and adjacent parent-level nodes. |
| `BeatEditorHeader.svelte` | Header controls for lock state and AI generation. |
| `BeatNotesPanel.svelte` | Notes editor, generation status, and prompt preview container. |
| `BeatPlanningActions.svelte` | Parent-node child-planning action controls. |
| `AiPromptPreview.svelte` | Raw AI prompt context preview. |
| `ScriptPanel.svelte` | Script surfaces with canonical source links, same-node placement refresh and explicit failed-read recovery. |
| `scriptCreationSource.ts` | Creation-source range agreement and bounded selected-projection refresh after retiming. |
| `scriptCreationSource.svelte.test.ts` | Actual frontend range command, same-node placement retry, delayed reads/resize, version/error and clear guards with invoke/SSR fixtures. |
| `ScriptSegmentSource.svelte` | Canonical source clip name/range, explicit Go to clip, read failure and missing/standalone states. |
| `scriptSourceNavigation.ts` | Read-only source lookup and transient selection/scroll bridge using existing projection request guards. |
| `scriptSourceNavigation.svelte.test.ts` | Actual source lookup/range command/navigation functions, exact draft preservation, missing/read/race/clear and SSR fixtures. |
| `ScriptBlockComposer.svelte` | Selected-context first-block and append writing surface. |
| `scriptBlockCreationDraft.svelte.ts` | Transient captured-context draft, retry identity and explicit placement refresh controller. |
| `ScriptBlockComposer.ssr.test.ts` | Rendering and authoring discoverability fixtures. |
| `scriptBlockCreationDraft.svelte.test.ts` | Refused/ambiguous save, exact text and captured-context frontend fixtures. |
| `scriptBlockCreationRetry.svelte.test.ts` | Lost acknowledgement, exact retry, guarded discard/retarget and definite native placement refusal through actual frontend command/store functions with an invoke fixture. |
| `ScriptBlockComposer.retry.ssr.test.ts` | Uncertain-save read-only controls, exact retry action and definite-refusal recovery rendering fixtures. |
| `ScriptBlockEditor.svelte` | Session-owned per-block edit/save/cancel surface with exact uncertain retry and explicit reload. |
| `scriptBlockEditDraft.svelte.ts` | Captured draft, immutable pending edit, read comparison and explicit continuation from that version. |
| `scriptBlockEditComparison.svelte.test.ts` | Exact draft/current-text comparison, explicit version continuation, stale/ABA/read failures, navigation, immutable retry and retired-session fixtures. |
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

- Screenplay context changes use the existing Needs review and targeted proposal
  controls. What changed identifies scenes entering/leaving the captured window;
  the saved block and any active manual draft stay under existing save/accept
  guards. A context cause does not authorize automatic replacement.
- Comparing saved text reads its exact text/version without changing draft text,
  base revision or canon. Explicit continuation uses the private read snapshot
  only when the current block still has that revision. Changed/ABA revisions
  require another comparison; backend expected-revision and lock validation
  remains authoritative when Save is later requested. Ambiguous saves disable
  comparison/continuation until exact retry is reconciled. Comparison snapshots
  and exact drafts survive normal workspace navigation; read failures retain
  drafts without admitting unread revisions.
- Timeline range changes refresh the selected-node projection even when its ID
  stays the same. Creation/recovery consumes a source only when its range agrees
  with the canonical timeline clip. A late pre-move read or second resize cannot
  expose stale recovery times; existing request/version guards retain ownership.
  Read completion is untracked by the range effect to avoid self-retrying loops.
  Returning to a cached matching placement while a retime read is pending still
  supersedes that request; a delayed intermediate placement cannot replace the
  returned placement. Failed reads and stale placements retain the draft and
  expose explicit Refresh selected clip, including when an older response is
  rejected after clearing a read error. The button is disabled while a read is
  pending and disappears once the selected source agrees with the timeline.
- Source links display current canonical timeline names/ranges and recheck clip
  availability at activation. Go to clip changes only transient selection and
  scroll, then reads its focused projection. It preserves exact editing/creation
  drafts, their captured revisions and uncertain payload/IDs. Missing sources
  and standalone screenplay remain explicit; navigation errors expose retry.
  Existing request/clear guards prevent a late read from replacing a later
  selection or a cleared session. No fictional-time mapping or durable write is
  inferred from a source link.
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

### Bible fact review causes

ScriptImpactNotice and ScriptImpactReview show consumed Bible field changes with
their part/key identity through the existing preview, reject and explicit Accept
update surface. A Bible change refreshes the canonical screenplay projection and
invalidates prompt context. This surface preserves existing authoring draft owners
and never replaces saved screenplay on a fact edit or a preview response.


Selected-parent authoring exposes Add Scene (or the corresponding child level)
through the existing canonical create-child command, then selects only that
acknowledged child. createSelectedTimelineChild retains the user's later selection
and existing editor session generation across delayed acknowledgements/unmounts.
The backend derives child hierarchy and placement; frontend controls do not create
an alternate story owner. Immediate generation admission errors clear streaming
state and remain visible without discarding authored screenplay drafts.

ScriptImpactNotice distinguishes Bible membership ContextChanged causes from screenplay continuity changes. Existing preview and explicit acceptance remain the only proposal replacement path; saved human text and retained drafts are preserved.
