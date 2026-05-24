# platform_strategy

## Purpose
This module owns native graph renderer platform detection and per-platform
runner strategy selection for the desktop composition root.

## Invariants
- Platform checks stay in this module tree. Renderer host, supervisor, command,
  UI, and backend projection code consume platform-neutral strategy APIs.
- Strategy state is capability reporting only until a platform runner is
  verified under the Tauri runtime.
- Unsupported or unproven platforms return typed backend status instead of
  falling back to legacy renderer paths.
