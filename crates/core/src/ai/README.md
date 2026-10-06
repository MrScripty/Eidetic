# crates/core/src/ai

## Purpose
This directory packages the domain-side AI helpers that turn project state into generation requests, recap windows, and consistency-analysis inputs.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `backend.rs` | Shared request/response shapes used by server-side AI backends. |
| `prompt.rs` | Prompt assembly for generation and child-planning flows. |
| `consistency.rs` | Diff-friendly consistency analysis helpers. |
| `helpers.rs` | Shared recap and neighboring-node extraction utilities. |

## Problem
AI backends need a consistent, domain-aware input shape so prompt behavior stays aligned across local and remote providers.

## Constraints
- Prompt inputs must be derived from authoritative project state.
- The server should consume prepared request shapes rather than reimplementing story logic.

## Decision
Keep prompt request assembly in core, where it can reuse timeline/story rules directly and stay testable without HTTP or backend adapters.

## Alternatives Rejected
- Building prompts in the server routes: rejected because route code should stay transport-focused.
- Building prompts in the UI: rejected because backend-owned project state belongs with the domain model.

## Invariants

- Backend-attached main-screenplay context can carry an optional complete-window
  receipt beside its exact blocks. The provider prompt uses authored evidence;
  existing generation history retains the ordered selection and placement epoch
  for later review. Legacy requests omit this receipt, and selection ranges do
  not infer fictional story time.
- GenerateRequest can carry backend-hydrated canonical screenplay evidence with
  exact source block/segment IDs and write events. Core request construction alone
  leaves this field unset; backend preview and generation share its hydration.
- Prompt helpers consume domain types rather than raw JSON fragments.
- Backend adapters treat these request shapes as the canonical source for generation inputs.

## Revisit Triggers
- Prompt assembly needs provider-specific branching that no longer fits one host-agnostic layer.
- `prompt.rs` or related helpers cross another major responsibility boundary listed in `ADR-001`.

## Dependencies
**Internal:** `crates/core/src/story`, `crates/core/src/timeline`, `crates/core/src/project`.
**External:** None beyond shared crate dependencies.

- Reason: request types use shared `serde`/`futures` traits; provider transport lives in server adapters.
- Revisit trigger: core AI helpers require a provider SDK or another module-specific external dependency.

## Related ADRs
- `ADR-001` decomposition baseline for oversized prompt-related modules.

## Usage Examples
```rust
use eidetic_core::ai::prompt::build_generate_request;
```

## API Consumer Contract
- ChildPlan and GenerateChildrenRequest optionally carry existing ScriptContextBlock receipts. Absence means legacy unknown evidence; an empty supplied array is a known empty selection. Consumers do not infer story facts or authorize screenplay replacement from these receipts.
- ChildPlan optionally carries ChildPlanBibleContext: the original resolved Bible projection and untimed BibleFieldInput values supplied to the provider. Missing evidence is legacy unknown; a supplied empty projection is known empty. Timed snapshots and withheld facts remain in the resolved projection and must not be relabelled as baseline field consumption. Recovery returns this original receipt, never current facts.
- None identified as of 2026-03-08.
- Reason: callers are internal Rust modules, not external clients.
- Revisit trigger: request types here become part of a published SDK or binding.

## Structured Producer Contract
- `backend.rs` defines the stable request/response shapes consumed by server-side AI adapters.
- Field semantics must stay aligned with backend adapters and frontend progress rendering.
- Changes to request defaults or enum meanings require coordinated server and test updates.

### Bible field evidence custody

GenerateRequest carries the canonical Bible projection and the untimed field
inputs captured in the same SQLite read snapshot. Generation forwards these
inputs unchanged through model I/O to the existing screenplay commit. Prompt
formatting remains driven by the resolved Bible context; lineage never substitutes
a baseline value for a timed override or unresolved fact.


Canonical generation carries optional ScriptGenerationTarget custody in the
existing request and generation command. Timeline/output/segment revisions plus
exact admitted notes and placement protect delayed completion and ABA; older
serialized requests/commands omit the receipt without invented backfill. The
backend enforces it, while canonical facts and complete-window lineage remain
independent consumed evidence. No new persistent memory owner or dependency.

GenerateRequest forwards optional BibleContextScope captured with canonical BibleFieldInput evidence; generation never replaces a late receipt with current membership.

Screenplay requests capture BibleRelationshipInput with the Bible projection in
one read snapshot. Runtime persistence forwards those original inputs, including
a known empty set, without rebinding late output to current relationships.

Canonical screenplay generation requests also carry optional
`BibleNodeNameInput` receipts for authored names in resolved Bible headers.
These accompany existing field and relationship inputs; downstream persistence
retains the captured name revision rather than rebinding late output to latest.
Legacy requests omit receipts and do not infer past name consumption.
