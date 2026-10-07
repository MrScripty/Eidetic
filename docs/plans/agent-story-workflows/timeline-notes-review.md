# Selected timeline Notes and screenplay review

## Verified gap and scope

Separate successor `feat/timeline-notes-screenplay-review` starts at PR21
`1bf7873579705a824ad45a1c95435e700cb09eda`, tree
`21c329291edbb06997748eecd79f156dc2628d26`. PR19/20/21 remain draft and unmerged.
Exact placement already propagates screenplay ranges and continuity review.
`prompt_format` supplies the target clip's exact Notes; `ScriptGenerationTarget`
already retains those Notes and the canonical node event before provider I/O.
The notes writer already records sparse owned `notes` revisions and publishes
`node_updated`, which rereads saved screenplay without resetting drafts.
However, generation records no Notes dependency, and targeted prompts omit Notes.
The baseline public-command regression
`manual_timeline_notes_edit_marks_its_saved_generated_screenplay_for_review`
ran exactly once and failed 0/1: saved generation then exact authored Notes edit
produced no review cause. The local failure log stays outside source Git.

## Decision and owners

Reuse the existing generation target receipt, command/proposal JSON, sparse node
history and semantic dependency store. Resolve the owned Notes field event at
the captured target node event; do not substitute today's Notes or node clock.
An existing TimelineNode endpoint with a distinct dependency identity represents
that supplied field. Presentation range, status and recap events do not advance
the Notes clock. Absent legacy target receipts and unowned Notes history remain
explicitly unknown; no historical consumption is invented.

The server owns capture, historical validation, impact and writer recapture.
The existing review proposal retains exact original/current Notes and revisions,
supplies the current text to the targeted prompt, and rejects changed or restored
Notes before storing or accepting results. Explicit acceptance installs current
Notes lineage alongside the existing screenplay/Bible/arc dependencies.
The UI displays a Notes-specific cause and exact evidence through the existing
Preview update / Accept update / Reject flow. Existing event, proposal and draft
owners stay authoritative. Notes edit and preview never replace saved screenplay.

## Acceptance and limits

- A public exact Notes edit marks only its proven consuming generated segment.
- Preview shows original/current exact Notes and preserves saved text, placement,
  unrelated authored blocks and drafts. Explicit acceptance alone replaces the
  chosen block and refreshes consumption; a later Notes edit derives review again.
- Notes clearing, field-owned ABA and deletion retain honest source custody.
  Delayed preview, target edits and locks refuse without partial history/results.
  Exact replay adds no provider call or canonical write.
- Backend regressions use the real SQLite command/projection/acceptance writers.
  UI evidence/render/event tests exercise maintained application components.
  Any provider fixture is labelled synthetic; real-model quality is unqualified.
- Ancestor/sibling Notes, names, beat types, arc tag membership, fictional-time
  inference, project switching, new models and embeddings are separate follow-ups.
- Raw screenshots/logs/PDFs/portable archives remain ignored or external. Ordinary
  display derivatives use JPEG quality 85; integrity-bound originals stay intact.
  Existing originals still require parent-owned durable delivery before Oct 10.

## Alternatives and revisit triggers

Automatic regeneration would replace authored work without acceptance. A second
memory or revision store would duplicate canonical evidence. Comparing whole-node
events as Notes revisions would make ordinary range/status changes look semantic.
Revisit when ancestor/sibling prose or unowned legacy source history is explicitly
scoped, or another consumed timeline field needs its own source receipt.
