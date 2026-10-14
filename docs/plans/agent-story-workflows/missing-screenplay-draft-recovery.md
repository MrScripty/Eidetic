# Missing saved-block draft recovery

Base: independently accepted Notes c703ebb and Bible exact-text 9998e15.

Before implementation, the actual ScriptPanel rendering regression retains exact
Unicode/newline manual text and its original revision in the session edit owner,
then reads a canonical document without that block. The panel renders no draft.
Only orphaned removal requests were displayed; canonical disappearance made an
active edit and its unknown-acknowledgment retry inaccessible. The preserved
baseline fails its rendered-text assertion. This is a concrete authoring loss of
access, separate from project switching and the accepted block-type operation.

Reuse the same per-block session edit owner. Surface unavailable drafts without
claiming a deletion from a pending/failed read; preserve original block/revision
identity. Read/check and immutable original retry remain under existing guards.
Explicit discard acts only on this draft. Explicit copy enters the existing
single composer for the visibly selected canonical clip, preserving original
text and kind and leaving the original draft intact. A busy/active composer or
uncertain original save refuses copy; no automatic recreate, retarget or commit.
Cancel of the copy preserves the original. Ordinary Save uses existing canonical
CreateScriptBlock, UserEdited provenance, history and downstream review. A copy
is a new authored block, not restoration of old identity or consumed lineage.

Qualification covers missing-block panel/remount, independent drafts, same-ID
return, reads failing or returning old versions, pending/lost-ack exact retry,
repeated copy/save/cancel and actual public create/remove storage. Preserve
saved unrelated material and require explicit downstream proposal acceptance.
No backend API/schema, dependency, alias mapping, new canonical store or native/
real-model qualification is implied. No hosted jobs or main merge.

## Local qualification

608 UI tests in 105 files pass; independent preliminary review reran 22 focused
tests. Svelte check reports zero errors/warnings. Final rendered Chromium uses
actual AppShell, Bible inspector, screenplay and timeline with a temporary IPC
adapter to real public SQLite services. It removes a saved block externally via
its ordinary canonical removal command while its exact manual draft is open.
The draft remains visible through panel navigation, explicit copy/cancel and
copy Save with a dropped post-commit acknowledgment. Exact retry creates one
UserEdited block, retaining the original draft and unrelated saved blocks.
The public agent-context preview contains exact copied text. Existing targeted
preview preserves all saved blocks; explicit acceptance changes only its bound
generated target text/revision. Only explicit discard clears the original draft.

The final walkthrough has three labelled synthetic HTTP replies: initial
generation, recap and targeted preview. Nine replies total are retained across
two earlier rehearsals plus the final isolated project. The final responder
distinguishes generation/preview by actual prompt, rather than a global count;
the final assertions require distinct target text/revision and complete equality
of every other block. Earlier layout rehearsals are retained separately.

These are instrumented browser/public-service checks, not native/release or
real-model narrative qualification. Native renderer status is unavailable in
the test adapter and visibly reports that limitation. Runtime exception checks
pass; they do not hide missing-adapter-route transcript errors. No reopen after
downstream acceptance is claimed. Stable served module bytes are retained after
the run, not capture-time network attestation. Temporary UI route, Vite config
and Rust adapter example are removed/restored before the product commit.

Evidence is outside Git under
`/workspace/scratch/missing-screenplay-draft-recovery-evidence`. Parent owns PRs,
merge gates, hosted native qualification and Library delivery.
