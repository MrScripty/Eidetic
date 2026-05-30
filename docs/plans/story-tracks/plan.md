# Story Tracks

This note captures the current understanding of Eidetic's story-building model while we are planning how the system works overall.

## Current Track Hierarchy

The application uses an NLE-like timeline metaphor. Each story level has a track row, and the "clips" on those rows are `StoryNode` records.

```text
Premise
  Act
    Sequence
      Scene
        Beat
```

In product terms:

- `Premise` is the overall episode/story container.
- `Act` divides the premise into major structural sections such as Cold Open, Act One, Act Two, Act Three, and Tag.
- `Sequence` is the intended level between acts and scenes.
- `Scene` represents concrete scene units inside a sequence.
- `Beat` represents the smallest current story-planning unit inside a scene.

There is currently no `Shot` story level in the model.

## Track vs Node Model

Tracks do not own clips directly. The timeline stores all clips in one flat `Timeline.nodes` list.

Each node has:

- `level`, which determines which track row displays it.
- `parent_id`, which determines hierarchy.
- `time_range`, which determines where it sits on the timeline.
- `name`, notes/content, optional beat type, and lock state.

So the visual track layout and the story hierarchy are related but separate:

- Track placement comes from `StoryNode.level`.
- Story containment comes from `StoryNode.parent_id`.
- Timing comes from `StoryNode.time_range`.

## Story Construction Flow

New projects create a 22-minute episode timeline with one `Premise` node spanning the full duration.

Templates then seed major structure:

- Cold Open
- Act One
- Act Two
- Act Three
- Tag

The app also creates story arcs such as A-Plot, B-Plot, and C-Runner. These are tags attached to nodes, not separate timeline tracks.

## AI Decomposition Flow

The AI planning flow decomposes a selected parent into children:

```text
Premise -> Acts
Act -> Sequences
Sequence -> Scenes
Scene -> Beats
```

The generated child plan includes names, outlines, relative duration weights, and, for scene/beat-level children, characters, location, and props.

When the plan is applied, the server:

- clears existing children for that parent,
- divides the parent's time range across the proposed children by weight,
- creates child nodes at the next story level,
- copies arc tags from parent to child,
- links or creates story-bible entities from the proposal.

## Current Caveat

The clean canonical hierarchy is `Premise -> Act -> Sequence -> Scene -> Beat`.

However, current project templates appear to seed `Scene` nodes directly under `Act` nodes, bypassing the normal hierarchy validator. That looks like legacy or transitional behavior and should be accounted for before making structural changes.

## Standards Compliance Gates

Implementation must follow the standards plan in `docs/refactors/eidetic-projection-architecture/final-plan.md`.

Specific requirements for tracks and hierarchy:

- Timeline structure is backend-owned. Svelte and Bevy display timeline projections and submit commands; they do not own canonical track/node state.
- Track edits must enter through validated commands with checked time ranges, parent IDs, level transitions, and duration bounds.
- `Premise -> Act -> Sequence -> Scene -> Beat` should be enforced by validated domain logic, not only UI conventions.
- Legacy template behavior that skips `Sequence` should be removed when the canonical hierarchy replacement lands.
- Timeline nodes store context only. Final screenplay text belongs to `ScriptDocument`.
- Bevy receives a versioned render projection and emits commands. It must not mutate persistent timeline state directly.
- Tests must cover hierarchy validation, invalid transition rejection, time-range splitting, projection rebuild, and command idempotency.
- Any touched `src/` timeline directories must keep README contracts current.

## Possible Future Extension

If shots become part of the model, the likely hierarchy would become:

```text
Premise
  Act
    Sequence
      Scene
        Beat
          Shot
```

Adding `Shot` would require updates across:

- core `StoryLevel` definitions,
- hierarchy validation,
- default tracks,
- frontend type helpers,
- timeline rendering,
- persistence and migrations,
- AI decomposition prompts,
- export/script behavior.
