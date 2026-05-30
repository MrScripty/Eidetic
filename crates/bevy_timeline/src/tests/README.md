# crates/bevy_timeline/src/tests

## Purpose

This directory contains focused tests for the Bevy timeline renderer.

## Contents

| File | Description |
| ---- | ----------- |
| `app.rs` | Headless renderer app behavior. |
| `app_command.rs` | Renderer command validation. |
| `native_command.rs` | Native command helper behavior. |
| `native_lifecycle.rs` | Native window control and lifecycle tests. |
| `native_navigation.rs` | Native viewport and playhead navigation tests. |
| `native_visual.rs` | Native scene rebuild and visual tests. |
| `split.rs` | Timeline split command tests. |

## Problem

Renderer behavior spans projection math, command validation, and optional
native-window lifecycle, so tests need named groupings by reasoning boundary.

## Constraints

- Default tests must run without a native display.
- Display-required Winit checks must be explicit ignored smoke tests.
- Tests may use crate-private helpers to preserve renderer invariants.

## Decision

Group tests by renderer concern instead of by production file name alone.

## Alternatives Rejected

- One large test module: rejected because renderer behavior has distinct
  invariants.
- Running display creation in default tests: rejected because CI is headless by
  default.

## Invariants

- Headless tests cover command, projection, and resource behavior.
- Ignored smoke tests document display requirements explicitly.
- Test fixtures should stay local to the concern they verify.

## Revisit Triggers

- CI gains a virtual display strategy.
- Renderer app and native window lifecycles are separated further.
- Shared graph/timeline renderer fixtures emerge.

## Dependencies

**Internal:** Parent `crates/bevy_timeline/src` modules and `eidetic-core`.
**External:** Bevy test utilities through the renderer crate.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```bash
cargo test -p eidetic-bevy-timeline
```

## API Consumer Contract

- None for external consumers.
- Reason: this directory contains tests only.
- Revisit trigger: test fixtures become shared support APIs.

## Structured Producer Contract

- None for production payloads.
- Reason: tests assert structured renderer outputs but do not produce runtime
  artifacts.
- Revisit trigger: snapshot artifacts are introduced.
