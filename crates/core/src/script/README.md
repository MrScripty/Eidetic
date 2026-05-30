# crates/core/src/script

## Purpose
This directory contains screenplay-format helpers used to interpret generated text, estimate page/runtime shape, and merge user/AI edits safely.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `element.rs` | Screenplay element classifications. |
| `format.rs` | Parsing and formatting helpers for screenplay text. |
| `merge.rs` | Merge helpers for edit reconciliation. |
| `mod.rs` | Script module exports. |

## Problem
AI-assisted writing and export features need screenplay-aware utilities instead of treating scripts as opaque text blobs.

## Constraints
- Formatting heuristics must stay fast enough for local editing loops.
- Merge behavior must preserve user edits when AI content changes.

## Decision
Keep screenplay-specific helpers together so prompt, editor, and export code reuse one text model.

## Alternatives Rejected
- Treating scripts as plain text only: rejected because screenplay-aware structure matters for editing and export.

## Invariants
- Script helpers remain independent from transport and persistence layers.
- Merge utilities preserve ordering and user-authored content boundaries as much as possible.

## Revisit Triggers
- Script import/export grows into multiple supported formats with distinct parsers.

## Dependencies
**Internal:** `crates/core/src/timeline`.
**External:** Uses only shared crate dependencies already owned by `eidetic-core`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```rust
use eidetic_core::script::format::estimate_page_count;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: consumers are internal modules and tests.
- Revisit trigger: script helpers become a published standalone package or binding.

## Structured Producer Contract
- Screenplay element classifications and merge semantics are consumed by export and editor layers.
- Changes to element labeling or merge ordering require synchronized downstream updates.
