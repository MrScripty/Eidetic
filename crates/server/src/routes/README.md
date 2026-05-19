# crates/server/src/routes

## Purpose
This directory defines the host-facing HTTP contract for Eidetic’s browser client and other local callers.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `project.rs` | Project lifecycle, save/load, and listing routes. |
| `commands.rs` | Projection-owned command endpoints that write through backend history storage, including object-field, bible-graph node, bible-graph field, bible-graph edge, and canonical-root commands. |
| `commands_tests.rs` | Route-level command tests for loaded-project, projection, idempotency, and validation behavior. |
| `projections.rs` | Projection read endpoints backed by persisted history and typed graph state, including bible graph node detail/list and render graph projections. |
| `projections_tests.rs` | Route-level projection read tests for loaded-project, missing, initial, and persisted projections. |
| `timeline.rs` | Timeline node, track, and hierarchy routes. |
| `story.rs` | Arc, bible-entity, and relation routes. |
| `script.rs` | Script editing and content routes. |
| `ai.rs` | AI generation, context, and extraction routes. |
| `reference.rs` | Reference document CRUD routes. |
| `support.rs` | Shared route helpers for active project path lookup and history error mapping. |

## Problem
The frontend needs a stable local API that exposes all editor actions with correct status semantics and shared validation.

## Constraints
- Route behavior must stay compatible with `ui/src/lib/api.ts`.
- Validation and error semantics need to be explicit and testable.
- `ai.rs` is already large enough to be covered by the decomposition baseline in `ADR-001`.

## Decision
Group routes by feature area, keep shared validation/error logic outside the handlers, and document the contract here as the canonical API consumer surface.

## Alternatives Rejected
- One monolithic route file: rejected because project, timeline, story, and AI concerns evolve independently.
- Pushing validation into the client: rejected because the server remains the authoritative boundary.

## Invariants
- Non-success outcomes use explicit HTTP status codes rather than `200` error bodies.
- Route payload field names stay aligned with frontend command/projection helpers.
- Load/save flows continue to validate paths and preserve project compatibility.

## Revisit Triggers
- A second external consumer needs versioned route behavior.
- `ai.rs` or another route file adds a second unrelated responsibility.

## Dependencies
**Internal:** `crates/server/src/error.rs`, `crates/server/src/validation.rs`, `crates/server/src/persistence.rs`, `eidetic-core`.
**External:** `axum`, `serde`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized route modules.

## Usage Examples
```rust
let app = crate::routes::project::router();
```

## API Consumer Contract
- Consumers send JSON requests to feature-specific endpoints rooted under `/api`.
- Error behavior is status-driven: non-2xx responses carry an `error` message and should not be treated as success.
- Realtime-sensitive callers should combine these routes with the websocket event stream instead of polling every mutation path.

## Structured Producer Contract
- Route responses produce stable JSON field names mirrored by `ui/src/lib/types.ts` and `ui/src/lib/api.ts`.
- Projection route additions must return backend-owned read models and must not imply that frontend stores own durable state.
