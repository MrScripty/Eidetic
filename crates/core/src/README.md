# crates/core/src

## Purpose
This directory contains the pure Rust domain layer for Eidetic: project aggregation, timeline structure, story arcs, screenplay helpers, projection contracts, and AI prompt/context logic without networking or UI concerns.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `lib.rs` | Public crate surface and module wiring for the core library. |
| `contracts/` | Host-agnostic command, event, revision, and projection contracts for backend-owned state. |
| `timeline/` | Timeline nodes, tracks, relationships, structure, and timing rules. |
| `story/` | Story arcs and progression analysis. |
| `script/` | Screenplay parsing, formatting, and merge helpers. |
| `ai/` | Prompt assembly, context packing, and consistency helpers. |

## Problem
The application needs a reusable domain layer that can serve both the local server and any future WASM or alternate frontend host without duplicating narrative logic.

## Constraints
- No direct HTTP, filesystem, or UI dependencies at this boundary.
- Timeline, story-arc, script, and projection contracts must remain serializable for persistence and transport.
- The library should remain suitable for future non-server hosts.

## Decision
Keep narrative behavior, data structures, and AI-facing domain helpers in one host-agnostic crate and push transport/persistence concerns into the server crate.

## Alternatives Rejected
- Folding domain logic into `crates/server/`: rejected because it would couple core behavior to one host.
- Moving prompt construction into the UI: rejected because prompt context depends on authoritative project state and shared domain rules.

## Invariants
- Core modules remain deterministic and host-agnostic.
- Project, timeline, story-arc, script, and projection types stay serializable across the server boundary.
- Cross-cutting decomposition decisions reference `ADR-001`.

## Revisit Triggers
- A second host requires a narrower published surface than the current crate.
- Domain behavior starts depending on direct IO or async services.
- Oversized modules listed in `ADR-001` gain another unrelated responsibility.

## Dependencies
**Internal:** `contracts/`, `timeline/`, `story/`, `script/`, `project/`, `ai/`.
**External:** `serde`, `uuid`, `thiserror`, `futures`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized core and UI modules.

## Usage Examples
```rust
use eidetic_core::Template;

let project = Template::MultiCam.build_project("Pilot".into());
assert!(!project.timeline.nodes.is_empty());
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this directory is an internal library boundary, not an out-of-process API surface.
- Revisit trigger: the crate is exposed through WASM bindings, an SDK, or another process boundary.

## Structured Producer Contract
- Core structs in this directory define the stable contract, project, timeline, story-arc, script, and projection shapes persisted by the server and mirrored by the UI.
- Compatibility-sensitive field changes must land with persistence, route, and frontend updates in the same change.
- Regeneration rules for saved projects are handled by the server persistence layer, not by ad hoc client migration.
