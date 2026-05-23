# Plan: Bevy 0.18 Text Editor Experiment

## Objective

Create a standalone Bevy 0.18 experiment that tests whether Eidetic can use a
direct native Bevy window for formatted screenplay and novel viewing, scrolling,
hit testing, and basic editing without Svelte overlays, CEF, Electron, Iced,
GPUI, or browser-surface input forwarding.

The output is a decision-quality prototype and report, not production editor
infrastructure.

## Scope

### In Scope

- Standalone Rust/Bevy experiment under `experiments/bevy-text-editor-018/`.
- Direct Bevy desktop window ownership for rendering, input, focus, and scroll.
- Fountain screenplay loading from ignored local fixtures under
  `screenplays/real-movies/`.
- Generated novel/manuscript fixture for long-paragraph stress testing.
- Semantic document model for screenplay and prose blocks.
- Layout cache that is independent from Bevy ECS entities.
- Virtualized text rendering for visible lines plus overscan.
- Basic editing operations sufficient to test latency, caret placement, and
  document mutation.
- Bevy-rendered debug metrics for frame time, layout time, visible line count,
  total document size, and entity count.
- A final recommendation on whether a custom Bevy editor is viable for Eidetic.

### Out of Scope

- Production integration with the main Eidetic application.
- Svelte, DOM, WebView, CEF, Electron, Tauri WebView overlay, Iced, or GPUI
  editor surfaces.
- Full Fountain specification compliance.
- Full screenplay pagination, revisions, locked pages, production sides, or
  collaboration workflows.
- Rich text authoring beyond simple screenplay/prose block styles.
- Accessibility parity for production use.
- IME, clipboard, undo/redo, spellcheck, comments, and export in the first
  prototype pass.

## Inputs

### Problem

Eidetic needs to decide whether long-form text surfaces should be implemented as
custom Bevy-native editor/viewer surfaces or delegated to another native or web
UI stack. The current concern is that DOM/WebView surfaces introduce rendering
overhead, input handoff complexity, and focus forwarding risk when combined with
Bevy-driven graph and timeline views.

This experiment isolates the text surface question by removing all UI overlay
frameworks and testing Bevy as the sole owner of the window, input events, text
viewport, and interaction loop.

### Constraints

- The experiment must remain separate from the main Eidetic runtime.
- Bevy and heavy rendering dependencies must stay in the experiment boundary and
  must not leak into `eidetic-core` or existing production crates.
- Local movie screenplay files remain ignored and must not be committed.
- The first usable slice must be a thin vertical viewer before broad editing
  features are added.
- Bevy owns only experiment-local transient state; this plan does not change
  Eidetic's production ownership model.
- The experiment must be runnable from its own directory with standard Rust
  commands.
- No web UI overlays are allowed in the experiment window.

### Assumptions

- Bevy 0.18 is the target version for the spike.
- The converted Fountain files under `screenplays/real-movies/` are adequate
  real-world fixtures even though they are heuristic PDF conversions.
- Perfect screenplay semantics are less important than rendering, layout,
  scrolling, and editing behavior for this decision spike.
- Monospace screenplay formatting is acceptable for the initial renderer.
- A generated novel fixture can stand in for copyrighted prose while testing
  long wrapped paragraphs.
- The implementation can use a local font asset, preferably a Courier-style
  font, for screenplay formatting.

### Dependencies

- `bevy = "0.18"` for direct native windowing, input, and rendering.
- `ropey` for editable text storage.
- `unicode-segmentation` for grapheme-aware caret movement.
- `unicode-width` or equivalent width measurement only if Bevy text metrics are
  not sufficient for the first layout pass.
- Local Fountain fixtures under `screenplays/real-movies/`, which are ignored by
  Git.
- A local monospace font asset for predictable screenplay layout.

### Affected Structured Contracts

- Experiment-local document model:
  `Document`, `Block`, `BlockKind`, `TextRange`, `Caret`, and `Selection`.
- Experiment-local layout model:
  `LaidOutLine`, `LineStyle`, `VisibleLineRange`, and hit-test coordinates.
- Experiment-local renderer contract:
  layout lines are the source of truth for Bevy text entities; ECS entities are
  disposable render projections.

No production API, database schema, IPC payload, Tauri command, generated
TypeScript type, or persisted Eidetic contract is affected.

### Affected Persisted Artifacts

- None for production Eidetic data.
- Local ignored screenplay fixtures may be read.
- Optional generated stress fixtures may be created inside the experiment or
  ignored fixture directories.
- The experiment must not write canonical project data.

### Concurrency and Lifecycle Review

- The Bevy app loop is the lifecycle owner for window, input, rendering, and
  per-frame systems.
