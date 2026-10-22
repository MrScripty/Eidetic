# crates/server/src/ai_backends

## Purpose
This directory implements server-side adapters that translate core AI requests into specific backend calls.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | Shared backend trait and adapter selection. |
| `llamacpp.rs` | Existing direct local llama.cpp OpenAI-compatible adapter. |
| `pumas.rs` | Managed Pumas typed chat operation v1 with capability checks, fenced transport and JSON-output validation. |
| `openrouter.rs` | OpenRouter HTTP adapter. |
| `sse.rs` | Shared stateful SSE framing, completion validation and transport error propagation. |
| `sse_tests.rs` | Buffer boundaries, fragmented framing, malformed input and bounded provider diagnostics. |
| `transport_tests.rs` | Actual loopback HTTP fixtures for both adapters, including split events and incomplete bodies. |

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
- Streaming tokens retain event order and UTF-8 bytes across network chunks.
  JSON SSE data events must parse successfully; provider error events and body
  read failures become stream errors. A successful stream requires `[DONE]`
  and clean HTTP EOF, including a complete declared body after the marker.
- Full-response collection returns the stream error instead of a partial draft.
  OpenRouter's production endpoint stays fixed; a private endpoint seam supports
  real loopback transport tests without an external model call.
- Lines retain at most 256 KiB and events at most 1 MiB, including multi-line
  separators. Limits are checked before appending. Each incoming chunk byte is
  scanned once, and tokens are yielded without accumulating an event queue.
  Provider error details are formatted through a 1 KiB UTF-8-safe bound before
  they can become generation error events. Successful completion still requires
  the terminator and a complete HTTP body; size failures persist no preview.

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

Pumas typed SSE uses `started`, `delta`, `completed` and `failed` JSON events,
checks contract version and request identity, and requires a valid terminal event
plus clean HTTP EOF. The existing OpenAI-compatible providers continue to require
`[DONE]`. Typed Pumas v1 has no JSON-mode option: structured calls explicitly
request JSON in their prompt and validate JSON before the established parsers.
This follows the producer contract at `ad31e391dcd94c204b3fcd748cc98d27b3281e65`;
no direct ONNX session or fake backend is selected in production.
