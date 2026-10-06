# Committed relationship-label replay after source deletion

## Source and finding

- Preserved PR14 predecessor: `bc041d890a54a8124cab428d8e50bfab5b47d0b3`,
  tree `2798f1b221cf9504dba40825d8eddc416dc8a528`.
- Repair: `55536fa85fb91b0673d94e97b9f06b7297f23b12`,
  tree `39da595626dd68ff73dc91f859aff6819b4ba42f`.
- Verified [review finding](https://github.com/MrScripty/Eidetic/pull/14#discussion_r4198977466).

An actual backend regression commits a label, deletes its edge through the
canonical command, then deletes the source node through the canonical command.
On the predecessor, replay fails with `Bible relationship source no longer
exists` even though existing history recognized the exact command ID/payload
and performed no write. The pre-repair regression fails 0/1; the UI null-response
regression also fails because cache admission reads a null projection's version.

## Narrow behavior change

The label apply path now returns `(AlreadyRecorded, None)` immediately after
existing history validates committed replay. It performs no live graph lookup
and cannot resurrect deleted graph material. Fresh writes retain the same
writer closure, owned-revision/active-identity checks, label-only update and
revision delta; successful fresh writes still return their source projection.

Only the label command gains `BibleGraphEdgeLabelCommandResponse`, with an
optional projection carried through the shared service and Tauri adapter.
Other Bible command responses keep required projections. The frontend API
forwards the response; its label store treats null projection as success,
clears pending state and leaves source/target caches untouched. Projected
responses still pass the existing version guard. No schema, dependency,
history-store implementation, provider, screenshot or qualification-driver
change is introduced.

## Executed qualification

- Server: **472 passed**, including all five label-command regressions.
- Core: **119 passed**.
- UI: **458 passed** across 77 files. The new API test preserves the exact
  null response; the store test preserves newer source and target caches.
- Strict server all-target Clippy with `-D warnings`: passed.
- Typecheck: zero errors/warnings. Lint, complete frontend format check,
  production UI build, Rust formatting, whitespace and decision traceability:
  passed. Normal source commit hooks passed.
- Locked all-feature dependency metadata: no-download feature union passed.
  Existing no-download checker tests: **5 passed**. No ONNX bypass was used.

The deletion regression executes replay with SQLite `query_only` enabled,
asserts `AlreadyRecorded` and no projection, compares every logical table row
and asserts unchanged `total_changes`. Repeating the payload under a fresh
command ID still refuses with the existing stale/deleted message and unchanged
rows; reusing the committed ID with a different payload also refuses unchanged.
Existing tests continue to cover live/edge-only replay, two-connection
structural/delete conflicts, owned sparse-history conflict, label ABA and
delete/recreate. Full server tests use the same task-local writable XDG fixture
directories as the original qualification.

The current local host lacks GLib/GTK/WebKit development packages. An actual
`cargo check --locked -p eidetic-desktop` stops at missing `glib-2.0.pc` in
`glib-sys`, before checking the Tauri adapter. The pre-push launcher's full native
workspace test was therefore excluded for this push; its affected server/core/UI
paths and frontend checks were run separately. Existing ordinary PR CI performs
the complete workspace checks on hosts that install those packages. Source
push started [CI run 37512093897](https://github.com/MrScripty/Eidetic/actions/runs/37512093897),
queued at this evidence checkpoint. Desktop qualification remains pending there;
this report does not turn a local environment failure into a pass.

The original native run37505367170, frozen branches, report, manifest and
screenshots remain intact. The original report and manifest are byte-identical
to bc041d8. That native execution qualifies its pinned predecessor, not this new
deleted-source replay. Its provider responses remain explicitly synthetic.

## Evidence custody and delivery

Local regression, gate, commit, desktop failure and push logs are retained under
`/workspace/scratch/relationship-lineage/replay-*`. The companion
`bible-relationship-replay-repair.json` records sealed log hashes and paths.
The normal, non-forced push updates the existing PR14 feature branch; bc041d8
remains an ancestor. No review-thread resolution, manual CodeRabbit request,
merge, credential change or alternate screenshot service was performed.
Parent retains final CI/review and merge ownership.

## Exact-successor hosted native qualification

The separately preserved qualification branch
`test/screenplay-bible-relationship-replay-native` at
`f9081da5dbc37a7fa50d9edaf04f0849464238f5`, tree
`76cf658cd279eadb61e3ce642ab357fcdd8f8088`, pins application source
`55536fa85fb91b0673d94e97b9f06b7297f23b12`. The enforced source comparison
permits only the qualification workflow, replay report/JSON and execution ledger;
every application, test and dependency blob matches that exact successor.
The original label-native branch remains at its earlier qualification.

[Run37512778165](https://github.com/MrScripty/Eidetic/actions/runs/37512778165),
job `112437930692`, **succeeded**. The ordinary locked desktop and fixture builds
passed on the standard hosted Linux environment. Core119/server472/UI458,
driver39/no-download5 and frontend gates passed. An additional named hosted
backend gate executes the exact deleted-source replay regression: **1 passed,
0 failed**; its untouched stdout is included in the artifact. This closes the
local missing-GLib build limitation for the source-pinned native application.

The maintained actual Tauri walkthrough again performs the exact Red-to-Blue
Bible field edit and Trust-to-Doubt relationship label edit, preserving saved
screenplay, timeline placement and the exact manual draft until the existing
explicit review decisions. Relationship label ABA refuses the old acceptance
with the exact stale message and identical before/after logical table snapshots.
Fresh preview retains the newer original-read revision; explicit acceptance
changes only generated scene B and refreshes its UsesFact revision. Manual A
stays exact. The final capture shows saved Doubt text, the Blue Bible fact,
bounded exact saved-label inspection and timeline together.

Artifact `11436690798`, **980112 bytes**, ZIP SHA256
`c07c120c8b9fe5eea76f140e7db9ddfeb27b11bd227cf93a5a0dd4027933b5b7`;
`capture-evidence.json` SHA256
`6a3d0ce78698fe0c41bb425a7bd7d1599a4da6f7555c2f0a692d9ac783006baa`.
Nine untouched1440x960 captures match recorded hashes and owned window/PID.
The first four are byte-identical to previously reviewed captures; the five
changed relationship captures were individually visually reviewed. Both top-level
fact status and nested relationship status pass; five accepted/context-checked
HTTP/SSE provider calls all explicitly record `real_model: false`.

Deleted-source command replay is exercised by the actual backend regression in
this hosted run. The native UI walkthrough exercises ordinary label saves and
explicit screenplay review; it does not expose a command-ID replay gesture.
Frontend null-response handling is qualified by executed API/store tests.
No real-model quality or native multi-user race claim is added.

The source/report successor PR head4e13849 also passed hosted Linux workspace
Clippy/tests, Frontend and Decision Traceability at the final evidence check;
Windows tests were still running. The qualification workflow is integrated into
the feature branch by fast-forward, and this evidence update changes only docs.
Parent retains final head CI/review. Original successful/failed native runs,
bc041d8 and the original report/manifest/screenshots remain preserved unchanged.
