# crates/server/src/diffusion

## Purpose
This directory owns diffusion-model lifecycle management and the bridge types used to talk to the Python-backed diffusion runtime.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `manager.rs` | Process/lifecycle coordination for the diffusion runtime. |
| `bridge.rs` | Boundary helpers for the Python bridge. |
| `types.rs` | Diffusion commands, updates, and error types. |
| `tests.rs` | Unit coverage for bridge-facing diffusion types. |

## Problem
Image or diffusion-style workflows need a long-lived managed runtime that is separate from the request/response lifetime of normal HTTP handlers.

## Constraints
- The runtime depends on Python-side libraries and bridge behavior.
- Commands and updates must stay serializable and predictable for route consumers.

## Decision
Isolate diffusion-specific lifecycle management under its own directory so the main route layer stays focused on API semantics instead of Python bridge details.

## Alternatives Rejected
- Folding diffusion behavior into generic AI backend modules: rejected because lifecycle and dependency behavior are materially different.

## Invariants
- Diffusion command/update types remain the canonical contract between routes and the manager.
- Python bridge details stay behind this boundary.

## Revisit Triggers
- Diffusion features expand enough to require multiple runtime managers or model families.

## Dependencies
**Internal:** `crates/server/src/routes`, `crates/server/src/error.rs`.
**External:** `pyo3`, `tokio`.

## Related ADRs
- `ADR-001` decomposition baseline for oversized modules.

## Usage Examples
```rust
use crate::diffusion::types::DiffusionStatus;
```

## API Consumer Contract
- None identified as of 2026-03-08.
- Reason: this is an internal runtime-management boundary.
- Revisit trigger: diffusion control becomes a pluggable backend surface.

## Structured Producer Contract
- Diffusion command/update/status types are consumed by routes and the frontend.
- Enum or field changes require synchronized route, test, and client updates.
