# crates/core/src/ai

## Purpose
This directory packages the domain-side AI helpers that turn project state into prompt context, recap windows, and consistency-analysis inputs.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `backend.rs` | Shared request/response shapes used by server-side AI backends. |
| `prompt.rs` | Prompt assembly for generation and child-planning flows. |
| `context.rs` | Budgeting and ranking helpers for prompt context packing. |
| `consistency.rs` | Diff-friendly consistency analysis helpers. |
| `helpers.rs` | Shared recap and neighboring-node extraction utilities. |

## Problem
AI backends need a consistent, domain-aware input shape so prompt behavior stays aligned across local and remote providers.

## Constraints
- Prompt inputs must be derived from authoritative project state.
- Token budgeting has to stay deterministic enough for tests.
- The server should consume prepared request shapes rather than reimplementing story logic.

## Decision
Keep prompt and context assembly in core, where it can reuse timeline/story rules directly and stay testable without HTTP or backend adapters.

## Alternatives Rejected
- Building prompts in the server routes: rejected because route code should stay transport-focused.
- Building prompts in the UI: rejected because backend-owned project state and token budgeting belong with the domain model.

## Invariants
- Prompt helpers consume domain types rather than raw JSON fragments.
- Context packing remains bounded and test-covered.
- Backend adapters treat these request shapes as the canonical source for generation inputs.

## Revisit Triggers
- Prompt assembly needs provider-specific branching that no longer fits one host-agnostic layer.
- `prompt.rs` or related helpers cross another major responsibility boundary listed in `ADR-001`.

## Dependencies
**Internal:** `crates/core/src/story`, `crates/core/src/timeline`, `crates/core/src/project`.
**External:** None beyond shared crate dependencies.

## Related ADRs
- `ADR-001` decomposition baseline for oversized prompt-related modules.

## Usage Examples
```rust
use eidetic_core::ai::prompt::build_generate_request;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: callers are internal Rust modules, not external clients.
- Revisit trigger: request types here become part of a published SDK or binding.

## Structured Producer Contract
- `backend.rs` defines the stable request/response shapes consumed by server-side AI adapters.
- Field semantics must stay aligned with backend adapters and frontend progress rendering.
- Changes to request defaults or enum meanings require coordinated server and test updates.
