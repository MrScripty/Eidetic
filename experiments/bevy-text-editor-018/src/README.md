# Source

## Purpose

This directory contains the standalone Bevy text editor experiment executable
and experiment-local implementation modules.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `main.rs` | Bevy app entry point, window setup, and initial debug text surface. |

## Problem

The source code must isolate Bevy rendering and input behavior so the text
surface can be evaluated without production Eidetic architecture changes.

## Constraints

- Keep document and layout logic free of Bevy types where practical.
- Keep Bevy-specific code at the app/rendering boundary.
- Do not read or write production Eidetic project data.
- Keep source modules small enough that each editor subsystem remains testable.

## Decision

Start with a single executable entry point for the project shell. Split into
document, parser, layout, renderer, input, metrics, and fixture modules as each
thin vertical slice lands.

## Alternatives Rejected

- Immediate multi-crate structure: rejected because the experiment needs a fast
  vertical slice before production boundaries are known.
- Reusing production crates: rejected because this spike should not influence
  existing build, launcher, or dependency boundaries.

## Invariants

- The document model remains experiment-local until an ADR approves production
  integration.
- Bevy entities remain disposable render projections.
- File IO is limited to fixture loading and optional local stress fixture
  generation.

## Revisit Triggers

- A source file exceeds the standards threshold for decomposition.
- A module starts exposing reusable contracts to production code.
- The experiment adopts asynchronous loading, background tasks, or IPC.

## Dependencies

**Internal:** None at project creation.

**External:** Bevy for the direct native window and render loop.

## Related ADRs

- None identified as of 2026-05-23.
- Reason: this directory is experiment-only.
- Revisit trigger: experiment output becomes a production architecture decision.

## Usage Examples

Run the executable from the experiment root:

```bash
cargo run
```

## API Consumer Contract

- No external API consumer contract exists.
- Reason: this directory currently builds only a standalone binary.
- Revisit trigger: code is imported by another crate or moved into production.

## Structured Producer Contract

- No structured producer contract exists.
- Reason: runtime metrics are displayed locally and are not consumed by tools.
- Revisit trigger: metrics or benchmark output become stable files consumed by
  CI or decision reports.

