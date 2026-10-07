# ui/src/lib/stores

## Purpose

This directory contains the shared reactive frontend state used to coordinate the timeline shell, editor, sidebar panels, notifications, and backend event-driven updates.

## Contents

| File/Folder                               | Description                                                                          |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| `editor.svelte.ts`                        | Editor-local UI state and generation-progress helpers.                               |
| `timeline.svelte.ts`                      | Timeline viewport, playhead, tool, and drag interaction state.                       |
| `timelineRenderProjection.svelte.ts`      | Timeline cache and command bridge with cache-generation ownership and version ordering. |
| `project.svelte.ts`                       | Active project session metadata.                                                     |
| `bibleGraphNodeProjection.svelte.ts`      | Focused cache/action layer for backend-owned bible graph node and field projections. |
| `bible.svelte.ts`                         | Typed transient graph selection state.                                               |
| `bibleGraphNodeDetailProjection.svelte.ts` | Existing Bible detail cache and owned read lifecycle; no canonical fact state. |
| `bibleGraphNodeReadOwners.ts` | Nonreactive request/inspector tokens; reads cannot retrigger mount effects. |
| `bibleRecallProjection.svelte.ts` | Disposable inspection cache with request, query, selection, session and revision guards. |
| `bibleRenderGraphProjection.svelte.ts`    | Focused cache layer for backend-owned bible render graph projections.                |
| `contextStackProjection.svelte.ts`        | Focused cache layer for backend-owned selected timeline context stack projections.   |
| `graphRendererCommands.ts`                | Applies validated transient Bevy graph renderer commands to UI selection state.      |
| `graphRendererWindow.svelte.ts`           | Backend-projected renderer-window status for display and lifecycle controls.         |
| `workspaceMode.svelte.ts`                 | Transient workspace layout mode for script, graph, and split views.                  |
| `bibleGraphSchemaProjection.svelte.ts`    | Focused cache layer for backend-owned bible graph schema projections.                |
| `objectFieldProjection.svelte.ts`         | Focused cache/action layer for backend-owned object-field projections.               |
| `scriptDocumentProjection.svelte.ts`      | Focused cache/action layer for backend-owned script document projections.            |
| `scriptBlockCreationSession.svelte.ts`    | Project-session owner of the transient creation draft and exact pending retry.       |
| `scriptBlockEditSession.svelte.ts`        | Session-scoped per-block edit drafts with captured revision and exact retry.          |
| `semanticProposalProjection.svelte.ts`    | Focused cache/action layer for semantic bible reference proposals.                   |
| `propagationProposalProjection.svelte.ts` | Focused cache/action layer for semantic propagation proposals.                       |
| `changeReviewProjection.svelte.ts`        | Focused cache layer for backend-owned change history review projections.             |
| `storyArcProjection.svelte.ts`            | Focused cache/action layer for backend-owned story arc projections.                  |
| `aiStatus.svelte.ts`                      | Shared AI-status polling ownership.                                                  |
| `shortcuts.svelte.ts`                     | Keyboard shortcut registry and dispatch helpers.                                     |
| `notifications.svelte.ts`                 | Toast notification queue state.                                                      |
| `serverEventHandlers.ts`                  | Backend event handlers that fan Tauri server events into stores.                     |

## Problem

`timelineProjectionLifecycle.test.ts` covers deferred refresh/command responses
across session clears and concurrent requests.
`scriptDocumentProjectionLifecycle.test.ts` covers the same boundary for script
reads, block edits and locks: a cleared document's requests must not publish into
its next cache lifetime, and one completed request must not hide another pending
request.

Multiple UI surfaces need shared state and event coordination without turning the route tree into a prop-drilling graph.

## Constraints

- Store state must remain aligned with backend payload semantics.
- Polling and backend event ownership must stay single-owner to avoid duplicate work.
- Tests now cover API error handling and shared AI-status polling behavior from this boundary.

## Decision

Use focused Svelte state modules per feature area and keep backend event and polling orchestration here instead of burying it in component lifecycles.

