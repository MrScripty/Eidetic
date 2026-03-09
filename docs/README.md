# docs

## Purpose
This directory holds repo-level architecture notes and external product research that inform Eidetic’s feature direction and structural decisions.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `research/` | Competitive and capability research used to compare Eidetic against existing writing tools. |
| `adr/` | Architecture decision records for codebase-wide choices that should outlive individual code changes. |

## Problem
The codebase needs a durable place for rationale that does not belong inside a single source directory README, especially when one decision affects multiple crates or the frontend/server boundary.

## Constraints
- Research material should stay separate from executable source.
- Architecture records need stable paths so directory READMEs can reference them.

## Decision
Keep long-lived supporting material under `docs/`, split into research and ADRs so product context and engineering decisions do not mix.

## Alternatives Rejected
- Storing architecture notes only in source READMEs: rejected because cross-cutting decisions span multiple directories.
- Keeping competitive research in ad hoc external documents: rejected because it would drift away from implementation decisions.

## Invariants
- `docs/research/` is for external product/context material, not implementation runbooks.
- `docs/adr/` is for durable engineering decisions with change triggers.

## Revisit Triggers
- The repo adds operator runbooks, release checklists, or incident docs that need a separate top-level documentation category.

## Dependencies
**Internal:** Root README, source-directory READMEs, and ADR references.
**External:** None.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```md
- Link source-directory rationale to ../docs/adr/ADR-001-decomposition-baseline.md
- Keep market research under docs/research/
```
