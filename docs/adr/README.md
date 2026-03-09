# docs/adr

## Purpose
This directory records architecture decisions that affect multiple source directories or verification workflows.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `ADR-001-decomposition-baseline.md` | Current decision on how Eidetic will handle oversized modules while preserving stable facades. |

## Problem
Some decisions are too cross-cutting to live only in a single directory README, but still need durable rationale and revisit triggers.

## Constraints
- ADRs should stay small, durable, and reference concrete triggers rather than aspirational future work.
- ADR numbering must remain stable once published.

## Decision
Use a lightweight `ADR-###` sequence under `docs/adr/` for cross-directory decisions referenced by source READMEs.

## Alternatives Rejected
- Embedding all cross-cutting rationale in `PLAN.md`: rejected because plans change more often than enduring architectural decisions.

## Invariants
- ADR filenames remain stable once referenced from source READMEs.
- ADRs capture active decisions plus the conditions that would invalidate them.

## Revisit Triggers
- The repo gains enough ADRs to need status tags or a separate index by domain.

## Dependencies
**Internal:** Source-directory READMEs under `crates/**/src` and `ui/src/**`.
**External:** None.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```md
See ADR-001 before splitting BeatEditor.svelte or persistence.rs.
```
