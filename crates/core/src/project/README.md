# crates/core/src/project

## Purpose
This directory holds the top-level project aggregate that ties timeline, story, and template-derived metadata into one serializable unit.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | The `Project` aggregate and project-level helpers. |

## Problem
The rest of the system needs one authoritative aggregate representing the entire editable script project.

## Constraints
- The aggregate must stay easy to serialize for persistence and transport.
- It must bridge timeline and story subsystems without introducing host-specific concerns.

## Decision
Keep the project aggregate in its own directory so crate consumers have one obvious entrypoint for full-project behavior.

## Alternatives Rejected
- Defining `Project` in `lib.rs`: rejected because the aggregate deserves its own boundary and documentation.

## Invariants
- `Project` remains the canonical aggregate for persistence and transport.
- Project-level changes must preserve compatibility with the server persistence layer.

## Revisit Triggers
- Project metadata grows into multiple files or versioned migration helpers.

## Dependencies
**Internal:** `timeline/`, `story/`, `template.rs`.
**External:** `serde`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```rust
use eidetic_core::Project;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: the aggregate is an internal library type.
- Revisit trigger: a public SDK starts constructing projects directly.

## Structured Producer Contract
- `Project` is the root structured artifact persisted by the server and mirrored by the UI.
- Field additions or semantic changes must preserve load/save compatibility or ship with migrations.
