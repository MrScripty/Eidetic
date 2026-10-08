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
  Retain the current Notes command with explicit Save Notes. The latest owner
  instruction prohibits silently saving unaccepted text and supersedes the audit
  checkpoint's proposed autosave cadence. Leaving a panel must neither save nor
  discard an intent nor retarget it to the newly selected clip.
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

Implemented on the separate Notes branch: a session-owned per-clip draft replaces
BeatEditor's component-local debounce. Save Notes captures an immutable command
ID, exact text and existing `TimelineNotesInput`. The existing writer compares the
original author read in its transaction, excluding its tentative Notes event;
stale/ABA, wrong-node, locked and unchanged guarded edits roll back all rows.
Legacy clients omit the guard and keep their existing wire behavior.

The existing command response adds optional `notes_read`: the original historical
receipt of that exact command, including a replay after newer Notes or a late
lock. Missing/uncertain receipts retain retry custody. Only a proven precommit
refusal releases it. Failed postcommit publication/refresh cannot be labelled a
precommit refusal; independent projection refresh does not invalidate a received
owned acknowledgement. Later queued text remains unsaved. Agent reads and raw
prompt preview continue to use committed Notes; downstream screenplay replacement
still uses the existing explicit acceptance flow.

Local qualification passes 123 core and 597 server tests, including the public
SQLite guarded-write/replay/save-reopen path and a guarded Notes edit that updates
existing impact history while preserving all saved screenplay material. Nineteen
focused UI tests cover custody, concurrency, immutable retries, failed reads,
rendered controls and the late-lock recovery case. The full UI suite and static
checks are recorded in the external evidence logs at publication.

A local Chromium run uses real AppShell, Bible, BeatEditor, ScriptPanel and timeline
components through a labelled IPC test adapter to public Rust/SQLite services.
Seven screenshots and DOM receipts show exact unsaved input, panel remount,
uncertain acknowledgement after a real commit, retry despite a late lock,
acknowledged save, explicit B save, canonical conflict and session reopen.
The entire saved screenplay payload is identical before/after; the projection
version is allowed to advance with Notes history. No runtime exceptions occur.
The fixtures contain manually authored screenplay; downstream generated-consumer
impact is qualified by the separate SQLite test, not claimed from those pixels.
The adapter and transformed module bytes are retained outside Git for inspection.

Unsaved custody is transient within an open project session. Reset/reopen expires
unsaved intents and reads committed Notes; project-switch/hard-reload recovery is
still deferred. No new schema, durable draft store, dependency, embedding or model
call is introduced. This is instrumented local browser/public-service evidence,
not native runtime, uninstrumented release, new hosted qualification or real-model
quality. Native renderer status is unavailable in this browser adapter. No hosted
jobs were started and main merge gates remain unchanged. Independent acceptance,
exact source IDs and evidence hashes are recorded in the publication handoff.
