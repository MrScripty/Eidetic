# crates/core/src/contracts

## Purpose
This directory defines host-agnostic command, event, revision, and projection contracts used by the projection-architecture refactor.

## Contents
| File/Folder | Description |
|-------------|-------------|
| `mod.rs` | Public contract types, validated IDs, typed values, change history records, and projection envelopes. |
| `agent_workflow.rs` | Backend-owned agent workflow, scoped tool manifest, tool request/result, budget, run, and policy contracts for graph-aware harness work. |
| `ai_context.rs` | Generation context with explicit fictional-time scope, effective sparse snapshot facts and unresolved timed-field references. |
| `affect.rs` | Backend-owned affect contracts for valence, arousal, mood, intensity, confidence, provenance, and scoped targets. |
| `bible_graph.rs` | Canonical story-bible graph contracts, expected root nodes, typed graph parts/fields/edges, and node-detail projection shapes. |
| `bible_graph_defaults.rs` | Built-in story-bible schema defaults used to project expected empty parts and fields for known graph schemas. |
| `bible_render_graph.rs` | Disposable Bevy-facing story-bible graph projection DTOs, deterministic layout helpers, and neighborhood indexes derived from canonical graph rows. |
| `context_influence.rs` | Timeline context stack with optional exact saved screenplay evidence and recorded evaluation/influence contracts. |
| `graph_proposal.rs` | Generic reviewable graph proposal contracts for agent-proposed bible nodes, fields, edges, and timeline-context links. |
| `script_document.rs` | Canonical script document, segment, block, span, lock, patch, and script projection contracts. |
| `script_impact_review.rs` | Explicit targeted preview request, bounded author-selected recall identities and captured screenplay/world-context evidence for revision-bound propagation proposals. |
| `semantic_dependency.rs` | Typed semantic relationships with optional source/target revision bindings for generation lineage. |
| `timeline_render.rs` | Timeline renderer projections, including core-derived gaps filtered by the renderer's minimum duration. |
| `script_fact_reconciliation.rs` | Canonical saved-edit receipts and identity-only request for one consumed baseline fact proposal. |

## Problem
The new architecture needs stable types for backend-owned commands, event history, sparse object revisions, and read projections before persistence, routes, Svelte, or Bevy can implement their slices safely.

## Constraints
- Fictional `story_time_ms` is an explicit query coordinate, never derived from
  screen-time clip placement or authoring revision timestamps. Absent query time
  is representable and must not imply zero or the playhead.
- Contracts must remain independent from HTTP, SQLite, Svelte, Bevy, Y.Doc, and AI backend implementations; renderer DTOs describe data shape only and do not depend on renderer crates.
- Public wire shapes must be serializable and round-trip testable.
- Canonical queryable facts must remain typed instead of hidden inside arbitrary JSON.

## Decision
Start with small core contract modules that own IDs, object kinds, field values, generic field-update commands, graph-node create/root-initialization/field/edge/snapshot commands, script document commands, affect commands, agent workflow/tool boundary DTOs, built-in graph schema defaults, change events, object revisions, projection envelopes, and the first story-bible graph, bible render graph, and script read models. Later slices can add domain-specific command/projection payloads without changing runtime infrastructure first.

## Alternatives Rejected
- Defining contracts in server routes: rejected because route-owned contracts would couple persistence, UI, and Bevy to HTTP handlers.
- Defining contracts in TypeScript first: rejected because backend-owned state and validation must be authoritative.

## Invariants
- Manual block creation carries main-document identity, selected source node,
  captured placement, block kind and exact text. IDs and append ordering are
  backend-owned; unknown client-owned fields are refused.
- Contracts remain deterministic and host-agnostic.
- Timeline render gaps come from core occupied-range queries; overlapping edits
  must not project occupied time as empty space.
- Long-lived boundary types have explicit serde shapes.
- Object revisions describe field-level deltas and do not require whole-object snapshots.
- Canonical bible roots are system-owned graph nodes, not enum-only branches in application logic.
- Built-in bible graph defaults are projected read models until a user command persists an actual field value.
- Script documents own generated screenplay artifacts; timeline nodes are referenced only as source context.
- Manual edits carry a block write-event expectation and text, rather than
  client-reconstructed document/segment metadata. Script context carries separate
  block and segment write identities, preserving authored evidence separately
  from consumed content membership and placement. Appends change the segment
  dependency identity even when previously consumed block text is unchanged.
- Generation commands retain the exact supplied screenplay context in their
  replay identity. Bound dependencies identify both the successful output event
  and consumed input event. Missing lineage differs from a known empty input set.
- Optional complete screenplay-window receipts retain the selected node, range,
  ordered segment identities and append-only selection epoch in existing
  generation/proposal history. Older absent receipts remain absent in serde and
  replay identity. Context-change causes describe entering/leaving or changed
  order; they neither replace saved text nor infer fictional time.
- Needs review is a derived read projection with changed/deleted input causes;
  it does not change canonical segment status or authorize proposal acceptance.
- Impact identifies its generated output block; a targeted preview cannot resolve
  that impact by updating another block. Optional output identity preserves older
  wire reads. Review bindings retain explicit fictional query time through the
  existing resolved bible projection, separately from presentation placement.
- A preview request authorizes proposal creation only. Accept/reject remain
  separate commands, and rejection must preserve the writer's canonical edit.
