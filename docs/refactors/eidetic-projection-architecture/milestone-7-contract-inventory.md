# Milestone 7 Contract Inventory

Milestone 7 replaces the production Axum HTTP/WebSocket boundary with Tauri
commands and events. This inventory freezes the current boundary surface before
service extraction starts so implementation can move in thin slices without
losing command/projection behavior.

## Backend Runtime Surface

Current composition root:

- `crates/server/src/main.rs` builds one Axum `Router`, nests `/api`, mounts
  `/ws`, serves `dist/ui`, applies CORS, binds `127.0.0.1:3000`, and owns
  `axum::serve`.
- `crates/server/src/state.rs` constructs `AppState`, event broadcast channels,
  the Y.Doc manager, autosave, active project database owner, AI config, vector
  store, and optional Pumas model library.
- `crates/server/src/error.rs` maps backend failures directly to HTTP status
  codes and Axum JSON responses.
- `crates/server/src/validation.rs` contains reusable validation helpers, but
  local-origin validation depends on Axum URI parsing and is deletion-target
  behavior for the Tauri production path.

Milestone 7 service extraction target:

- Introduce backend-owned runtime/service APIs that compile without Axum,
  Tauri, Svelte, or Bevy.
- Keep command handling, projection reads, persistence, AI gateway access,
  event production, export, Y.Doc operations, and history transactions inside
  backend services.
- Keep Tauri as the binding/composition layer that validates frontend IPC
  payloads, calls backend services, maps errors, and emits frontend events.

## HTTP Route Surface To Replace

Project and persistence:

- `POST /api/project`
- `GET /api/project`
- `PUT /api/project`
- `POST /api/project/save`
- `POST /api/project/load`
- `GET /api/project/list`

AI and model runtime:

- `POST /api/ai/generate`
- `POST /api/ai/generate-children`
- `POST /api/ai/generate-batch`
- `GET /api/ai/context/{id}`
- `GET /api/ai/status`
- `PUT /api/ai/config`
- `GET /api/models`

Commands:

- `POST /api/commands/object-field`
- `POST /api/commands/script/block`
- `POST /api/commands/script/lock`
- `POST /api/commands/story/create-arc`
- `POST /api/commands/story/update-arc`
- `POST /api/commands/story/delete-arc`
- `POST /api/commands/timeline/node-range`
- `POST /api/commands/timeline/create-node`
- `POST /api/commands/timeline/apply-children`
- `POST /api/commands/timeline/create-relationship`
- `POST /api/commands/timeline/delete-relationship`
- `POST /api/commands/timeline/split-node`
- `POST /api/commands/timeline/node-lock`
- `POST /api/commands/timeline/node-notes`
- `POST /api/commands/timeline/delete-node`
- `POST /api/commands/bible-graph/node`
- `POST /api/commands/bible-graph/field`
- `POST /api/commands/bible-graph/edge`
- `POST /api/commands/bible-graph/canonical-roots`
- `POST /api/commands/bible-graph/snapshot-field`
- `POST /api/commands/semantic/bible-reference-proposal`
- `POST /api/commands/semantic/bible-reference-proposal/reject`
- `POST /api/commands/semantic/bible-reference-proposal/accept`
- `POST /api/commands/semantic/dependency`
- `POST /api/commands/semantic/propagation-proposal`
- `POST /api/commands/semantic/propagation-proposal/reject`
- `POST /api/commands/semantic/propagation-proposal/accept`
- `POST /api/commands/semantic/propagation-proposal/update`
- `POST /api/commands/semantic/child-plan/reject`

Projections:

- `GET /api/projections/object-field`
- `GET /api/projections/bible-graph/node`
- `GET /api/projections/bible-graph/nodes`
- `GET /api/projections/bible-graph/schemas`
- `GET /api/projections/bible-graph/render`
- `GET /api/projections/history/changes`
- `GET /api/projections/script/document`
- `GET /api/projections/timeline/render`
- `GET /api/projections/timeline/selected-node`
- `GET /api/projections/story/arcs`
- `GET /api/projections/story/arc-progression`
- `GET /api/projections/semantic/bible-reference-proposals`
- `GET /api/projections/semantic/dependencies`
- `GET /api/projections/semantic/propagation-proposals`
- `GET /api/projections/semantic/child-plans`

References and export:

