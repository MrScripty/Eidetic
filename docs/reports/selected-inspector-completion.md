# Selected inspector completion repair

## Outcome and exact source

Source `f05fa888245db0fff1dad5a3165b0c1d3e07539c`, tree
`f096352611b570f9c31617fcc4a97887874fac0d`, is published on PR17's existing
`fix/selected-inspector-generation-refresh` branch. Its direct parent is frozen
`5ac9eb99ffb155878d5dcfda77baa01d60ef24f7`. No merge, reviewer request, credential
change or model download was made. The separate arc feature was retained and
subsequently based on this repair.

Independent review identified a real completion blocker:
https://github.com/MrScripty/Eidetic/pull/17#discussion_r4201851986.
The original generation-complete event awaited the selected inspector reread
inside its projection Promise.all before clearing streaming state. An inspector
request that never settled therefore held an already completed generation open.

The successor schedules that nonessential inspector refresh independently.
Canonical timeline, screenplay and context refreshes retain their existing
completion gate. Session and handler ownership still guard completeGeneration.
Selected inspector reads retain the original selection/session/handler,
request-ID and projection-version guards, including stale-error suppression.
Current read errors remain exposed through the selected projection store and
the existing refresh queue. No speculative timeout or new error-swallowing
policy was introduced. The source delta is one production store and its tests.

## Deterministic qualification

Seven added regressions fail on the original source while all thirteen previous
tests pass. They hold the inspector promise unresolved, require generation to
finish, then resolve or reject after selection, session or teardown changes.
The seventh verifies a current inspector failure remains visible after completion.
The existing tests also retain unrelated draft text, pending proposals, placement
intent, coalescing and stale/version custody.

Exact successor results: focused20/20, full UI487/82 files, typecheck0 errors /
0 warnings, lint, format, production build, traceability and conventional
precommit gates pass. Full native workspace qualification is hosted; a first
local push-hook invocation lacked cargo in its environment and rejected that
push. Explicit source checks passed and the published source's hosted native
builds/test suite below supplies the native checks. No ONNX403 bypass occurred.

## Hosted native qualification

Successful run: https://github.com/MrScripty/Eidetic/actions/runs/37554492625

- Application source: `f05fa888245db0fff1dad5a3165b0c1d3e07539c`.
- Qualification head: `e78a38c63b5b1961532c8c69b7ef7b5451ccdb57`, tree
  `2dc7cfd22ec2f2563b7697cda61b5edec79dd74d`; application source is an ancestor
  and the workflow checks an eight-file QA-only delta.
- Job: `112577527433`, success. Actual core120/server486/UI487, driver52 and
  no-download5 checks pass, with the standard locked native runtime build.
- Artifact: `11454550450`, `eidetic-selected-inspector-completion-native-f05fa88`.
  Original ZIP SHA256:
  `9718801f66d048986160cbac52ae687c1097ef15ca736c266fd5d4c12392b573`.
- Download file ID: `file_00000000706881f7af6ec5fd2bbe7849`.
- Untouched captures and metadata:
  `/workspace/scratch/inspector-completion-native-success-37554492625/`.

The actual X11/AT-SPI application performs native writing, generation, manual
screenplay saves, exact E placement, retained F draft, blue-to-amber Bible Save,
pending targeted preview and explicit B acceptance. Canonical reads verify
the saved text and placement, preview preserves all blocks/drafts, and explicit
acceptance alone replaces B and refreshes consumed field lineage. Original
raw capture-evidence.json retains all IDs, values and checks; the manifest embeds
it unchanged. The Vite frontend belongs to the checked-out qualification source;
the unchanged Rust executable hash alone does not establish frontend authority.

All eight new PNGs are byte-identical to the individually inspected originals
from exact-source predecessor run37550726323; all eight capture hashes were
reverified. Those original views visibly show the generated B Has content badge,
saved manual text, correct 180–210-second placement, Bible/timeline/screenplay
review, fully visible pending proposal/acceptance controls and retained draft.
The badge remains visually qualified, not machine-readable through AT-SPI.
The native run exercises successful inspector reads; unresolved inspector
completion behavior is established by the deterministic store regressions.

All three production HTTP/SSE provider calls are labelled synthetic,
real_model:false. No real-model narrative-quality claim is made. Standard
Pumas/ort no-download admission was preserved.

## Preserved first attempt

Run37553719957 / job112574955338 on QA head
`4b0059c27b01516e40623c364b071a2fde17ebd8` has cancelled final job status although
its native qualification step succeeded. The workflow pinned application source
f05fa888, but the driver still advertised 5ac9eb9 in its receipt and the artifact
name. The QA successor reads the exact workflow source and corrects that label;
its push cancelled the earlier run. This first attempt is not the final receipt.

Artifact11453383367 is retained with original ZIP SHA256
`f7065795519458f576ec4b16adcb381a222f6ef7b7edce5dda4f68829919b74e`, file ID
`file_00000000bd7881f6aa84919b6b4bb642`, and unaltered files under
`/workspace/scratch/inspector-completion-native-first-37553719957/`.
The final manifest seals both attempts, raw official job metadata/logs and
local verification logs. Earlier PR17 source/evidence remains untouched.
