# ui/src/routes

## Purpose
This directory contains the Svelte route entrypoints that mount Eidetic’s single-page application shell.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `+layout.svelte` | Route-level layout wrapper. |
| `+page.svelte` | Main page entrypoint that mounts the shell and backend event lifecycle. |

## Problem
The frontend needs stable route entrypoints that stay thin while still owning app bootstrap concerns.

## Constraints
- Route files should avoid accumulating feature logic that belongs in `$lib`.
- Backend event setup and teardown must remain deterministic from the page root.

## Decision
Keep route entrypoints small and delegate feature behavior into `$lib` components and stores.

## Alternatives Rejected
- Mounting feature logic directly inside route files: rejected because it obscures ownership and hurts reuse.

## Invariants
- `+page.svelte` remains the mount point for the shell and backend event handler setup.
- Route files stay thin wrappers over documented library boundaries.

## Revisit Triggers
- The app adds multiple route groups or authenticated shells that need distinct entrypoints.

## Dependencies
**Internal:** `ui/src/lib/components/layout`, `ui/src/lib/serverEventClient.ts`, `ui/src/lib/stores/wsHandlers.ts`.
**External:** SvelteKit.

## Related ADRs
- `ADR-001` decomposition baseline for oversized frontend modules.

## Usage Examples
```svelte
<AppShell />
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: these are route entrypoints, not an external API surface.
- Revisit trigger: route modules are exported as a reusable shell package.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: routes mount UI state but do not define the canonical schemas.
- Revisit trigger: routes start generating persisted route metadata or build-time manifests.
