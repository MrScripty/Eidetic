# Native selected recalled-fact qualifier

## Purpose

Label native controls that isolate stale selection replay and far-peer canonical mutation.

## Contents

| File | Role |
| --- | --- |
| `RecalledFactsControls.svelte` | Visible QA buttons and readonly accessible receipt. |
| `recalledFacts.svelte.ts` | Capture exact real selected request; canonical mutation and stale replay through unchanged public commands. |

## Problem

The normal UI retires selection when recall is invalidated. It cannot submit its exact prior selector after a mutation, which is necessary to qualify backend source guards separately.

## Constraints

Application source stays byte-identical to frozen `e140290`. No DOM injection, replaced IPC, direct database writes, network changes or model downloads. Provider replies are labelled synthetic.

## Decision

An explicit qualifier Vite config wraps only the proposal store's command API import to record real selected requests. New visible controls call the unchanged production public commands for a far-peer fact edit and an exact stale replay. Selection, preview, reject and acceptance use ordinary application controls.

## Alternatives Rejected

Editing production files changes the frozen implementation. Silent script injection bypasses the native application path. Relabelling earlier screenshots cannot qualify this source.

## Invariants

Actual canonical services resolve chosen values and revisions. The wrapper passes requests unchanged. QA replay uses a new proposal/command identity and the old exact selectors; it must refuse before HTTP work. Explicit acceptance alone replaces saved text. Unrelated drafts and prior proposals retain their existing owners.

## Revisit Triggers

If the ordinary app exposes safe stale-request inspection or far-peer editing, replace the corresponding QA controls.

## Dependencies

Existing Svelte/Vite, command API, proposal/recall stores and supported hosted GTK/WebKit qualification workflow.

## Related ADRs

No production architecture change; this is an isolated qualification host.

## Usage Examples

`cd ui && npx vite build --config ../scripts/recalled-facts-vite.config.mts`

The ordinary application build does not install this qualifier plugin.