Projection stores are the only allowed frontend caches for backend-owned
durable state. Transient stores may coordinate selection, hover, focus,
scrolling, zoom, local drafts, pending/error flags, and gesture state. Legacy
ownership paths listed below must be removed before the Bevy timeline renderer
continues.

## Store Ownership Audit

| Store                                     | Classification                      | Current status                                                                                                                            | Milestone 6 action                                                                                            |
| ----------------------------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `affectProposalProjection.svelte.ts`      | Projection cache and command bridge | Caches backend affect proposal projections and replaces cache from command responses with stale-response guards.                          | Keep as proposal projection state; durable affect changes must remain backend-command driven.                 |
| `aiStatus.svelte.ts`                      | Projection/status cache             | Owns the last-known AI backend status and the single polling lifecycle.                                                                   | Keep; add stale-response guards if config-driven overlapping refreshes become possible.                       |
| `bible.svelte.ts`                         | Transient UI state                  | Stores typed graph selection for nodes, edges, influences, context layers, and neighborhoods.                                              | Keep; do not use graph selection as durable graph state.                                                       |
| `bibleGraphNodeProjection.svelte.ts`      | Projection cache and command bridge | Caches backend projection envelopes and replaces cache from command responses with stale-response guards.                                 | Keep; later share common projection command helpers if useful.                                                |
| `bibleGraphSchemaProjection.svelte.ts`    | Projection cache                    | Caches backend schema projection with stale-response guards.                                                                              | Keep.                                                                                                         |
| `bibleRecallProjection.svelte.ts` | Disposable inspection projection cache | Holds only explicitly recalled backend evidence; request, query, selection, session and revision guards revoke stale reads. | Keep read-only; Bible writes/events invalidate evidence while existing drafts and proposals retain their owners. |
| `bibleRenderGraphProjection.svelte.ts`    | Projection cache                    | Caches backend render graph projection with stale-response guards.                                                                        | Keep.                                                                                                         |
| `changeReviewProjection.svelte.ts`        | Projection cache                    | Caches backend change review projection with stale-response guards.                                                                       | Keep.                                                                                                         |
| `contextStackProjection.svelte.ts`        | Projection cache                    | Caches the backend context stack for the selected timeline node with stale-response guards.                                               | Keep; graph context layer UI should read this cache rather than deriving hierarchy in Svelte.                 |
| `editor.svelte.ts`                        | Transient UI state                  | Stores selected timeline node IDs, selected level, and generation-progress state only.                                                    | Keep durable selected-node data in `selectedNodeEditorProjection.svelte.ts`.                                  |
| `graphRendererCommands.ts`                | Transient command application       | Applies validated renderer selection/inspect commands to typed transient graph selection helpers.                                        | Keep as transient selection adapter; durable graph mutations must still use backend commands.                 |
| `graphRendererWindow.svelte.ts`           | Projection/status cache             | Stores the last backend-projected renderer-window status for display and renderer lifecycle controls.                                     | Keep as status projection only; do not derive durable graph state here.                                       |
| `selectedNodeEditorProjection.svelte.ts`  | Backend projection cache            | Caches the focused backend editor projection keyed by selected timeline node ID with request-id and stale-response guards.                | Keep as the editor read model; do not patch durable node fields locally.                                      |
| `notifications.svelte.ts`                 | Transient UI state                  | Owns discardable toast messages.                                                                                                          | Keep.                                                                                                         |
| `objectFieldProjection.svelte.ts`         | Projection cache and command bridge | Caches backend object-field projections and replaces cache from command responses with stale-response guards.                             | Keep.                                                                                                         |
| `project.svelte.ts`                       | Transient/session metadata          | Stores active project metadata used to gate the shell after backend create/load.                                                          | Keep visible durable project data projection-backed; do not store full `Project` DTOs here.                   |
| `projectSession.ts`                       | Projection refresh orchestration    | Owns project create/load activation ordering, lifecycle clearing, transient-state reset, and initial projection refreshes.                | Keep as the single project activation owner; do not hydrate broad project/timeline DTOs into stores.          |
| `projectionCacheGuards.ts`                | Projection cache infrastructure     | Provides shared version guards for replace-only projection cache writes.                                                                  | Keep as infrastructure; projection stores should use it instead of ad hoc stale-response checks.              |
| `projectionRefreshQueue.ts`               | Projection refresh orchestration    | Coalesces backend event/project-triggered projection refreshes and resolves queued waiters during teardown.                                | Keep as the single refresh coalescing owner; do not start per-component refresh state machines.               |
| `propagationProposalProjection.svelte.ts` | Projection cache and command bridge | Caches backend proposal list and replaces cache from command responses with stale-response guards.                                        | Keep.                                                                                                         |
| `timelinePlacementSession.svelte.ts` | Transient UI state | Captured per-node placement intent and immutable uncertain retry within the editor session. | Canonical placement and review remain backend/projection-owned. |
| `scriptBlockEditSession.svelte.ts`        | Transient UI state                  | Retains independent block text drafts, captured revisions and immutable pending edits across Script consumer replacement. | Keep session scoped; canonical text and propagation remain backend/projection-owned. |
| `scriptBlockCreationSession.svelte.ts`    | Transient UI state                  | Retains captured creation context, text and immutable pending payload/ID across Script consumer lifetimes; project activation replaces its owner. | Keep session scoped; canonical screenplay remains backend-owned and projection-backed.                        |
| `scriptDocumentProjection.svelte.ts`      | Projection cache and command bridge | Caches backend script document projections and replaces cache from command responses with stale-response guards.                          | Keep.                                                                                                         |
| `semanticProposalProjection.svelte.ts`    | Projection cache and command bridge | Caches backend semantic proposal list and replaces cache from command responses with stale-response guards.                               | Keep.                                                                                                         |
| `shortcuts.svelte.ts`                     | Transient UI infrastructure         | Owns in-memory shortcut registrations.                                                                                                    | Keep; ensure cleanup on component unmount remains deterministic.                                              |
| `storyArcProjection.svelte.ts`            | Projection cache and command bridge | Caches backend story arc projection and replaces cache from command responses with stale-response guards.                                 | Keep.                                                                                                         |
| `timeline.svelte.ts`                      | Transient UI state                  | Stores viewport, zoom, playhead, active tool, snapping, and connection drag only.                                                         | Keep timeline clips/tracks/arcs in `timelineRenderProjection.svelte.ts`; do not add broad timeline DTO state. |
| `timelineKeyboardCommands.ts`             | Transient command application       | Maps keyboard shortcuts to backend timeline command helpers without storing durable timeline data.                                         | Keep as a shortcut adapter; command responses must replace projection caches rather than patching clips.      |
| `timelineRenderProjection.svelte.ts`      | Projection cache and command bridge | Desired pattern for timeline commands: command responses replace the projection cache with stale-response guards.                         | Keep; add coalescing and shared command helper only after ownership cleanup.                                  |
| `timelineRendererWindow.svelte.ts`        | Projection/status cache             | Stores the last backend-projected timeline renderer-window status for display and lifecycle controls.                                      | Keep as status projection only; do not store renderer-owned timeline data here.                               |
| `serverEventHandlers.ts`                  | Projection refresh orchestration    | Routes Tauri backend events into projection refresh requests through `projectionRefreshQueue.ts`; does not hydrate or patch broad durable DTOs. | Keep as orchestration only; add new event handling by requesting focused projection refreshes.                |