- No background tasks are required for the first milestone.
- If file watching, async loading, or benchmark workers are added later, each
  must have a single owner, cancellation path, shutdown behavior, and bounded
  communication channel before implementation.
- Document edits and layout invalidation must run in a deterministic order:
  input command, document mutation, layout invalidation, visible range update,
  render projection update.

### Risks

| Risk | Impact | Mitigation |
| ---- | ------ | ---------- |
| Bevy text rendering requires too many entities or updates for smooth scrolling. | High | Virtualize visible lines plus overscan and track entity count as a pass/fail metric. |
| Layout and hit testing drift from rendered glyph positions. | High | Keep layout state as the source of truth and add explicit hit-test checks for line and column mapping. |
| Bevy text APIs are insufficient for editor-grade shaping, fallback, or measurement. | High | Isolate layout and document models so a different text shaper or renderer can be substituted. |
| PDF-to-Fountain conversions contain semantic errors. | Medium | Use them as stress fixtures, not correctness fixtures; keep a small hand-authored fixture for format expectations. |
| Editing scope expands into a full production editor before the rendering question is answered. | Medium | Gate editing milestones behind viewer and layout performance acceptance. |
| Heavy dependencies bleed into production crates. | Medium | Keep the experiment as a standalone leaf project and document dependency boundaries. |
| IME, accessibility, clipboard, and undo/redo needs change the viability decision. | Medium | Record them as explicit second-phase gates after basic viewing/editing behavior is measured. |

## Definition of Done

- The experiment opens a direct Bevy 0.18 desktop window.
- At least one real screenplay fixture renders in screenplay-like formatting.
- The largest current screenplay fixture scrolls smoothly with bounded rendered
  entity count.
- The renderer displays only visible lines plus overscan.
- Basic editing supports caret placement, typing, delete/backspace, and enter
  in a way that exposes text latency and layout invalidation costs.
- A generated novel fixture renders and scrolls with long wrapped paragraphs.
- Debug metrics are visible in the Bevy window.
- The experiment has a short final report with a recommendation for Eidetic.

## Milestones

### Milestone 1: Project Shell

**Goal:** Create an isolated Bevy 0.18 app that runs independently from Eidetic.

**Tasks:**

- [ ] Create `experiments/bevy-text-editor-018/` as a standalone Rust project.
- [ ] Add a local `README.md` explaining purpose, constraints, and run command.
- [ ] Add minimal Bevy dependencies without touching production crates.
- [ ] Open a Bevy window with a blank text surface and Bevy-rendered metrics.
- [ ] Add a fixture path configuration that can find ignored local scripts.

**Verification:**

- `cargo fmt`
- `cargo check`
- `cargo run`
- Confirm no Bevy dependency is added to existing production crates.

**Status:** Not started.

### Milestone 2: Fountain Viewer Slice

**Goal:** Render one real screenplay as formatted read-only text.

**Tasks:**

- [ ] Implement a conservative Fountain loader for title, scene heading, action,
  character, dialogue, parenthetical, transition, and unknown blocks.
- [ ] Convert parsed blocks into experiment-local semantic `Document` data.
- [ ] Add a screenplay stylesheet with page width, margins, indentation, block
  spacing, and monospace font settings.
- [ ] Render the first script as text in the Bevy window.
- [ ] Add keyboard shortcut or config selection for the initial fixture.

**Verification:**

- Unit tests for Fountain block classification using small hand-authored samples.
- Manual visual comparison against the first pages of the extracted source.
- `cargo test`
- `cargo run`

**Status:** Not started.

### Milestone 3: Layout Cache and Virtualized Rendering

**Goal:** Keep document layout independent from Bevy ECS and render only visible
lines plus overscan.

**Tasks:**

- [ ] Define `LaidOutLine`, line style, block reference, byte/grapheme range,
  and y-position data.
- [ ] Implement line wrapping and block spacing for screenplay blocks.
- [ ] Implement visible range calculation from scroll offset and viewport size.
- [ ] Reuse or update a bounded pool of Bevy text entities instead of spawning
  entities for the whole document.
- [ ] Show metrics for total blocks, laid-out lines, visible lines, layout time,
  render update time, and entity count.

**Verification:**

- Unit tests for wrapping and visible range calculation.
- Manual scroll test on every current screenplay fixture.
- Entity count remains bounded as the document size changes.
- `cargo test`
- `cargo run --release` for scroll profiling.

**Status:** Not started.

### Milestone 4: Caret, Hit Testing, and Basic Editing

**Goal:** Add enough editing behavior to evaluate Bevy-native input and document
mutation latency.

**Tasks:**

- [ ] Store editable block text with `ropey`.
- [ ] Map mouse clicks from viewport coordinates to block/line/grapheme caret
  positions.
