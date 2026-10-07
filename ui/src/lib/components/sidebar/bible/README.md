# ui/src/lib/components/sidebar/bible

## Purpose

This directory contains the story-bible UI backed by backend-owned bible graph projections.

## Contents

| File/Folder                        | Description                                                                    |
| ---------------------------------- | ------------------------------------------------------------------------------ |
| `StoryBibleTab.svelte`             | Top-level story-bible panel backed by bible graph node-list projections.       |
| `BibleGraphAddControls.svelte`     | Category-aware graph-node creation controls.                                   |
| `BibleGraphCategoryFilters.svelte` | Category filter controls for the graph-node list.                              |
| `BibleGraphEdgeEditor.svelte`      | Projection-backed edge creation form that writes through graph edge commands.  |
| `BibleGraphEdgeList.svelte`        | Incoming/outgoing edges with revision-guarded label editing and deletion.      |
| `BibleGraphNodeCard.svelte`        | List-card summary for backend-owned bible graph nodes.                         |
| `BibleGraphNodeDetail.svelte`      | Detail panel for backend-owned bible graph node projections and commands.      |
| `BibleGraphPartFields.svelte`      | Projection-backed bible graph field editor that writes through graph commands. |
| `BibleRenderGraphOutline.svelte`   | Keyboard-accessible graph-node outline from bible render graph projections.    |
| `bibleGraphFieldDrafts.svelte.ts`  | Local field draft bases, observed conflicts and owned save state.              |
| `bibleGraphCategories.ts`          | Category/root mapping helpers for graph-node list and creation UI.             |

## Problem

Narrative entities need a dedicated editing experience that is richer than inline timeline metadata and that supports both manual curation and AI-assisted extraction.

## Constraints

- Bible graph node state must stay aligned with server validation and persistence semantics.
- Legacy entity detail state must not become a second source of truth for graph-node list or detail behavior.

## Decision

Keep story-bible components together while routing list, detail, field, edge, and snapshot behavior through backend-owned graph projections and commands.

## Alternatives Rejected

- Folding entity editing into generic sidebar components: rejected because the bible flow has deeper lifecycle and validation needs.

## Invariants

- Story-bible list/navigation reads come from backend-owned bible graph projections, not broad legacy entity caches.
- Entity detail edits use bible graph commands, not broad legacy entity APIs or local-only schema forks.
- Development points are represented as graph snapshots.
- Canvas graph selection keeps semantic Svelte controls backed by the same backend render graph projection.

## Revisit Triggers

- A change touches both generic graph-node fields and category-specific detail sections in the same component.
- Another consumer needs relation editing independently of the full detail panel.

## Dependencies

**Internal:** `ui/src/lib/stores/bibleGraphNodeProjection.svelte.ts`, `ui/src/lib/stores/bible.svelte.ts`, `ui/src/lib/types.ts`.
**External:** Svelte.

## Related ADRs

- `ADR-001` decomposition baseline for oversized frontend components.

## Usage Examples

```svelte
<StoryBibleTab />
```

## API Consumer Contract

- None identified as of 2026-03-08.
- Reason: these are internal UI panels.
- Revisit trigger: entity/bible panels become externally packaged components.

## Structured Producer Contract

- None identified as of 2026-03-08.
- Reason: the directory edits story-bible data but does not own the canonical schema definition.
- Revisit trigger: entity templates or exported bible artifacts are generated from this boundary.

Existing relationship lists allow label-only editing through the canonical
set-edge command. Save retains endpoints, kind, direction and order; Cancel and
failed saves retain canonical screenplay. This is a narrow label editor, not a
new full relationship authoring workflow.

Label editors capture owned edge revision IDs from the node detail read. They submit only edge ID, label and expected revision; conflicting save/delete actions wait, and interrupted or removed-edge drafts remain editable until explicit cancellation. The server rechecks live identity and owned revision in its writer transaction.

`BibleRecall.svelte` adds an explicit “Recall related story facts” action to the
selected Bible entity detail, with optional fictional time and direction/kind
filters. It displays named neighbors, typed stored orientation, exact field and
relationship source revisions, unresolved fields and whole-record omissions.
Untimed associations are qualified as connectedness evidence, not truth. Input
changes revoke evidence without a fact-change notice; component disposal releases
the shared inspector owner, revoking only when no current inspector remains.
The effect tracks only its node ID, so projection updates do not retrigger cleanup.
No graph view or generation change is required. SSR and asynchronous read tests cover the visible evidence
and stale-result guards alongside preserved screenplay work.

Recall relationship paths render as one text expression, keeping the native
accessible label stable when source formatting wraps markup. The exact-path SSR
assertion runs after formatting and remains part of hosted qualification.

### Field drafts during accepted fact changes

`bibleGraphFieldDrafts.svelte.ts` owns only local field text, its original typed
value/identity base and save/error state, extracted from `BibleGraphPartFields`.
Clean inputs follow refreshed committed fields. Dirty inputs retain exact text
and original bases through shared node-detail refresh; an observed conflicting
saved value latches conflict even if a later value returns to the original.
The field shows its original base and current saved value and disables Save until
the writer explicitly discards the draft. Unrelated field drafts remain separate.
This is a frontend observed-value conflict guard; the existing field command
contract is unchanged, with no new claim of a revision/CAS writer guarantee.

Part components are keyed by node and part identity. Local save completion/error
may clear only the same admitted owner, and input is disabled during that save.
Late acknowledgement after a selection change cannot clear another field draft.
Completion additionally requires the same save request, the submitted typed value
in both the acknowledgement and current committed field, and no observed conflict.
A newer or mismatched committed value retains exact submitted text and its base
for explicit discard. Observed conflicts remain latched even if a later value
matches the submitted value. The expected pending save value alone is not
treated as an external conflict.
An owned matching acknowledgement may clear its submitted draft while a detail
read is pending; that read still gates the next Save until verified recovery.
No canonical text is optimistically changed. Existing trimming at explicit field
Save remains unchanged; unsaved input is retained exactly. Dirty draft persistence
across inspector destruction/project switching is not added.

The existing oversized node-detail component only delegates its read lifecycle;
new ownership and field state live in the extracted small modules. Tests exercise
actual client rune state, canonical refresh versus dirty bases, explicit discard,
interruption and rapid selection/delayed ownership. Native qualification must show
the clean accepted fact and a preserved dirty draft simultaneously.

Cached details remain visible during a failed refresh. The inspector displays the
failure and an ordinary Retry saved facts action. Field Save is gated in both its
button and handler until an owned, version-admitted read succeeds; clearing the
error at retry start does not restore permission. Drafts remain editable during
verification and failure, and recovery does not rebase them.

An initial uncached detail failure also exposes Retry saved facts in place.
Retry uses the same owned refresh, shows Loading while pending, and admits only
the current node/session response; selecting another node retires the old read.