## Alternatives Rejected

- Centralizing every UI field in one store file: rejected because it would collapse unrelated lifecycles into one mutable surface.

## Invariants

- Generation completion and matching node updates invalidate the selected inspector;
  timeline/hierarchy changes invalidate its canonical placement and hierarchy read.
  The shared refresh queue coalesces these reads. Event reads may publish only while
  their original selected node, editor session and handler remain current, in
  addition to request/version guards. They never change selection or rebase local
  screenplay/placement drafts or accept pending propagation proposals.
- Existing-block edit drafts are keyed by document/block in the active project
  session. Navigation, selection and canonical refresh cannot silently replace
  draft text or its captured base revision. An uncertain edit retries its exact
  payload/ID; only exact known native edit refusals unlock correction/discard.
  Explicit reload discards a draft only after its canonical read succeeds.
  Saved-text comparison uses the existing canonical read; its snapshot is
  separate from the retained draft. Only explicit continuation advances the
  expected draft revision, and later Save still rechecks it in the backend.
  Project activation resets these transient owners; this does not persist drafts
  across application restart or cancel admitted backend writes.
- The creation draft belongs to the active project session. Script/Graph/Split
  navigation and Script panel replacement retain its text, captured source and
  exact pending payload/ID, including acknowledgement failure while no Script
  consumer exists. Project activation replaces the owner; old callers cannot
  submit another command into the new session, and old completions cannot mutate
  its draft. This is transient memory, not persistence across application restart
  or cancellation of an already admitted backend write.
