# crates/core/src/story

## Purpose
This directory models story arcs, the story bible, character-facing context, and progression analysis over the episode timeline.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `arc.rs` | Story-arc identities, types, and color metadata. |
| `bible.rs` | Entities, snapshots, relations, and bible-context assembly. |
| `progression.rs` | Arc progression analysis over timeline state. |
| `character.rs` | Character-focused helpers shared by bible and timeline flows. |

## Problem
The editor needs a stable narrative model for arcs, entities, and continuity checks that survives persistence and powers AI context.

## Constraints
- Story state must stay serializable and queryable by time.
- Entity and snapshot behavior must line up with timeline references and AI context packing.
- `bible.rs` already exceeds the decomposition threshold recorded in `ADR-001`.

## Decision
Keep story-specific concepts together and defer splitting `bible.rs` until its mutation, query, and context responsibilities can be separated without changing persisted semantics.

## Alternatives Rejected
- Folding bible logic into timeline modules: rejected because narrative entities are broader than timeline placement.
- Splitting `bible.rs` during the standards pass: rejected because contract stabilization was higher risk than documenting the split boundary first.

## Invariants
- Entity resolution by time remains compatible with saved snapshots.
- Arc and bible types stay serializable for persistence and UI mirroring.
- Decomposition work must preserve existing entity and relation semantics.

## Revisit Triggers
- A future change touches both bible mutation and query behavior in one patch.
- Another consumer needs bible context without the current full `bible.rs` surface.

## Dependencies
**Internal:** `timeline/`, `project/`, `ai/`.
**External:** `serde`, `uuid`.

## Related ADRs
- `ADR-001` decomposition baseline for `bible.rs`.

## Usage Examples
```rust
use eidetic_core::story::bible::StoryBible;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this is an internal domain boundary.
- Revisit trigger: story types are exposed through standalone bindings or plugins.

## Structured Producer Contract
- Story entities, snapshots, and relations form part of the saved project shape and frontend contract.
- Field additions or enum changes must preserve persistence compatibility or ship with coordinated migrations.