- [ ] Render caret position from layout state.
- [ ] Handle normal text input, backspace, delete, and enter.
- [ ] Invalidate and recompute only affected layout regions where practical.
- [ ] Keep scroll position stable while editing.

**Verification:**

- Unit tests for edit operations and caret movement across block boundaries.
- Manual typing latency test near the top, middle, and end of a large script.
- Manual click-placement checks on action, dialogue, and parenthetical lines.
- `cargo test`
- `cargo run --release`

**Status:** Not started.

### Milestone 5: Selection and Script Switching

**Goal:** Test editor interactions that usually expose viewport/input model
weaknesses.

**Tasks:**

- [ ] Add drag selection inside the text viewport.
- [ ] Add shift-arrow selection.
- [ ] Render selection highlights behind text using Bevy primitives.
- [ ] Support switching among current screenplay fixtures without restarting.
- [ ] Add generated screenplay and novel stress fixtures.

**Verification:**

- Manual selection across wrapped lines and block boundaries.
- Manual rapid script switching and scroll tests.
- Generated novel fixture reaches long-paragraph stress target.
- `cargo test`
- `cargo run --release`

**Status:** Not started.

### Milestone 6: Decision Report

**Goal:** Produce a concise recommendation for Eidetic's text-surface strategy.

**Tasks:**

- [ ] Record performance observations for screenplay and novel fixtures.
- [ ] Record interaction issues for caret, hit testing, input, and selection.
- [ ] Identify unsupported editor requirements that would need second-phase
  proof: IME, clipboard, undo/redo, accessibility, font fallback, and export.
- [ ] Compare Bevy-native viability against Svelte overlay, Iced, GPUI, and
  custom text renderer alternatives.
- [ ] Recommend one of: Bevy-native editor, Bevy-native viewer only, external
  native editor surface, or web overlay despite integration cost.

**Verification:**

- Report links to exact commit or local experiment state used for the decision.
- Recommendation explicitly states pass/fail evidence and remaining risk.
- No production Eidetic architecture changes are implied without a follow-up
  plan or ADR.

**Status:** Not started.

## Execution Notes

- Plan created before implementation.
- Current known local screenplay fixtures are ignored by Git and should stay
  local.
- Before implementation begins, inspect `git status` and resolve or explicitly
  allow unrelated dirty implementation files according to `PLAN-STANDARDS.md`.
- The existing `.gitignore` change for `/screenplays/` is related setup work.

## Commit Cadence Notes

- Commit when a logical slice is complete and verified.
- Keep the project shell, viewer slice, virtualization slice, editing slice, and
  final report as separate commits when possible.
- Follow commit format/history rules from `COMMIT-STANDARDS.md`.
- Do not include ignored screenplay PDFs, extracted text, or converted movie
  scripts in commits.

## Optional Subagent Assignment

None planned for the initial implementation.

Reason: the first vertical slice is small and benefits from one owner keeping
the document, layout, and renderer contracts aligned.

Revisit trigger: implementation splits into independent benchmark/report,
parser, and renderer workstreams with non-overlapping write sets.

## Re-Plan Triggers

- Bevy 0.18 text APIs cannot provide enough measurement or rendering fidelity
  for the initial viewer slice.
- The experiment needs a text shaping engine such as `cosmic-text`.
- Smooth scrolling cannot be achieved with visible-line virtualization.
- Basic editing requires architectural changes to the document or layout model.
- IME, clipboard, undo/redo, or accessibility becomes a first-phase acceptance
  requirement.
- The experiment needs to integrate with production Eidetic crates or Tauri.
- Dependency or platform constraints conflict with standards requirements.

## Recommendations

- Start with the thinnest read-only viewer slice before implementing editing.
  This answers the rendering and scroll question before the prototype absorbs
  full editor complexity.
- Keep the document and layout modules free of Bevy types where practical. That
  preserves reuse if the renderer changes to Iced, GPUI, or another native text
  stack.
- Treat real converted movie scripts as stress fixtures, not correctness
  fixtures. Add one small original fixture for parser and formatting tests.
- Measure in `--release` before drawing conclusions about scroll or typing
  performance.

## Completion Summary

### Completed

- Not started.

### Deviations

- None.

### Follow-Ups

- Add an ADR only if the experiment produces a production architecture decision.
- Add an implementation plan only if the experiment moves from spike to Eidetic
  integration.

### Verification Summary

- Not run. This document is the planning artifact only.

### Traceability Links

- Module README updated: N/A until the experiment directory is created.
- ADR added/updated: N/A until the experiment produces an architecture
  decision.
- Standards followed:
  `/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/Coding-Standards/PLAN-STANDARDS.md`
  and
  `/media/jeremy/OrangeCream/Linux Software/repos/owned/developer-tooling/Coding-Standards/DOCUMENTATION-STANDARDS.md`.

