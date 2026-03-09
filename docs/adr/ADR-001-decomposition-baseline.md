# ADR-001: Decomposition Baseline For Oversized Modules

## Status
Accepted on 2026-03-08

## Context
The standards audit identified several files that exceed the project’s decomposition thresholds, including `crates/server/src/persistence.rs`, `crates/server/src/ydoc.rs`, `crates/server/src/routes/ai.rs`, `crates/core/src/story/bible.rs`, `crates/core/src/timeline/mod.rs`, `ui/src/lib/components/editor/BeatEditor.svelte`, `ui/src/lib/components/sidebar/bible/EntityDetail.svelte`, and `ui/src/lib/components/sidebar/AiConfigPanel.svelte`.

These modules are already carrying real behavior, tests, or user workflows. Splitting them immediately during the standards remediation would increase regression risk while other boundary fixes are still landing.

## Decision
Keep the current public facades and stabilize contract correctness first. Track oversized modules explicitly in directory READMEs, and split them only when the split can preserve existing entrypoints and be validated inside the same change.

The next decomposition targets, in order, are:

1. `crates/server/src/persistence.rs`: separate project metadata IO, SQLite persistence, and project listing.
2. `crates/server/src/ydoc.rs`: separate CRDT command handling, snapshot helpers, and persistence serialization.
3. `crates/core/src/story/bible.rs`: separate entity mutation, context assembly, and snapshot resolution/query logic.
4. `ui/src/lib/components/editor/BeatEditor.svelte`: separate context panels, extraction UI, and script editing/generation panels.
5. `ui/src/lib/components/sidebar/bible/EntityDetail.svelte`: separate shared field sections, relation editing, and category-specific detail panels.

## Consequences
- The repo stays compliant on documentation and revisit triggers immediately, even where implementation splits are deferred.
- Future structural refactors must preserve route names, persisted project compatibility, and current UI entrypoints.
- Oversized files should not accumulate additional unrelated responsibilities before their documented split.

## Revisit Triggers
- A listed file crosses another major feature boundary.
- A bug fix touches two or more unrelated responsibilities inside the same oversized file.
- A second caller or consumer would naturally reuse only one slice of the file’s current behavior.
