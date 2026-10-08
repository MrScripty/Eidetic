# ui/src/lib

## Purpose

This directory holds the shared frontend surface for the Eidetic UI: typed API contracts, client-side stores, reusable components, and cross-cutting helpers that let the Svelte app render the current project state without duplicating server schema knowledge.

## Contents

| File/Folder                | Description                                                                                      |
| -------------------------- | ------------------------------------------------------------------------------------------------ |
| `types.ts`                 | Shared TypeScript mirrors of core timeline, story, and UI layout contracts.                      |
| `bibleGraphSchemaTypes.ts` | Focused TypeScript mirrors for backend-owned bible graph schema projection shapes.               |
| `api.ts`                   | Tauri command helpers for project, references, AI, models, export, and persistence operations.   |
| `desktopTransport.ts`      | Tauri IPC detection and command invocation helpers for desktop-hosted frontend code.             |
| `commandApi.ts`            | Browser-side command helper barrel for backend-owned commands and versioned command projections. |
| `timelineCommandApi.ts`    | Timeline-specific command helpers that send backend-owned commands through Tauri IPC.            |
| `commandTransport.ts`      | Shared command ID generation for backend-owned command envelopes.                                |
| `serverEventClient.ts`     | Backend event client for Tauri desktop event transport.                                         |
| `commandApi.test.ts`       | Tests for command helper request shape and backend error handling.                               |
| `projectionApi.ts`         | Tauri command helpers for focused backend projections.                                           |
| `projectionApi.test.ts`    | Tests for projection helper command shape and transport error handling.                          |
| `stores/`                  | Reactive Svelte state used to coordinate the UI around backend-driven data.                      |
| `components/`              | Feature UI modules for layout, timeline editing, sidebars, and relationship views.               |
| `scriptFactTypes.ts` | Saved manual-edit and consumed-fact projection/request/review binding shapes. |
| `propagationProposalTypes.ts` | Existing review/acceptance shapes, including separate original/current selected timeline Notes receipts. |

## Problem

The UI needs one place where backend-backed shapes, local UI constants, and shared rendering behavior stay consistent. Without a common `lib/` boundary, the app would drift into per-component contract copies and fragile ad hoc wiring.

## Constraints
- AI preview/content/child-plan helpers accept explicit optional fictional time.
  Omitted time preserves the existing IPC shape; supplied values must be
  non-negative safe integer milliseconds. No helper infers time from UI state.

- Backend responses remain the source of truth for project content.
- Timeline rendering depends on stable shared geometry constants across multiple components.
- The UI mixes persistent project data with transient interaction state, so the boundary must distinguish the two clearly.

## Decision

Keep shared UI contracts, stores, and feature components under `ui/src/lib` and route layout-sensitive constants through `types.ts` so rendering code uses one source of truth for timeline sizing and related chrome.

### Complection Review

Large frontend test files such as `commandApi.test.ts` and
`stores/timelineRenderProjection.svelte.test.ts` stay grouped by boundary
contract rather than operation count. `commandApi.test.ts` verifies browser-side
command envelopes, transport errors, and projection returns through one command
helper surface. `timelineRenderProjection.svelte.test.ts` verifies the store's
refresh/cache/error behavior around one backend projection. Splitting either by
raw size would separate scenarios that share the same contract fixtures.

## Alternatives Rejected

- Putting timeline geometry constants directly inside individual Svelte components: rejected because it would make synchronized layout changes error-prone.
- Splitting each store next to every consumer component: rejected because the timeline/editor shell shares state across multiple panels.

## Invariants

- `childPlanningTypes.ts` includes optional original Bible context and exact
  untimed field revision inputs on child plans. Missing legacy evidence stays
  unknown. Review and recovery display the recorded facts and relationships
  without fetching current Bible values or writing screenplay/timeline state.

- Pending child-plan recovery reads the registered `projection_child_plans`
  endpoint through `getChildPlanListProjection`. Backend records retain their
  original canonical children, statuses and screenplay receipts; callers filter
  Pending records for the selected parent and require an explicit review choice.
  A projection read never generates, accepts or patches saved material.

- The script wire model admits a derived `context_changed` impact reason for
  scene order/window membership. Existing editor review labels explain entering
  and leaving material; saved text still changes only through explicit guarded
  save or proposal acceptance.
- Desktop invocation preserves the original native error as `Error.cause` while
  keeping its user-facing message. Creation retry can recognize its specific
  pre-recording placement refusal without guessing certainty from message text
  or treating other command/transport failures as definitely uncommitted.
- Manual creation wire types carry captured source placement, kind and exact
  text. The command API forwards a stable draft command ID through native invoke;
  persistent block/span IDs and ordering come from the backend response.
- Targeted screenplay previews transport the proven dependency, generation and
  expected output-block revision. They return propagation review projections;
  they do not send an acceptance command or alter authored screenplay text.
- Script impact wire types mirror revision-bound changed/deleted source causes;
  optional impact preserves compatibility with unbound older projections.

