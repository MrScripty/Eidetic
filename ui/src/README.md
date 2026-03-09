# ui/src

## Purpose
This directory is the Svelte application root for Eidetic’s browser client: shared app styling, route entrypoints, and the `$lib` feature surface used by the UI shell.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `app.css` | Global frontend styling tokens and baseline page styles. |
| `app.html` | SvelteKit document shell. |
| `lib/` | Shared frontend contracts, components, and stores. |
| `routes/` | Route entrypoints for the single-page application. |

## Problem
The frontend needs one coherent source root where route entrypoints and shared UI code can evolve together.

## Constraints
- The app is served as static assets by the local server.
- Shared contracts in `$lib` must stay aligned with server payloads.

## Decision
Keep the Svelte entrypoint, global styles, and shared library boundary under `ui/src/` following standard SvelteKit structure.

## Alternatives Rejected
- Flattening routes and shared code into one directory: rejected because it would blur entrypoints and reusable UI boundaries.

## Invariants
- Route entrypoints stay thin and delegate feature behavior into `$lib`.
- Global styling remains in `app.css` rather than duplicated per route.

## Revisit Triggers
- The UI gains multiple route groups or a second app shell.

## Dependencies
**Internal:** `ui/src/lib`, `ui/src/routes`.
**External:** SvelteKit, Vite, Svelte.

## Related ADRs
- `ADR-001` decomposition baseline for oversized frontend modules.

## Usage Examples
```ts
import '$app/forms';
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this directory is an internal frontend source root.
- Revisit trigger: a second deployed frontend consumes these entrypoints as a package.

## Structured Producer Contract
- `app.css` defines shared styling tokens consumed by the route and component tree.
- Changes to route structure or global styles should preserve the current shell expectations unless updated in the same change.
