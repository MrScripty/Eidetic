# Native Bible recall lifecycle qualifier

## Purpose

Provide labelled native controls for production recall component lifetimes and delayed completions.

## Contents

| File | Role |
| --- | --- |
| `RecallLifecycleInspector.svelte` | Wrap unchanged production inspectors with final disposal/remount controls. |
| `RecallLifecycleControls.svelte` | Native controls and accessible receipts, labelled as QA fixtures. |
| `recallLifecycle.svelte.ts` | Hold actual domain-read completion; optionally reject with a synthetic error. |

## Problem

Closing the final ordinary inspector clears selection. The regular API has no delay seam, so it cannot isolate same-anchor final disposal or force late failure.

## Constraints

The production component/store/transport bytes remain frozen. No DOM/IPC injection, canonical writes, model or alternate dependency route. Synthetic completion errors must remain labelled.

## Decision

An explicit separate Vite qualification config instruments only component hosting and recall completion. The ordinary sidebar lifecycle is tested with normal app controls; compiled QA controls isolate the final owner without changing selection.

## Alternatives Rejected

Modifying production source introduces an unnecessary lifecycle test dependency. Relabelling earlier captures would break source/run custody. Timing a fast read without a seam does not establish a delayed completion.

## Invariants

Every successful recall value comes from the original desktop command/domain read. Delays occur after that read and before its Promise completes. The unchanged production component runs actual client effects. Manual text, drafts and pending proposals remain in their existing stores.

## Revisit Triggers

If the app adds a normal final-inspector hide control or request-delay facility, use it and retire the corresponding qualifier seam.

## Dependencies

Existing Svelte/Vite, production recall/transport modules and the normal native qualification workflow.

## Related ADRs

No new architectural decision in the application; this is an isolated test host.

## Usage Examples

`cd ui && npx vite build --config ../scripts/recall-lifecycle-vite.config.mts`

The normal `npm run build` does not install these controls.
