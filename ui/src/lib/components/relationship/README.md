# ui/src/lib/components/relationship

## Purpose
This directory renders and manages relationship-focused views over the story graph.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `RelationshipGraph.svelte` | SVG graph rendering for entity relationships. |
| `RelationshipPanel.svelte` | Container panel that drives graph interaction. |

## Problem
Story relationships need a focused visualization separate from the main timeline so users can inspect structural links without editing beat content directly.

## Constraints
- Graph interaction must remain keyboard-accessible.
- Relationship data is derived from story entities managed elsewhere.

## Decision
Keep graph rendering and graph panel orchestration together in one focused component directory.

## Alternatives Rejected
- Rendering relationships inside the main timeline only: rejected because the graph view serves a different inspection mode.

## Invariants
- Relationship visualization stays downstream of story-entity state.
- Interaction handlers preserve keyboard and mouse parity.

## Revisit Triggers
- Relationship editing grows beyond a simple graph/panel split.

## Dependencies
**Internal:** `ui/src/lib/stores/story.svelte.ts`, `ui/src/lib/types.ts`.
**External:** Svelte, SVG rendering.

## Related ADRs
- `ADR-001` decomposition baseline for oversized frontend components.

## Usage Examples
```svelte
<RelationshipPanel />
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this directory is an internal UI feature area.
- Revisit trigger: graph components become reusable package exports.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: the graph renders relationship data but does not define the authoritative schema.
- Revisit trigger: graph layout metadata starts being persisted or exported.
