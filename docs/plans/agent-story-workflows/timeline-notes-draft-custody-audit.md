# Timeline Notes draft custody: admitted authoring successor

Starts from integrated PR27 head `515b4b17b9461c49aa2ea208ffba87b817b77ce4`
(tree `eac8c47b7a2d229a3e254ee2d1d76e92ad51c5ab`) on separate
`feat/timeline-notes-draft-custody`. PR27 and the independently preserved
block-type product57b0 / QAee167 remain outside this branch's write set.

## Concrete evidence before implementation

The roadmap's manual-authoring acceptance includes repeated/interrupted writing,
unchanged human drafts and propagation into inspectable agent memory. Selected
and ancestor Notes already propagate through existing history, generation source
receipts and targeted screenplay review. Adding another Notes command, semantic
store or preview/accept flow would duplicate working behavior.

`BeatEditor.svelte` creates one component-local 500ms
`createDebouncedNodeNotesSave` for every selected clip. `handleNotesInput` passes
the exact textarea value into `schedule`; `onDestroy` calls `dispose`.
`debouncedNodeNotesSave.ts` retains only a timer, cancels it before every new
schedule, cancels it on disposal, and invokes `void save(...)` without owning a
failed/uncertain submission or retained draft. `BeatNotesPanel.svelte` renders the
committed projection's Notes; unlike title editing, no session Notes draft owner
can restore cancelled input when Script returns. The current Notes command also
lacks the title/range commands' expected author read.

An outside-Git reproduction executes the actual TypeScript helper with a virtual
clock and checks the editor's actual schedule/disposal call sites. With exact
input `  Mara reveals the witness — 雨.\n\n  `, disposal before500ms issues zero
canonical commands and exposes no retained-draft read. Scheduling input for a
second clip before500ms issues only the second clip's command. Existing three
debounce tests pass because they explicitly specify those cancellation semantics.
This is source/helper evidence, not a native UI or SQLite integration claim. Its
receipt is retained in `timeline-notes-draft-custody-evidence/notes-custody-baseline.json`
outside Git. No model/provider execution occurred.

The missing feature is custody of the authored Notes intent during ordinary
same-project selection/panel changes, failed saves and uncertain acknowledgement.
Without a canonical save, these intended story changes never reach graph-backed
generation/agent reads or the existing downstream review machinery.

## Bounded implementation plan

- Own exact Notes text, original read and save status in the existing editor
  session, separately per clip; reuse the title/edit session lifetime patterns.
  Retain the current Notes command and autosave cadence. Leaving a panel must
  neither discard an intent nor retarget it to the newly selected clip.
- Carry an optional exact author read using existing owned Notes field history;
  compare it inside the current writer transaction. Keep legacy wire behavior and
  command replay. Notes edit/restore ABA must refuse a stale read without writes.
- Preserve dirty Notes against canonical refresh. Show current committed conflict
  evidence and explicit discard/reload; do not silently rebase authored text.
  Unknown completion retains immutable command ID/text/read for an explicit retry.
  Keep later queued authored text separately from the submitted immutable payload.
- Acknowledged writes continue through canonical projection refresh, source-memory
  invalidation and existing stale-result guards. Agent/prompt reads use saved
  Notes only. Existing downstream proposals still require separate acceptance
  before replacing any saved screenplay; do not add an alternative review flow.
- Qualify actual client session/proxy behavior and public SQLite command/history
  behavior locally, including two clips, panel remount, concurrency, ABA, failure,
  uncertain retries and unaffected drafts/screenplay. Independent review precedes
  publication of implemented product behavior.

## Current milestone and proof limits

This checkpoint begins the separate feature with inspected source, a reproducible
missing-custody receipt and an admitted plan. Runtime implementation remains
pending; no draft-preservation fix, writer guard, screenshot or native acceptance
is claimed. The owner paused new hosted CI spending. Continue local qualification
without changing required main merge gates. No new durable store/schema, model or
embedding dependency, project-switch recovery or real-model quality claim belongs
to this slice.
