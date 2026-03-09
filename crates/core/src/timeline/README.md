# crates/core/src/timeline

## Purpose
This directory defines the timeline data model that underpins Eidetic’s clip-based story editor: nodes, tracks, relationships, structure bars, and timing rules.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | Timeline aggregate behavior and traversal helpers. |
| `node.rs` | Story node identity, hierarchy, and content-bearing types. |
| `track.rs` | Track metadata and ordering. |
| `relationship.rs` | Inter-node relationship types and IDs. |
| `structure.rs` | Episode structure/act segmentation metadata. |
| `timing.rs` | Time-range rules and helpers. |

## Problem
The editor’s timeline-first workflow needs a single authoritative representation of narrative clips, their hierarchy, and their temporal constraints.

## Constraints
- Time ranges and hierarchy rules must remain deterministic and testable.
- Timeline state is shared by persistence, AI context, and frontend rendering.
- `mod.rs` is above the preferred decomposition threshold captured in `ADR-001`.

## Decision
Keep the timeline model centralized here, with a documented future split of aggregate helpers if another major responsibility is added to `mod.rs`.

## Alternatives Rejected
- Spreading timing and relationship logic across unrelated modules: rejected because timeline invariants need one home.

## Invariants
- Time-range validation remains authoritative in this boundary.
- Track/node ordering semantics stay stable for persistence and frontend rendering.
- Hierarchy traversal continues to flow through timeline helpers rather than duplicated callers.

## Revisit Triggers
- A feature adds another major behavior layer to `mod.rs`.
- Multiple callers need only timing or traversal helpers without the full aggregate.

## Dependencies
**Internal:** `project/`, `story/`, `script/`.
**External:** `serde`, `uuid`.

## Related ADRs
- `ADR-001` decomposition baseline for `timeline/mod.rs`.

## Usage Examples
```rust
use eidetic_core::timeline::Timeline;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: callers are internal Rust modules.
- Revisit trigger: timeline types become a public binding surface.

## Structured Producer Contract
- Timeline nodes, tracks, structure segments, and relationships are persisted and mirrored by the UI.
- Ordering and enum semantics must stay stable unless accompanied by coordinated persistence and frontend updates.
