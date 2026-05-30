# src-tauri/src/bin

## Purpose

This directory contains desktop helper binaries that exercise runtime behavior
outside the main Tauri application entry point.

## Contents

| File | Description |
| ---- | ----------- |
| `native_renderer_smoke.rs` | Native renderer smoke entry point for release and platform checks. |

## Problem

Release and renderer validation need small binary entry points that do not
start the full interactive desktop application.

## Constraints

- Helper binaries must not own product behavior.
- Smoke binaries must be safe to run in automation.
- Main application startup remains in `src-tauri/src/main.rs`.

## Decision

Keep helper binaries under `src/bin` and use them only for bounded smoke or
diagnostic workflows.

## Alternatives Rejected

- Adding smoke-only flags to every runtime path: rejected because helper
  binaries keep automation concerns isolated.
- Moving helper binaries into backend crates: rejected because they depend on
  desktop renderer behavior.

## Invariants

- Helper binaries must have explicit scope and exit behavior.
- Long-running desktop state should remain owned by the main app.

## Revisit Triggers

- Release smoke expands into multiple platform-specific binaries.
- CI adopts a virtual display strategy for full GUI smoke.

## Dependencies

**Internal:** Desktop renderer and Tauri-side host crates.
**External:** Same runtime dependencies as `eidetic-desktop`.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```bash
cargo run -p eidetic-desktop --bin eidetic-native-renderer-smoke
```

## API Consumer Contract

- None for app consumers.
- Reason: binaries are automation entry points, not library APIs.
- Revisit trigger: another process invokes them as stable CLI contracts.

## Structured Producer Contract

- Smoke binaries may print machine-readable status in release workflows.
- Output shape changes require release workflow updates.