- Backend-owned project data enters the UI through typed Tauri IPC contracts instead of free-form objects.
- Shared timeline geometry values are defined once and reused by all dependent components.
- Stores own transient UI coordination; components render from store state rather than manual DOM mutation.

- ScriptContextBlock is shared by context-stack evidence and propagation bindings;
  block/segment identity, exact text and separate write revisions mirror core.
  Context-stack evidence is optional so legacy absence is never backfilled.

## Revisit Triggers

- A second frontend client needs a slimmer shared contract package.
- Timeline layout constants become large enough to justify a dedicated layout module.
- The app introduces SSR or multiple entrypoints that need different store composition roots.

## Dependencies

**Internal:** `ui/src/routes`, `ui/src/app.html`, Rust backend APIs exposed through `api.ts`, `commandApi.ts`, `projectionApi.ts`, and Tauri commands in `desktopTransport.ts`.
**External:** Svelte 5, SvelteKit, Vite.

## Related ADRs

- None identified as of 2026-03-08.
- Reason: the current UI structure changes are local to the frontend module boundary.
- Revisit trigger: a future change alters frontend/backend contract ownership or splits the timeline UI into separate packages.

## Usage Examples

```ts
import { mainTimelinePanelHeightPx } from '$lib/types.js';
import { timelineState } from '$lib/stores/timeline.svelte.js';
import { refreshTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';

const fixedTimelineHeight = mainTimelinePanelHeightPx();

async function openTimeline() {
  timelineState.scrollX = 0;
  await refreshTimelineRenderProjection();
}
```

## API Consumer Contract
Targeted-preview types mirror optional `recall_selection` with exact baseline
selectors and displayed name/path receipts. `ScriptRecallFacts` stages author
intent for one existing Needs review block; it never changes generation defaults,
world data, context links or existing proposals. Canonical selected values appear
only in the backend proposal's existing `bible_inputs` receipt. The server
revalidates source revisions at capture, recording and atomic acceptance.


- Internal consumers import typed shapes and helpers from `$lib/*`.
- Store consumers should treat backend-backed entities as read-through state and mutate them through API/store actions, not local object surgery.
- Command helpers return backend projections and must not patch persistent stores optimistically.
- The label-only Bible command returns `BibleGraphEdgeLabelCommandResponse`:
  fresh writes include source node detail; committed replay returns a null
  projection even when the relationship/source no longer exists. Consumers must
  retain current cache state rather than fabricate a projection on replay.
- Desktop command helpers return backend projections through Tauri IPC; legacy HTTP fallback paths are not production frontend contracts.
- Projection helpers are read-only, use Tauri IPC directly, and return backend-owned versioned read models.
- Layout consumers should reuse exported constants/helpers instead of re-declaring pixel budgets in component-local CSS.
- Compatibility is maintained by updating this directory README or an ADR whenever shared contracts materially change.

## Structured Producer Contract

- `types.ts` exports stable field names that mirror backend timeline/story payloads used throughout the UI.
- UI layout helper exports define default semantics for fixed panel sizing; consumers should treat them as the canonical budget for the main timeline shell.
- When a shared shape or layout helper changes, dependent components must be updated in the same change to preserve visual and type consistency.

Consumed Bible relationships use the existing screenplay input, dependency and
proposal DTOs. Original endpoints/kind/direction/label and owned revision remain
in generation/review receipts; proposal previews require explicit acceptance
and stale results refuse without replacing manual screenplay. Label-only edge
commands carry the original expected revision through desktop transport. Node
detail's edge revision map belongs to the same backend read snapshot; the writer
owns revision/live-identity authority, while the UI owns retained draft and
conflicting-operation controls. No parallel memory store or embedding dependency.

Screenplay proposal custody mirrors optional `bible_node_name_inputs` and owned
name-absence revisions from the canonical server. The existing review surface
labels name causes and discloses recorded names/revisions; proposal acceptance
continues through the guarded server command, without client replacement text.

Timeline range commands optionally carry a canonical TimelineNodeRangeRead captured by the selected-node editor. Exact start/end milliseconds and the optional owned node event preserve placement intent across UI edits; omitted receipts keep existing legacy command signatures and renderer callers. Screen placement remains distinct from fictional time, and range acknowledgements use existing projection/session guards.

`bibleRecallTypes.ts` mirrors bounded related-fact requests, typed relationship
paths, baseline/snapshot source clocks and unresolved/omitted evidence.
`projectionApi.ts` exposes `getBibleRecallProjection` through the existing typed
desktop transport; recall does not extend generation context automatically.

## Typed arc screenplay review

`storyArcTypes.StoryArcFieldInput` mirrors exact consumed prompt values, with a
nullable revision for unbound template history. Existing `ScriptImpactCause`
and targeted proposal types carry per-field arc causes and original/current
evidence. The editor displays exact changes and optional full revision details;
`story_changed` rereads canonical impact instead of patching saved screenplay.

Decision: keep Preview update and Accept update as the existing explicit review
boundary, preserving unrelated author drafts and backend source/version custody.


`removeScriptBlock` carries the captured block revision and stable command ID through native transport. It shares the screenplay projection response with create/edit commands.