- `GET /api/references`
- `POST /api/references`
- `DELETE /api/references/{id}`
- `POST /api/export/pdf`

## Event Surface To Replace

Current `/ws` text events are `ServerEvent` values broadcast from backend
mutations:

- `connected`
- `timeline_changed`
- `hierarchy_changed`
- `story_changed`
- `node_updated`
- `generation_context`
- `generation_progress`
- `generation_complete`
- `generation_error`
- `bible_changed`
- `semantic_proposals_changed`
- `script_changed`

Current `/ws` binary frames are Y.Doc CRDT updates. They are mixed with app
events on the same WebSocket, but should be separated during the Tauri move:

- Backend projection invalidation and generation progress should become typed
  Tauri events or a bounded event-drain contract.
- Retained Y.Doc sync should become an explicit backend service/adapter
  contract with bounded buffering, teardown, and recovery semantics instead of
  being implicitly tied to WebSocket frame type.

## Frontend Consumers

HTTP helper seams:

- `ui/src/lib/api.ts`: project, references, AI, model listing, export, save/load.
- `ui/src/lib/commandApi.ts`: object, script, story, timeline, bible graph, and
  semantic command helpers.
- `ui/src/lib/projectionApi.ts`: object, bible graph, script, timeline, story,
  semantic, and change-review projection helpers.
- Proposal-specific tests assert concrete `/api` paths and should move to
  adapter-level command names instead of HTTP URL strings.

WebSocket/Y.Doc seams:

- `ui/src/lib/ws.ts` owns WebSocket connection, reconnect timer, text event
  dispatch, binary Y.Doc update dispatch, and outgoing Y.Doc update attachment.
- `ui/src/lib/stores/wsHandlers.ts` maps backend events to projection refreshes
  and generation-progress store updates.
- `ui/src/routes/+page.svelte` starts and tears down the current WebSocket client.
- `ui/src/lib/yjs.ts` documents binary WebSocket sync as the current transport.

Clean frontend migration seam:

- Add one Tauri-backed transport module that exposes command/projection methods
  and event subscription primitives.
- Keep existing feature helpers as thin functions over that transport at first,
  then collapse duplication after tests prove parity.
- Keep `setupWsHandlers` behavior as an event-handler map, but rename it away
  from WebSocket once it accepts a transport-agnostic event source.

## Test Surface To Replace

Backend route tests currently verify valuable behavior through Axum
`oneshot`, `Request`, `StatusCode`, and response JSON helpers. They should not
be deleted until equivalent service-level and Tauri-adapter coverage exists.

Primary migration groups:

- Command route tests become service command tests plus small adapter tests for
  payload validation and error mapping.
- Projection route tests become service projection tests plus adapter serde
  round-trip tests.
- WebSocket tests become event adapter tests for subscribe, unsubscribe,
  teardown, stale-response handling, bounded buffering, and generation progress.
- Frontend API helper tests should assert Tauri command names and payloads
  rather than `/api` URLs.

## Anti-Patterns And Risks

- Axum-specific `ApiError` and HTTP status assertions are mixed into backend
  behavior tests. Extract neutral backend errors before introducing Tauri.
- Detached `tokio::spawn` calls exist for autosave, Y.Doc, AI generation, batch
  generation, and reference embedding. Move them behind one lifecycle owner.
- App events and Y.Doc updates share one WebSocket. Split app projection events
  from CRDT sync before adopting Tauri events.
- `ui/src/lib/api.ts`, `commandApi.ts`, and `projectionApi.ts` duplicate
  transport and error handling. A single transport adapter will make the
  frontend easier to reason about.
- The launcher and Vite config assume a browser/server topology. They should
  move only after the Tauri vertical slice is working.

## Recommended Slice Order

1. Add backend-neutral error and service contract types without deleting routes.
2. Move one command/projection pair behind backend services and compare route
   output against service output.
3. Add the Tauri scaffold and wire health, one projection, one command, and one
   event through Tauri.
4. Replace frontend transport internals with Tauri-backed adapters while keeping
   existing feature helper names.
5. Migrate route tests to service and Tauri-adapter tests.
6. Delete Axum, WebSocket, static serving, CORS, browser launch, and `/api`
   proxy assumptions after equivalent coverage exists.
7. Remove wasm renderer bridges after the native desktop host boundary exists.
