# Affect Model Semantics

This document records how Eidetic interprets affect state after Milestone 11.
The backend owns affect state; Svelte and Bevy only render projections or send
commands.

## Canonical State

- Canonical affect values are stored in SQLite as current-state rows with
  command/event/revision history.
- A value has a target scope, valence, arousal, emotional intensity,
  confidence, mood labels, provenance, and optional rationale.
- Timeline story levels are addressed through timeline-node targets. The node
  type determines whether the affect applies to a premise, act, sequence,
  scene, beat, or shot.
- Bible and script targets are addressed through bible-node, bible-snapshot, and
  script-segment targets.
- Proposed affect changes are separate from canonical values until accepted.

## Generation

- Script generation receives affect context from backend hydration, not from UI
  state.
- Prompt construction treats affect as a local constraint on tone, energy, and
  emotional direction.
- Valence and arousal do not override locked script text or accepted bible facts.
  They guide wording, pacing, and subtext inside the scope being generated.
- Mood labels summarize the intended emotional surface; numeric values provide
  stable ordering and intensity for model prompts.
- Confidence communicates how strongly the generator should follow the affect
  value. Low-confidence affect should be treated as soft guidance.

## Timeline Overlays

- Timeline render projections include `affect_overlays` derived from canonical
  backend affect rows.
- Overlay samples are joined to current clip ranges for timeline-node affect
  targets.
- Renderers decide how to draw overlays, but they do not infer or own affect
  state.

## Propagation And Review

- Manual script edits or agent analysis can create affect proposals instead of
  directly mutating canonical affect state.
- Rejecting a proposal records only the proposal status change.
- Accepting a proposal records the proposal status change and writes the
  proposed affect value to canonical affect state in the same backend change.
- Review surfaces consume proposal projections and send accept/reject commands.

## Dependencies

- Affect dependencies link affect traits to timeline nodes, script segments,
  bible nodes or fields, and generation prompts.
- These records are queryable rows, not opaque JSON blobs.
- Dependencies explain why a specific affect trait influenced a prompt, script
  segment, or bible detail.

## Undo, Redo, And History

- Affect values, dependencies, and proposals produce history revisions.
- Before/after review is available through recorded change events and object
  revisions.
- Undo and redo should operate by replaying backend command/event history rather
  than by patching UI state.

## Projection Boundary

- Svelte may hold temporary draft input state, such as a slider value before the
  user submits a command.
- Persisted affect state, proposal status, review outcomes, overlays, and prompt
  hydration are backend-owned.
- Bevy and Svelte render the same backend projections; neither renderer is a
  source of truth.
