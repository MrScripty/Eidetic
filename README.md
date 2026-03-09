# Eidetic

Eidetic is a local-first, AI-assisted scriptwriting workspace built around a Rust core, an Axum server, and a Svelte frontend. The product model is timeline-first: the backend owns project state, persistence, and realtime synchronization, while the frontend renders editing, timeline, relationship, and story-bible workflows over those contracts.

## Workspace Layout

- `crates/core/` contains the domain model, timeline/story logic, screenplay helpers, and AI prompt/context assembly.
- `crates/server/` exposes HTTP and WebSocket APIs, persistence, static asset serving, and AI backend adapters.
- `ui/` contains the Svelte application, shared client contracts, state stores, and frontend verification tooling.
- `docs/` holds product research and architecture/decomposition records.

## Canonical Workflows

```bash
./launcher.sh --install
./launcher.sh --build
./launcher.sh --run
./launcher.sh --test
```

The launcher isolates state by default under `.launcher-state/`. Set `EIDETIC_LAUNCHER_ISOLATE_STATE=0` only when intentionally using host state directories.

## Verification

- `./launcher.sh --test` is the canonical repo verification command.
- The frontend also exposes `npm run lint`, `npm run format:check`, `npm run check`, and `npm run test` from [`ui/package.json`](/media/jeremy/OrangeCream/Linux%20Software/Eidetic/ui/package.json).
- Oversized-module decomposition posture is recorded in [`ADR-001`](/media/jeremy/OrangeCream/Linux%20Software/Eidetic/docs/adr/ADR-001-decomposition-baseline.md).
