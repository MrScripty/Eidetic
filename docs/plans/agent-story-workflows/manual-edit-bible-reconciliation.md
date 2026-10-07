# One saved manual edit to one consumed Bible fact

Local successor of PR20 head `ed21f69250388bc387a84f489594bf48e63f2b33`.
PR19/20, their green CI and sealed native evidence stay unchanged.

The documented script-generation loop requires explicit analysis of authored
edits, proposed world changes and before/after acceptance. Canonical manual saves
and downstream screenplay review already work. `ScriptBlockEditor` has Save and
Compare but no fact analysis; existing `SetBibleField` propagation proposals have
a transactional writer. This slice connects those owners rather than adding a
semantic-claim store or another inference loop.

## Scope and invariants

- One explicit Analyze saved edit action, after a real `script.edit_block` save.
  The server loads exact before/after text and revisions from owned history.
- Choose one baseline text field actually consumed by the scene's current
  generation. Missing, generic/unbound, unconsumed, cleared and unresolved timed
  fields are unavailable. Identity-only requests carry no caller-supplied truth.
- Provider receives only the selected fact and canonical saved edit. Strict
  value/rationale JSON creates an immutable existing propagation proposal.
  Partial, malformed, target-injecting, empty and no-change output cannot commit.
- Pending review shows current fact before/proposed value after, rationale and
  exact saved-edit/consumed-fact provenance. Only Accept fact update changes a
  Bible value. Reject may retire a stale proposal without changing its sources.
- Source edit, segment placement/generation, fact revision and consumed name/
  relationship custody are checked again before storing and under the acceptance
  writer lock. ABA is a changed revision even when text returns to its old value.
- Preserve all screenplay blocks/spans, other facts/scenes, timeline placement
  and unsaved drafts. Acceptance derives existing Needs review; it never calls
  screenplay generation or replaces saved screenplay.
- Exact retry reuses command/proposal IDs and returns canonical current state
  without another provider call or write. A different proposal decision cannot
  repeat an already committed acceptance. Late responses cannot populate a newly
  activated proposal owner.
- No public writes, external review, paid services, graph expansion, embeddings,
  project-switch recovery or automatic analysis in this stage.

## Native acceptance plan (not executed here)

Use the existing hosted native/no-download setup in a separate qualifier branch
after parent authorization. Keep PR20's original qualifiers/reports untouched.
Use a clearly labelled synthetic local HTTP provider; do not claim model quality.
Do not retry local GTK or bypass local ONNX403.

1. Seed through public canonical services: Bible Mara umbrella red, another
   unconsumed sibling/Eli fact, two generated scenes consuming Mara's field and
   an authored association, and an unrelated saved scene/draft. Keep Bible,
   timeline and screenplay visible in the actual Tauri window.
2. Ordinary native typing and Save changes exactly one source screenplay block
   to `  Mara carries a blue umbrella — 雨.\n\n  `. Record the actual before/after
   revisions and confirm Bible remains red; drafts and placement remain exact.
3. Choose Mara's actually consumed field. Ordinary Analyze saved edit creates a
   pending synthetic blue fact proposal. Show source revisions, fact before/after,
   rationale and actual Accept/Reject controls in native viewport geometry. Check
   actual provider prompt contains only the chosen fact and exact saved edit.
4. Ordinary Reject preserves Bible, saved material and drafts. Explicit fresh
   analysis plus ordinary Accept updates only Mara's fact; both actual consumers
   visibly acquire Needs review, while all screenplay text/placement/drafts stay
   exact. Capture the accepted field revision and existing dependency cause.
5. In independent fresh fixtures, pause synthetic analysis or leave a proposal
   pending, then use public commands/native saves to edit/restore the saved text,
   fact or consumed association. Verify refusal before storage/acceptance,
   unchanged pending columns/history on refusal and preserved drafts. Do not
   infer a replacement relationship after association drift.
6. Replay exact request/decision IDs through labelled QA controls. Confirm no
   additional provider call or write, and preserve canonical accepted/rejected
   state. Reopen the same project to verify proposal/binding reconstruction.
7. Seal original screenshots, actual backend/prompt/SQLite receipts, exact source
   and served-source hashes. Report failed attempts as failed. Qualification is
   pending until the actual native walkthrough executes and originals are read.
