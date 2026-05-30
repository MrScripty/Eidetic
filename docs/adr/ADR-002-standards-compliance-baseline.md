# ADR-002: Standards Compliance Baseline

## Status

Accepted.

## Context

The standards audit found that Eidetic had working local tooling but incomplete
repository governance. The immediate remediation touches source files, launcher
behavior, tests, documentation, and future CI policy in one branch.

## Decision

Use `docs/plans/codebase-standards-compliance/plan.md` as the active execution
plan and treat this ADR as the traceability record for the baseline compliance
work. The branch is allowed to update multiple source roots while preserving the
current architecture and avoiding refactors justified only by file length.

The first completed slices establish a passing Rust clippy baseline, make the
default test launcher safe in headless environments, and add decision
traceability infrastructure before enabling CI enforcement.

## Consequences

- Source-root README changes can refer to this ADR for the standards baseline.
- Display-required Bevy/Winit smoke checks are explicit ignored tests; default
  tests cover state/resource behavior without requiring a display.
- The development launcher now requires the Vite dev server to be reachable
  before it `exec`s the desktop app process.
- Further compliance work should land in thin verified commits and update the
  active plan as gaps are found.
