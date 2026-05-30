# ui/src/lib/components/relationship

## Purpose

This reserved directory marks the future boundary for relationship-specific UI
components if timeline and graph relationship editing need a shared component
surface.

## Contents

The directory currently has no Svelte or TypeScript source files.

## Problem

Relationship UI behavior may eventually need shared components, but current
relationship rendering and editing still live in timeline and graph-specific
boundaries.

## Constraints

- Empty reserved directories must not imply implemented UI behavior.
- Shared components should be added only when they reduce duplication without
  hiding domain-specific invariants.

## Decision

Document the reserved boundary and leave implementation in existing component
owners until a shared relationship workflow is justified.

## Alternatives Rejected

- Adding placeholder components: rejected because unused UI surfaces add
  maintenance cost.
- Moving existing timeline relationship components now: rejected because the
  standards pass should not refactor by directory shape alone.

## Invariants

- No route imports components from this directory while it is empty.
- Future shared components must keep graph and timeline command semantics
  explicit.

## Revisit Triggers

- Graph and timeline relationship editors share behavior.
- The directory remains empty after frontend accessibility work is complete.

## Dependencies

**Internal:** None at runtime because no source files exist.
**External:** None at runtime because no source files exist.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```text
No usage while the directory is reserved.
```

## API Consumer Contract

- None for consumers.
- Reason: no component API exists yet.
- Revisit trigger: shared relationship components are added.

## Structured Producer Contract

- None for producers.
- Reason: no structured component output exists yet.
- Revisit trigger: relationship components emit typed events.
