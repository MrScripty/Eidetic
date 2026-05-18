# ui/src/lib/components/sidebar/bible

## Purpose

This directory contains the story-bible UI: entity cards, detail editing, development timelines, and extraction review.

## Contents

| File/Folder                   | Description                                                                    |
| ----------------------------- | ------------------------------------------------------------------------------ |
| `StoryBibleTab.svelte`        | Top-level story-bible panel backed by bible graph node-list projections.       |
| `BibleGraphNodeCard.svelte`   | List-card summary for backend-owned bible graph nodes.                         |
| `BibleGraphNodeDetail.svelte` | Read-only detail panel for backend-owned bible graph node projections.         |
| `EntityCard.svelte`           | List-card summary for bible entities.                                          |
| `EntityDetail.svelte`         | Full entity editing surface, including relations and category-specific fields. |
| `DevelopmentTimeline.svelte`  | Timeline view of entity development points.                                    |
| `EntityExtractPanel.svelte`   | Review/apply flow for AI-driven extraction results.                            |

## Problem

Narrative entities need a dedicated editing experience that is richer than inline timeline metadata and that supports both manual curation and AI-assisted extraction.

## Constraints

- Bible graph node state must stay aligned with server validation and persistence semantics.
- Legacy entity detail state remains during migration and must not become a second source of truth for new graph-node list behavior.
- `EntityDetail.svelte` exceeds the preferred size threshold tracked in `ADR-001`.
- Extraction review must preserve acceptance state without breaking Svelte ownership rules.

## Decision

Keep story-bible components together while moving list/navigation reads to backend-owned projections. Defer the `EntityDetail.svelte` split until shared field groups and relation editing can be extracted onto bible graph commands without changing the current UI contract.

## Alternatives Rejected

- Folding entity editing into generic sidebar components: rejected because the bible flow has deeper lifecycle and validation needs.

## Invariants

- Story-bible list/navigation reads come from backend-owned bible graph projections, not broad legacy entity caches.
- Entity detail edits remain backed by server APIs until they are replaced by bible graph commands, not local-only schema forks.
- Development points stay ordered by timeline semantics.
- Extraction review preserves explicit accept/reject state per suggestion.

## Revisit Triggers

- A change touches both generic entity fields and category-specific detail sections in the same component.
- Another consumer needs relation editing independently of the full detail panel.

## Dependencies

**Internal:** `ui/src/lib/stores/bibleGraphNodeProjection.svelte.ts`, `ui/src/lib/stores/bible.svelte.ts`, `ui/src/lib/types.ts`.
**External:** Svelte.

## Related ADRs

- `ADR-001` decomposition baseline for `EntityDetail.svelte`.

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
