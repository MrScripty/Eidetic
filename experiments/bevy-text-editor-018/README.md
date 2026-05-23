# Bevy Text Editor Experiment

## Purpose

This standalone Rust project tests whether Eidetic can use a direct native
Bevy 0.18 window for formatted screenplay and novel viewing, scrolling, hit
testing, and basic editing.

## Contents

| File/Folder | Description |
| ----------- | ----------- |
| `Cargo.toml` | Isolated Cargo package and nested workspace boundary for the experiment. |
| `src/` | Bevy application and experiment-local document, layout, rendering, and input code. |
| `../../screenplays/` | Ignored local screenplay fixtures used by the experiment at runtime. |

## Problem

Eidetic needs evidence for whether text-heavy editor surfaces can live inside a
Bevy-owned native window without browser surface overhead or cross-framework
input forwarding.

## Constraints

- The experiment must not be added to the root Eidetic workspace.
- Bevy dependencies must remain inside this experiment or future renderer leaf
  crates.
- Movie screenplay fixtures stay under the ignored root `screenplays/`
  directory and must not be committed.
- The first implementation slices should prove viewing and scrolling before
  broader editing behavior is added.
- No Svelte, DOM, CEF, Electron, Iced, GPUI, or WebView overlay is allowed in
  this experiment window.

## Decision

The experiment is a nested Cargo workspace with both `[package]` and
`[workspace]` in its local `Cargo.toml`. This keeps the spike runnable with
standard Cargo commands while preventing accidental inclusion in the root
workspace and launcher verification paths.

## Alternatives Rejected

- Root workspace member: rejected because canonical Eidetic checks would pull a
  GUI spike into production verification before the architecture decision is
  made.
- Production leaf crate under `crates/`: rejected because this is a decision
  spike, not a committed renderer boundary.
- Web overlay prototype: rejected because the specific question is whether
  Bevy can own text rendering and input directly.

## Invariants

- The root `Cargo.toml` workspace member list does not include this experiment.
- Production crates do not depend on this experiment.
- Runtime fixtures under `../../screenplays/` remain ignored by Git.
- Bevy ECS entities are render projections, not the canonical document model.

## Revisit Triggers

- The experiment produces a recommendation to integrate a Bevy-native text
  surface into Eidetic.
- The experiment needs production data contracts, Tauri commands, IPC, or
  backend persistence.
- Bevy text APIs require a separate shaping/layout dependency that changes the
  dependency-risk profile.

## Dependencies

**Internal:** None. The experiment intentionally does not depend on Eidetic
production crates.

**External:** `bevy = 0.18.1` with explicit native window, input, text, sprite,
render, default font, and Linux platform features. The experiment avoids the
broader `2d` feature collection so audio, UI, and picking are not pulled in
before the spike needs them.

## Related ADRs

- None identified as of 2026-05-23.
- Reason: this project is a spike to gather evidence before an architecture
  decision.
- Revisit trigger: the decision report recommends production integration.

## Usage Examples

Run from this directory:

```bash
cargo run
```

Validate from this directory:

```bash
cargo fmt --check
cargo check
cargo test
```

## API Consumer Contract

- This experiment has no public API consumers.
- Reason: it is a standalone binary spike.
- Revisit trigger: another package imports experiment code or the spike is
  promoted into a reusable crate.

## Structured Producer Contract

- This experiment does not publish machine-consumed production metadata.
- Reason: debug metrics are local runtime observations, not stable artifacts.
- Revisit trigger: benchmark reports, fixture manifests, or renderer contracts
  become consumed by CI or other packages.
