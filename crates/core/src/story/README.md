# crates/core/src/story

## Purpose
This directory models story arcs, character-facing context, and progression analysis over the episode timeline. Story bible data is owned by the projection graph contracts under `contracts/` rather than this legacy story module.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `arc.rs` | Story-arc identities, types, and color metadata. |
| `progression.rs` | Arc progression analysis over timeline state. |
| `character.rs` | Character-focused helper types for generated plans and timeline-adjacent flows. |

## Problem
The editor needs stable arc and progression models while bible/worldbuilding state moves through backend-owned graph commands and projections.

## Constraints
- Story-arc state must stay serializable for current project payloads.
- Bible graph state must not be reintroduced through broad project or timeline DTOs.

## Decision
Keep story-arc and progression analysis here. Put bible/worldbuilding structure in graph contracts and backend projection stores so it has one backend-owned source of truth.

## Alternatives Rejected
- Folding bible graph logic into timeline modules: rejected because worldbuilding graph state is broader than timeline placement.

## Invariants
- Arc types stay serializable for current project payloads.
- Bible graph state is accessed through graph command/projection contracts, not story module entity structs.

## Revisit Triggers
- A future change needs AI context from bible graph state.
- Story arcs move fully to command/projection storage and no longer belong in the broad project payload.

## Dependencies
**Internal:** `timeline/`, `project/`, `ai/`, `contracts/`.
**External:** `serde`, `uuid`.

## Related ADRs
- `docs/refactors/eidetic-projection-architecture/final-plan.md`.

## Usage Examples
```rust
use eidetic_core::story::arc::StoryArc;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this is an internal domain boundary.
- Revisit trigger: story types are exposed through standalone bindings or plugins.

## Structured Producer Contract
- Story arc DTO changes must land with server route, persistence, and frontend type updates until arcs move to command/projection storage.
