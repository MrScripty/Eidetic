# src-tauri/gen

## Purpose

This directory contains generated Tauri support artifacts.

## Contents

| Folder | Description |
| ------ | ----------- |
| `schemas/` | Generated JSON schemas for Tauri configuration and capability files. |

## Problem

Tauri tooling produces generated artifacts that should be kept separate from
source-owned desktop behavior.

## Constraints

- Generated files may change when Tauri tooling changes.
- Source code should not hand-edit generated schema payloads.

## Decision

Keep generated artifacts under `src-tauri/gen` and document that source policy
lives in Rust modules and capability manifests.

## Alternatives Rejected

- Moving generated files into `docs/`: rejected because Tauri expects this
  generated layout.
- Treating generated schemas as manually maintained source: rejected because
  regeneration would overwrite edits.

## Invariants

- Generated artifacts are not the owner of desktop behavior.
- Manual changes require a documented tooling reason.

## Revisit Triggers

- Tauri changes generated artifact locations.
- CI adds generated-file drift checks.

## Dependencies

**Internal:** `src-tauri/capabilities`.
**External:** Tauri tooling.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```bash
ls src-tauri/gen/schemas
```

## API Consumer Contract

- None for runtime API consumers.
- Reason: generated files support tooling.
- Revisit trigger: another process consumes generated schemas as a stable API.

## Structured Producer Contract

- Generated files are JSON schema artifacts produced by Tauri tooling.
- Schema shape changes should be reviewed as dependency/tooling changes.
