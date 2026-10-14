# ui/src/lib/components/sidebar

## Purpose
This directory contains the right-hand control surfaces for arcs, change review, AI configuration, references, progression, and the nested story-bible panels.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `AiConfigPanel.svelte` | AI backend configuration controls with separate reference embedding selection. |
| `PumasSelection.svelte` | Installed Pumas model/profile selectors and explicit Load/Unload controls. |
| `pumasLifecycle.ts` / `pumasLifecycle.test.ts` | Typed unload outcome messages and exact ONNX/Ollama failure, already-unserved and completed-unload regressions. |
| `EmbeddingConfigPanel.svelte` | Independent disabled/Pumas/OpenAI-compatible embedding selection and authentication. |
| `ArcList.svelte` / `ArcDetail.svelte` | Story-arc browsing and editing. |
| `ChangeReviewPanel.svelte` / `changeReviewPanel.css` | Projection-backed change history review surface for accepted backend events and revision deltas. |
| `ReferencePanel.svelte` | Reference document management. |
| `ProgressionPanel.svelte` | Arc progression analysis display. |
| `bible/` | Story-bible and entity-detail panels. |

## Problem
The editor needs secondary control surfaces that expose story metadata and backend configuration without overloading the central beat editor.

## Constraints
- Sidebar panels share stores with the main editor shell.
- Large sidebar panels remain on the decomposition watchlist in `ADR-001`.

## Decision
Group all sidebar-facing panels here and track the larger configuration/detail panels for future focused splits instead of moving them across unrelated feature areas.

## Alternatives Rejected
- Folding sidebar behavior into `AppShell.svelte`: rejected because it would make the shell a dumping ground for unrelated panel logic.

## Invariants
- Sidebar panels remain downstream of shared store state and API actions.
- Story metadata editing and backend configuration stay distinct subareas.

## Revisit Triggers
- A sidebar panel adds a second configuration workflow to `AiConfigPanel.svelte`.
- Sidebar tabs become numerous enough to justify another nesting layer.

## Dependencies
**Internal:** `ui/src/lib/stores`, `ui/src/lib/api.ts`, `ui/src/lib/components/sidebar/bible`.
**External:** Svelte.

## Related ADRs
- `ADR-001` decomposition baseline for oversized sidebar panels.

## Usage Examples
```svelte
<ReferencePanel />
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: sidebar components are internal UI consumers of server contracts.
- Revisit trigger: sidebar panels become separately packaged UI modules.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: sidebar panels edit existing schemas rather than defining them.
- Revisit trigger: sidebar flows begin emitting reusable saved panel configurations.

The local Pumas selector obtains the producer catalog and runtime profiles through
the backend. A successful control-plane connection is insufficient for connected
chat status: the exact model/profile must be loaded and advertise chat capability.
Reference status shows indexing/ready/stale/failed, chunk count, dimensions and
model/load revision. Reindex rebuilds saved source text. Provider/model selection
changes schedule a rebuild when configuration is saved. Generation notifications
name consumed reference sources or display the retrieval error; the existing
context viewer includes the exact retrieved text actually sent to the model.

Unload reports completion only for `unloaded: true` without a provider error.
An already-unserved result is shown separately; `success: true` cannot override
an ONNX/Ollama unload error or become a successful unload message.