- Successful manual block creation replaces the screenplay cache from its
  canonical response and invalidates prompt memory. Refusal leaves cache and
  context revision unchanged; existing lifetime/version guards reject late data.
- Targeted preview responses populate the existing propagation proposal store.
  Failed/stale previews retain prior review state and expose the refusal. Explicit
  accept/reject commands remain separate from preview creation.
- Script projection refresh retains backend generation impact and exact authored
  output together. Version guards prevent older clean responses from hiding a
  newer Needs review cause, including explainable deleted-source evidence.
- Manual screenplay saves publish canonical projections and invalidate prompt
  context. Script-change events also invalidate context, including external
  edits; refused saves preserve the committed projection/context revision.

- Shared polling and backend event flows retain explicit ownership and cleanup semantics.
- Stores remain the source of transient UI coordination; components react to them.
- Editor reset advances its session generation. Keyboard delete/split completions
  may clear selection only in their original session while the command's target
  is still selected. Shortcut failures notify only their original session.
- Projection stores cache backend envelopes and must not patch broad durable entity state optimistically.
- Clearing the timeline projection cache starts a new generation. Earlier
  refreshes/commands still settle for their callers but cannot publish projection,
  error or pending state into that generation. Within a generation, envelope
  versions govern replacement, pending counts all outstanding requests, and only
  the latest-started request can report an error. This is cache ownership, not
  backend project custody or cancellation of in-flight writes/caller effects.
- Clearing a script document cache removes its lifetime identity. Earlier reads,
  block commands and lock commands still return their original results or reject
  for their callers, but cannot publish projection, error or pending state into
  the replacement lifetime. Within each document lifetime, envelope versions
  govern replacement, pending counts every outstanding request, and only the
  latest-started request can report an error. Clearing another document has no
  effect. This does not cancel backend writes or guard caller continuations.
- Backend contract changes are reflected here before individual components fork around them.

- Existing script/context/node/timeline events refresh only an already requested
  context stack through the shared refresh queue. They never activate an absent
  consumer. Request and projection-version guards retain post-save screenplay
  evidence against older responses and navigation/clear continuations.

## Revisit Triggers

- Another realtime channel or polling workflow appears without a clear current owner.
- A store starts mixing durable project data and ephemeral UI-only state without clear boundaries.

## Dependencies

**Internal:** `ui/src/lib/api.ts`, `ui/src/lib/serverEventClient.ts`, `ui/src/lib/types.ts`.
**External:** Svelte 5.

## Related ADRs

- `ADR-001` decomposition baseline for oversized frontend modules.

## Usage Examples

```ts
import { refreshTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';

await refreshTimelineRenderProjection();
```

## API Consumer Contract

- None identified as of 2026-03-08.
- Reason: stores are an internal frontend coordination layer.
- Revisit trigger: store modules become a published state-management package.

## Structured Producer Contract

- Projection cache stores hold backend `ProjectionEnvelope` payloads as replace-only snapshots. Components may read cached envelopes but must not mutate payload contents.
- Transient stores own local interaction state only. If the backend stores or acts on a field, that field must be changed through a command helper and refreshed through a projection response or invalidation.
- Changes to shared store fields must land with dependent component, backend event, README, and verification updates in the same logical slice.

### Bible changes invalidate story consumers

The bible_changed event reloads screenplay impact together with Bible/history
projections and invalidates cached prompt context. Canonical generated field
dependencies derive the review state; the frontend does not infer dependencies
from text or write replacements. Existing edit/creation draft owners remain intact.

