# src-tauri/capabilities

## Purpose

This directory contains Tauri capability manifests that define what the desktop
application is allowed to access.

## Contents

| File | Description |
| ---- | ----------- |
| `default.json` | Default desktop capability manifest. |

## Problem

Desktop permissions must be explicit and reviewable instead of inferred from
runtime behavior.

## Constraints

- Capability changes affect desktop security posture.
- Generated schemas under `src-tauri/gen` describe valid manifest structure.
- Capability files are consumed by Tauri tooling.

## Decision

Keep capability manifests in Tauri's expected directory and review permission
changes as structured configuration changes.

## Alternatives Rejected

- Embedding permissions in Rust code: rejected because Tauri tooling expects
  manifest files.
- Treating generated schemas as source policy: rejected because manifests are
  the reviewed permission source.

## Invariants

- Capability manifests remain valid Tauri JSON.
- Permission changes require security review in the PR rationale.

## Revisit Triggers

- Tauri changes capability schema layout.
- New plugins require broader permissions.

## Dependencies

**Internal:** `src-tauri/gen/schemas`.
**External:** Tauri capability tooling.

## Related ADRs

- `ADR-002` standards compliance baseline.

## Usage Examples

```json
{ "identifier": "default" }
```

## API Consumer Contract

- None for runtime API consumers.
- Reason: capability files are tool-consumed configuration.
- Revisit trigger: another tool consumes this manifest as a stable API.

## Structured Producer Contract

- Files are JSON capability manifests consumed by Tauri.
- Schema-invalid changes must fail before release.
