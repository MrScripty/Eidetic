# ui/src/lib/components/layout

## Purpose
This directory contains the top-level shell components that partition the Eidetic UI into the editor workspace, side panels, and the bottom timeline stack.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `AppShell.svelte` | Primary application frame that composes the sidebar, editor/script region, right panel, and bottom timeline stack. |
| `AiStatusIndicator.svelte` | Floating AI connection status indicator positioned by the shell. |
| `AppToolbar.svelte` | Focused top toolbar surface for shell-level save/export commands. |
| `AppWorkspace.svelte` | Focused central workspace switcher for script, graph, and split layouts. |
| `BottomTimelineStack.svelte` | Fixed-height bottom region that keeps the timeline anchored to the window bottom and conditionally stacks the character timeline beneath it. |
| `GraphRendererWindowControls.svelte` | Projection-only launch/focus/close/status controls for the floating Bevy bible graph renderer window. |
| `GraphSelectionDetail.svelte` | Right-panel detail projection for selected graph edges, influence paths, context layers, and neighborhoods. |
| `GraphRightInspector.svelte` | Right inspector owner that selects between node detail and graph selection detail projections. |
| `graphSelectionDetails.ts` | Pure adapter from bounded bible render graph projections plus transient selection into inspectable detail rows. |
| `GraphWorkspacePanel.svelte` | Central workspace graph projection surface for graph-focused and split workspace modes. |
| `GraphWorkspaceSideLists.svelte` | Keyboard-accessible side lists for inspectable context layers, graph influences, edges, and neighborhoods. |
| `graphWorkspaceItems.ts` | Pure adapter from bounded bible render graph projections into graph workspace side-list rows. |
| `PanelResizer.svelte` | Generic drag handle used for resizable in-shell panel boundaries that remain user-adjustable. |
| `Sidebar.svelte` | Left-side navigation and detail entry point for story/bible content. |
| `SplashScreen.svelte` | Project bootstrap UI shown before a project is active. |

## Problem
The app shell has to coordinate multiple panels with different layout rules. The editor region must absorb viewport changes, while the timeline region must stay visually stable and anchored to the bottom for timeline editing to remain predictable.

## Constraints
- The bottom timeline panel must keep a fixed height based on shared timeline geometry.
- The upper workspace must remain scrollable/resizable without pushing the timeline off the window edge.
- Layout code already lives in large Svelte components, so new shell behavior should stay isolated where possible.

## Decision
Keep the main composition in `AppShell.svelte` but isolate the bottom timeline stack in `BottomTimelineStack.svelte` and shell command bar in `AppToolbar.svelte` so the shell owns panel composition, the shell owns the user-selected timeline height, and focused child components own their own markup and styling.

## Alternatives Rejected
- Leaving the timeline directly in `AppShell.svelte` with more inline sizing logic: rejected because `AppShell.svelte` already exceeds the decomposition review threshold.
- Letting the timeline continue to consume remaining flex height: rejected because it makes the bottom editing surface unstable across window sizes.

## Invariants
- The timeline stack is the bottom-most shell region whenever a project is open.
- Fixed-height timeline sizing is derived from shared constants, not component-local literals.
- The user-selected main timeline height is preserved across window resize; viewport changes may clamp the rendered height temporarily but do not overwrite the preferred value.
- The upper workspace gets leftover vertical space after the timeline stack claims its budget.

## Revisit Triggers
- Product wants the main timeline vertically resizable again.
- More bottom-docked panels are introduced and need coordinated stack management.
- Shell layout responsibilities outgrow a single `AppShell` composition root.

## Dependencies
**Internal:** `../editor/*`, `../timeline/*`, `../sidebar/*`, `$lib/types.js`, `$lib/stores/*`.
**External:** Svelte runtime only.

## Related ADRs
- None identified as of 2026-03-08.
- Reason: this is an internal shell decomposition within the existing frontend architecture.
- Revisit trigger: the app introduces a broader shell/layout framework or multiple shell variants.

## Usage Examples
```svelte
<div class="app-shell has-project">
	<div class="upper-section">...</div>
	<BottomTimelineStack />
</div>
```

## API Consumer Contract
- `AppShell.svelte` is the composition root for project-active layout.
- `AppToolbar.svelte` receives save/export/workspace callbacks from the shell
  and does not read or mutate project state directly.
- `AppWorkspace.svelte` owns central workspace mode rendering and script-editor
  height state.
- `GraphWorkspacePanel.svelte` consumes backend-owned bible render graph
  projection caches and emits only transient graph selection.
- `GraphRightInspector.svelte` reads projection caches for the right panel and
  does not write durable graph state directly.
- `GraphWorkspaceSideLists.svelte` provides keyboard-accessible alternatives
  for context layer, graph influence, edge, and neighborhood inspection from
  projection payloads.
- `GraphSelectionDetail.svelte` reads the current bounded render graph
  projection and transient graph selection; it must not request broader graph
  data or store durable graph facts locally.
- `GraphRendererWindowControls.svelte` owns only transient pending/error UI for
  launch, focus, close, and status commands. Renderer lifecycle, projections,
  command queues, and durable graph/timeline state remain backend/desktop-host
  owned.
- `BottomTimelineStack.svelte` exposes no custom props; it renders from shared stores and shared layout helpers.
- `AppShell.svelte` owns the preferred timeline height and passes the currently rendered height into `BottomTimelineStack.svelte`.
- Consumers should not duplicate the bottom stack markup elsewhere; alternative shell layouts should compose this module or replace it explicitly.
- If resizer ownership changes, update this README to record which module owns the vertical split behavior.

## Structured Producer Contract
- None identified as of 2026-03-08.
- Reason: this directory renders UI composition and does not emit persisted machine-consumed artifacts.
- Revisit trigger: a future layout module starts generating saved panel presets or serialized workspace layout metadata.
