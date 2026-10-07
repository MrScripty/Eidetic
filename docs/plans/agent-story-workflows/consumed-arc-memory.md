# Consumed story-arc context and screenplay review

## Source and bounded scope

Branch `feat/screenplay-arc-memory` starts at PR17's frozen source
`5ac9eb99ffb155878d5dcfda77baa01d60ef24f7`, tree
`ba5c1d15e3ce002511190936441d7b322164ccf4`. Its parent is verified main
`25895e7bcf215e5a013dd7c4a98873e9b7412b8a`. PR17's source and evidence remain
separate and untouched. Parent owns review, PRs and merges.

ArcDetail already submits typed name/description changes to the canonical
story-arc metadata command. That command records sparse StoryArc field history.
The screenplay prompt consumes tagged arc name, type and nonempty description,
but generation receipts and semantic dependency endpoints cannot retain those
inputs. Later edits therefore cannot mark consuming saved scenes for review.
The active workflow plan explicitly leaves arc-description binding separate.

## Consumed-field ownership

Read canonical node tags, arc values and each field's owned history in one
SQLite snapshot, alongside target custody. Replace the tagged arc prompt
context with that same read. Retain exact supplied field values in the existing
generation-command JSON. Name and arc_type are always supplied; description is
supplied when nonempty. Arc type uses its canonical serialized value. Color,
parent, ordering, untagged arcs and recap arc labels are outside this receipt.

Each supplied field has its own latest owned new-value revision. Validate the
historical value against that event; reject contradictions between history and
canonical values. Existing template arcs can lack field history: retain the
exact consumed value with an explicitly missing revision, create no invented
dependency, and never backfill an old generation. Absent legacy receipt JSON
remains unknown. Color-only changes do not advance any prompt-field clock.

Dependencies use the existing semantic storage and generation event. Impact
compares only originally bound consumed fields. Keep original exact values and
revisions in durable receipts, with historical excerpts for review. A fresh
targeted preview captures current tagged-arc values/history and explicitly binds
deletion of previously consumed arcs. A live consumed arc outside current
context refuses preview rather than silently dropping its lineage. Clearing a
previously consumed description retains its current empty-field evidence.
New tag membership and previously unconsumed empty descriptions are separate.

## Review and custody

Reuse existing targeted proposal, preview and acceptance. Include changed arc
evidence in the visible review and provider prompt. Validate the same binding
under the writer before proposal storage and explicit acceptance. Preserve
target revision/lock/placement, sparse-history ABA, deletion/restore, uncertain
replay and frontend project-session/result guards. A late normal generation
retains the historical arc input it actually consumed; it may need review on
arrival, and must never claim it consumed a later edit.

Saving arc context or requesting preview never replaces saved screenplay.
Explicit acceptance replaces only the chosen block and installs newly consumed
lineage. Retain unrelated manual blocks and drafts. No parallel state, database,
embedding/model dependency, inferred world update or project-switch feature.

## Acceptance and qualification

- Exact public typed description edit marks consuming generated scenes only.
- Name/type field clocks and description clearing/deletion retain exact custody;
  unconsumed arcs, color-only changes and missing legacy receipts stay distinct.
- Generation receipts keep historical consumption across delayed output, replay
  and later context edits. Forged or inconsistent owned field evidence refuses.
- Preview supplies changed input and leaves canonical text/drafts intact.
  Explicit acceptance replaces only the selected block and clears its cause.
- ABA, deletion/restore, further edits, target edits and locks refuse stale
  previews/output without losing drafts or installing partial results.
- Deterministic server/UI tests exercise the existing public commands and
  projections. Run maintained checks on exact source.
- Hosted native qualification uses actual application controls and public
  service fixtures, showing Bible, timeline and screenplay in review captures.
  Preserve original screenshots, exact manual text and source/receipt hashes.
  Synthetic HTTP model responses are labelled; real-model quality remains
  unqualified. Use existing no-download admission; never bypass local ONNX403.
