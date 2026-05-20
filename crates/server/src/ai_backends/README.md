# crates/server/src/ai_backends

## Purpose
This directory implements server-side adapters that translate core AI requests into specific backend calls.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | Shared backend trait and adapter selection. |
| `llamacpp.rs` | Local Pumas llama.cpp OpenAI-compatible adapter. |
| `openrouter.rs` | OpenRouter HTTP adapter. |

## Problem
The server needs one abstraction for multiple text-generation providers without leaking provider-specific details into route handlers.

## Constraints
- Backend adapters must preserve the request/response semantics defined by `eidetic-core`.
- Provider-specific transport failures need to map back into consistent route error behavior.

## Decision
Keep provider adapters behind a shared module boundary so routes can depend on one backend-facing surface.

## Alternatives Rejected
- Calling providers directly from routes: rejected because it would duplicate mapping and error handling.

## Invariants
- Backend adapters consume core request types rather than ad hoc JSON.
- Provider-specific configuration stays behind this boundary.

## Revisit Triggers
- Another provider introduces streaming or capability semantics that no longer fit the current adapter shape.

## Dependencies
**Internal:** `crates/core/src/ai`, `crates/server/src/routes`, `crates/server/src/error.rs`.
**External:** `reqwest`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```rust
use crate::ai_backends::AiBackend;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this directory is an internal adapter layer consumed by the server only.
- Revisit trigger: adapters become part of a plugin or external provider SDK surface.

## Structured Producer Contract
- Backend adapters must preserve the field semantics defined by core AI request/response types.
- Changes to response interpretation require coordinated route and frontend progress-handling updates.
