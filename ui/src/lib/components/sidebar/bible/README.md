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
| `BibleGraphEdgeList.svelte`        | Read-only incoming/outgoing edge list for graph node detail projections.       |
| `BibleGraphNodeCard.svelte`        | List-card summary for backend-owned bible graph nodes.                         |
| `BibleGraphNodeDetail.svelte`      | Detail panel for backend-owned bible graph node projections and commands.      |
| `BibleGraphPartFields.svelte`      | Projection-backed bible graph field editor that writes through graph commands. |
| `BibleRenderGraphOutline.svelte`   | Keyboard-accessible graph-node outline from bible render graph projections.    |
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