Bible-driven impact changes carry an advanced canonical screenplay projection
version. A deferred older read cannot hide a newer fact review cause while exact
block text remains equal. The normal version/session guards continue to own cache
admission; no frontend-generated revision or automatic retry is introduced.


Timeline child creation uses the same runTimelineProjectionRequest and cache
lifetime/version guards as other timeline writes. Selected-parent authoring uses
the existing editor session generation to refuse late selection side effects.

Context influence changes invalidate cached prompt context and refresh canonical screenplay/review projections alongside the active Bible graph. They reuse backend context revision clocks and never patch durable text locally.

Story arc changes use the same event ownership: invalidate cached prompt context
and refresh canonical arc, screenplay and context-stack projections. The backend
captures consumed arc fields and identifies affected saved blocks. Event refresh
preserves per-block manual drafts, pending proposals and placement intent; it
never accepts a targeted update or rewrites saved screenplay in the frontend.

Bible relationship label writes forward the editor's original expected edge
revision and label-only payload. Node detail caches include owned edge revision
IDs. Delayed responses still pass existing projection version guards; failed
writes preserve cached material and clear pending state. Incoming endpoint detail
is invalidated/refreshed through the existing edge mutation path.

A committed label replay returns no projection. The label store resolves that
response and clears pending state without replacing source detail or invalidating
the target cache. Ordinary projected responses still use the existing version
guard, and interrupted/stale failures still preserve cached material.

The timelinePlacementSession owner retains transient placement intents per node within the existing editor session generation. Old project callers cannot issue commands, and new sessions get independent drafts. Canonical ranges and downstream memory remain in the existing timeline/script projections and range writer; this store is not a parallel story authority.

Explicit Bible recall is a disposable inspection cache in
`bibleRecallProjection.svelte.ts`. Reads require a user action and guard the
request, exact query, selected entity, editor session and prior revision floor.
Bible command admission/completion and `bible_changed` invalidate pending reads
and displayed evidence synchronously. Selection and project-session cleanup
revoke reads; late successes/errors cannot publish into a newer inspector.
Existing screenplay drafts, placement intents and proposal caches remain owned
by their current stores and are preserved during recall and Bible invalidation.

Mounted recall inspectors retain the shared read by anchor and editor session.
Disposing one inspector preserves another current inspector's pending/displayed
evidence; disposing the last revokes responses without setting the fact-change
notice or resetting the revision floor. Query changes also revoke without
claiming a canonical change. Bible writes/events alone set the invalidation
notice. Releases are idempotent, and obsolete session/anchor owners cannot revoke
a current inspector. A per-component cache was rejected because both visible
inspectors consume the same projection; revisit if selection becomes independent.


### Owned Bible node-detail refresh

`bibleGraphNodeDetailProjection.svelte.ts` owns the existing detail cache and its
read lifecycle, extracted from the command/list facade to avoid expanding that
large module. The facade reexports its prior API and state identity. Mounted
`BibleGraphNodeDetail` consumers retain/release inspectors by node; two visible
inspectors may share one read cache. `bible_changed` coalesces owned detail reads
through the existing projection refresh queue without clearing live projections
or recreating field editors. Request identity, node identity, project session,
mounted ownership and projection version admit each response. Retired reads
cannot publish success/error or clear a newer pending read; old releases cannot
clear a newer inspector. Project cache cleanup retires all detail reads.

A disposable verified-read flag shares this cache lifetime. Starting a read
invalidates it; only an owned response admitted by the existing version guard
restores it. Failures and pending retries preserve cached values and drafts while
field Save stays disabled. Command responses cannot recover a failed read, and
older read responses cannot certify a newer cached value. Visible cached-detail
errors expose ordinary retry without destroying the editor.

This is UI cache coherence, not project-switch recovery. It does not create a
parallel fact store. Command/list behavior remains in the original facade;
existing version guards still apply to command responses. Actual client tests
qualify shared inspectors, clean/dirty forms, selection ABA, delayed read success
and failure, project reset, wrong-node payloads and newer request ownership.
