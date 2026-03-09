# ui/src/lib/components

## Purpose
This directory groups Eidetic’s reusable frontend feature components by UI concern: editor, layout, relationship views, sidebar panels, and timeline rendering.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `editor/` | Beat editing, script editing, and AI-generation UI. |
| `layout/` | Application shell, panel layout, and splash/toast chrome. |
| `relationship/` | Relationship graph and relationship-focused controls. |
| `sidebar/` | Sidebar panels for arcs, AI config, references, and bible data. |
| `timeline/` | Timeline rendering and clip-level interaction components. |

## Problem
The frontend needs reusable feature boundaries that stop the top-level app shell from becoming one giant component tree.

## Constraints
- Many components share stores instead of parent-child prop chains.
- Several components already approach or exceed size thresholds captured in `ADR-001`.

## Decision
Organize components by UI concern and document oversized sub-areas so future splits preserve the current app shell surface.

## Alternatives Rejected
- Keeping all components in one flat directory: rejected because feature ownership becomes opaque too quickly.

## Invariants
- Feature directories remain the primary component ownership boundary.
- Oversized components should be split within their feature directories rather than moving responsibilities across unrelated areas.

## Revisit Triggers
- A new feature area appears that does not fit the current editor/layout/relationship/sidebar/timeline split.

## Dependencies
**Internal:** `ui/src/lib/stores`, `ui/src/lib/types.ts`, `ui/src/routes`.
**External:** Svelte 5.

## Related ADRs
- `ADR-001` decomposition baseline for oversized frontend components.

## Usage Examples
```svelte
<AppShell />
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: these are internal frontend components.
- Revisit trigger: components are published as a shared UI package.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: this directory renders structured data but does not define the canonical schema.
- Revisit trigger: a component directory starts publishing reusable schema/config artifacts.
