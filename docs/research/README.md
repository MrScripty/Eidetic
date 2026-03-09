# docs/research

## Purpose
This directory stores external-product research used to validate Eidetic’s differentiation and feature tradeoffs against incumbent writing tools.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `celtx-screenwriting-capabilities.md` | Structured notes on Celtx’s current screenwriting workflow and constraints. |
| `final-draft-screenwriting-capabilities.md` | Structured notes on Final Draft’s current workflow surface. |
| `eidetic-vs-celtx-vs-final-draft.md` | Cross-product comparison that translates market findings into product direction. |

## Problem
Product direction should be based on captured evidence rather than memory of competitor behavior.

## Constraints
- Research should remain source-attributed and separate from implementation docs.
- Findings need stable filenames so future planning can reference them directly.

## Decision
Keep competitive analysis as checked-in markdown so design and roadmap work can cite a shared baseline.

## Alternatives Rejected
- Storing research only in chat transcripts: rejected because it is not durable or reviewable in the repo.

## Invariants
- Research files describe external tools, not Eidetic implementation details.
- Comparative conclusions should remain traceable to the source-specific notes in this directory.

## Revisit Triggers
- A competitor meaningfully changes its workflow or pricing in a way that alters Eidetic’s positioning.

## Dependencies
**Internal:** `PLAN.md`, product planning discussions, and repo ADRs that cite market context.
**External:** Public product documentation for the researched tools.

## Related ADRs
- None identified as of 2026-03-08.
- Reason: these files provide product context rather than codebase decisions.
- Revisit trigger: research directly drives a durable architectural decision.

## Usage Examples
```md
See docs/research/eidetic-vs-celtx-vs-final-draft.md before expanding script-format scope.
```
