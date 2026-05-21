# Eidetic

Eidetic is a local-first, AI-assisted scriptwriting workspace built as a Tauri desktop app with a Rust backend runtime and a Svelte projection UI. The product model is timeline-first: the backend owns project state, persistence, and realtime synchronization, while the frontend renders editing, timeline, relationship, and story-bible workflows over those contracts.

## Workspace Layout

- `crates/core/` contains the domain model, timeline/story logic, screenplay helpers, and AI prompt/context assembly.
- `crates/server/` owns backend services, persistence, event production, and AI backend adapters consumed by the Tauri desktop shell.
- `src-tauri/` contains the desktop shell and Tauri command/event adapters over backend-owned services.
- `ui/` contains the Svelte application, shared client contracts, state stores, and frontend verification tooling.
- `docs/` holds product research and architecture/decomposition records.

## Canonical Workflows

```bash
./launcher.sh --install
./launcher.sh --build
./launcher.sh --release-smoke
./launcher.sh --run
./launcher.sh --test
```

The launcher isolates state by default under `.launcher-state/`. Set `EIDETIC_LAUNCHER_ISOLATE_STATE=0` only when intentionally using host state directories.
`--run` starts the Vite dev server for the Tauri webview and opens the desktop app; it no longer starts the legacy Axum browser server.
`--release-smoke` runs the release desktop binary with `--smoke`, initializes the backend runtime, reports JSON health, and exits without opening a window.

## Verification

- `./launcher.sh --test` is the canonical repo verification command.
- The frontend also exposes `npm run lint`, `npm run format:check`, `npm run check`, and `npm run test` from [`ui/package.json`](/media/jeremy/OrangeCream/Linux%20Software/repos/owned/creative-media/Eidetic/ui/package.json).
- Oversized-module decomposition posture is recorded in [`ADR-001`](/media/jeremy/OrangeCream/Linux%20Software/repos/owned/creative-media/Eidetic/docs/adr/ADR-001-decomposition-baseline.md).