- Agent workflows receive typed manifests, budgets, policies, and backend tool requests/results; they do not receive app, renderer, or frontend state.
- Affect values use validated integer basis-point domain types for valence,
  arousal, intensity, and confidence so invalid floats cannot cross contract
  boundaries.

- Context-stack screenplay evidence is optional for legacy compatibility; a known
  empty read differs from unavailable evidence. Exact text and existing block and
  segment revisions remain separate from recorded semantic summaries.

## Revisit Triggers
- Contracts become public SDK or binding surface.
- A persistence slice needs additional field value types.
- Projection envelopes need cross-process delivery metadata beyond version and change event.
- Bible graph schemas require richer typed field constraints than the current field value primitives.

## Dependencies
**Internal:** This is the lowest-level contract boundary in `eidetic-core`.
**External:** `serde`, `uuid`.

## Related ADRs
- `ADR-001` decomposition baseline.
- Refactor plan: `docs/refactors/eidetic-projection-architecture/final-plan.md`.

## Usage Examples
```rust
use eidetic_core::contracts::{ChangeEvent, ChangeEventKind, CommandId};

let event = ChangeEvent::new(CommandId::new(), ChangeEventKind::UserEdit, "edit script");
assert_eq!(event.summary, "edit script");
```

## API Consumer Contract
Targeted-preview requests optionally carry `recall_selection`: the existing recall
query, up to eight baseline field identities/expected revisions, and exact
displayed endpoint-name/path receipts. Nonempty selections require unspecified
story time in both recall and preview. The backend re-reads canonical recall;
selectors do not authorize client values or replacement text. Unknown sources,
snapshot-backed, unresolved and omitted facts refuse. Empty facts preserve the
existing no-selection behavior. Existing request, proposal and acceptance owners
retain replay, immutable binding and replacement authority.

- These types are stable internal Rust contracts for command/event/projection slices.
- External API exposure must add explicit boundary validation and serialization round-trip tests in the consuming layer.
- Compatibility is not required for pre-refactor project data.

## Structured Producer Contract
- `ChangeEvent`, `ObjectRevision`, and `ObjectRevisionField` are intended to map directly to SQLite command/event/revision rows.
- `ProjectionEnvelope<T>` is a versioned read model wrapper for Svelte, Bevy, AI, and export projections.
- Field value variants define typed persistence semantics; adding variants requires persistence and wire round-trip tests.

### Consumed Bible field revisions

`BibleFieldInput` binds an untimed resolved field's stable node/part/key/field
identity, exact value and write event to a generated screenplay command. The
optional input collection preserves older unbound generations. These inputs
become ordinary UsesFact semantic dependencies with revision bindings; there is
no additional canonical fact store. Timed overrides, unresolved fields, node
headers and relationships are outside this field-lineage contract.


Canonical generation carries optional ScriptGenerationTarget custody in the
existing request and generation command. Timeline/output/segment revisions plus
exact admitted notes and placement protect delayed completion and ABA; older
serialized requests/commands omit the receipt without invented backfill. The
backend enforces it, while canonical facts and complete-window lineage remain
independent consumed evidence. No new persistent memory owner or dependency.

Optional BibleContextScope stores scoped untimed entity/field IDs and the existing Bible/context epoch in generation/proposal history. Absent legacy receipts remain absent; BibleFieldInput retains exact values and revisions.

Consumed untimed Bible relationships retain exact edge payloads and source
revisions in optional BibleRelationshipInput receipts. BibleEdge UsesFact
dependencies reuse existing dependency storage; absent legacy receipts remain
unknown. Captured endpoints, kind, label and direction are historical evidence.
Targeted proposals also retain optional per-identity absence revision pairs for
previously consumed edges. These bind missing-state history without inventing a
consumed value; a missing legacy receipt requires fresh review.

`BibleNodeNameInput` binds each authored name actually supplied in a resolved
Bible prompt header to its node identity and owned name revision. Optional
`bible_node_name_inputs` in generation/proposal custody preserves unknown legacy
reads. Existing `BibleNode` / `UsesFact` dependencies carry these name receipts;
node metadata-only edits do not advance the name clock. Proposal absence receipts
retain deleted-node history, without inventing a consumed name.

Exact timeline placement uses optional TimelineNodeRangeRead expected custody in the existing range command. Omission preserves legacy command JSON signatures. A present receipt records exact start/end and a known optional node history event; absent receipt is not inferred from fallback projections.

## Explicit Bible recall
`bible_recall.rs` defines an exact entity and bounded one-hop inspection request,
typed directed paths, resolved fields with baseline/snapshot source revisions,
unresolved timed identities and explicit omissions. Limits are eight neighbors,
32 paths and 32 KiB; this evidence grants no generation or replacement authority.
`ReadBibleRecall` shares that contract within existing agent graph-read budgets.

## Consumed arc field contracts

`StoryArcPromptField` restricts dependency identity to name, description and
arc_type. `StoryArcFieldInput` preserves exact supplied values and optional owned
field revisions; missing template history is explicitly unbound. Generation and
targeted-review receipts add optional arc inputs, original consumed evidence and
deleted-arc revision custody. Absent legacy JSON remains unknown. Existing
semantic endpoints extend with `story_arc_field`; no parallel store is added.

Decision: per-field revision ownership makes color-only edits irrelevant while
retaining semantic ABA, historical late consumption and explicit acceptance.
