# Scene-order screenplay memory qualification

Application source: `9429daa53de9d7ce6679f0b166b1f736f332892a`, tree
`17b166b08d10e0c3a07047eb289c2a5f576f7de2`. Branch
`feat/scene-order-story-memory`, based on frozen Bible head
`af603e7682417fd49dbaac02e75142fd1b9d0d61`. Merged main
`302851dbb5bf67cda922b4d79f70623e444891dc` has the same Bible tree
`becb86624608663a9e91ff629edd1080d6eada54`. Ancestry-only reconciliation
`5b057efdf038a5959630270dea5c749075307844` changed no application files.
Original application 9429daa and qualification checkpoint da54a85 remain ancestors.

## Demonstrated gap and bounded implementation

The actual-source reproduction passed on the original source: E at 9000 was not
consumed by generated B at 4000. Moving E to 3000 entered B's canonical context,
and moving B to 8500 removed A/E, yet no old consumed revision changed and B had
no review cause. Existing two-scene consumed placement/reorder already works.

New generation captures a complete ordered continuity-window receipt alongside
exact consumed screenplay blocks. Existing command/proposal JSON and semantic
dependencies retain it; there is no added database schema, vector dependency or
parallel memory store. Derived review identifies entering/displaced members or
changed external order/relative position and preserves manual text. Preview uses
the fresh window; replacement requires existing explicit targeted acceptance.
Acceptance refreshes actual input lineage. Old absent scope is not backfilled. Pre-upgrade pending previews may require
a fresh preview; their saved text and proposals remain intact.

The append-only main-document segment epoch guards pending window equality,
including an unconsumed move-in/move-out ABA. An unrelated segment revision may
conservatively require a fresh preview; unchanged selection alone creates no
review cause. Placement remains presentation time, never fictional story time.
The existing timeline UI's drag bounds and semantic extraction are separate gaps.

## Local validation

- 116 actual core tests pass, including legacy scope serde/replay compatibility.
- 215 actual-source module tests pass: 208 existing tests, six new boundary
  regressions and the preserved legacy gap reproduction. This harness imports
  production modules without source copies; it is not full native server execution.
- 401 frontend tests pass, including the existing draft/stale/review suite and
  the context cause/exact entered-left explanation through actual components.
- 10 stdlib qualification helper tests pass. Synthetic HTTP admission refuses
  obsolete/displaced A, missing entered E, old generated B instead of saved human
  B, duplicate accepted previews and incomplete/wrong request shapes.
- Strict all-target server Clippy, Rust formatting, frontend lint/format/typecheck
  and production build pass. Local server Clippy/check are compilation only with
  ORT_SKIP_DOWNLOAD; no native execution or alternate ONNX download is claimed.
- Pre-push native tests require unavailable local GTK/GLib headers. Source is
  published with only that known pre-push test excluded; hooks remain installed.
  Post-commit traceability found two adjacent README omissions, corrected in the
  separate qualification milestone. Hosted Ubuntu executes native tests normally.

## Passed hosted native qualification

[Run 37354081654](https://github.com/MrScripty/Eidetic/actions/runs/37354081654),
job **111911938710**, passed on qualification commit
`4fcd79e1bc5b0c4fbd52d66247fe37463dc513d7`, tree
`1803c3eaa376a641a00d770263d1a14e86917242`. The unchanged-source guard binds
application behavior to 9429daa above. Actual runtime tests pass: **116 core and
396 server**, including the public AppState complete-window range/preview/acceptance
regression. The Tauri walkthrough completed in 25.179 seconds.

Dedicated branch `test/scene-order-native` and workflow
`.github/workflows/scene-order-native.yml` use standard Ubuntu 24.04
GTK/Webkit/Xvfb and the locked Pumas/ORT build route. No local ONNX download
bypass or alternative runtime was used. Native executable SHA256:
`ffe7f431c9f2f6722ca909b2fd33026e01e27b60db3f53df37362e101027a728`.

Public native services create six proper hierarchy scenes and exact authored
blocks, generate B, then move E from 540000–570000 to 180000–210000.
Order changes A,F,B,C,D,E → A,F,E,B,C,D. B's captured external window A,F,C,D
changes to F,E,C,D, with its own saved B included in the fresh preview context.
The driver saves/reopens the prepared project before generating to refresh the
existing legacy mirror. This setup does not qualify generation immediately after
creating a scene without reopening, nor a timeline drag/reorder through the GUI.

The real GUI uses native AT-SPI geometry and X11 input, including the existing
Resize panels splitter's keyboard control and ordinary horizontal scrolling.
Bible, screenplay and timeline are visible together. It verifies:

1. Exact manual B save: `Manual B: retain this station beat.\n\n`, revision
   `35e94dce-cac0-4263-b772-580ebc15a3d0`.
2. Visible Needs review and disclosed `Entered: SCENE E. Left: SCENE A.`.
3. Pending proposal `script.review.ddba91d9-49a3-4861-801e-7c6ca8d496d6`, exact
   text `Synthetic preview: B follows the newly preceding E.\n\n`.
   Read-only SQLite confirms saved B remains exact until acceptance. The actual
   accessibility tree exposes the exact proposed text before native acceptance.
4. Explicit Accept update changes only B, with revision
   `bd0ccf97-9bfa-4ce1-b002-f7fa29fe5dd8`. All other authored blocks remain exact;
   the old review clears, and normal scrolling reveals the saved canonical B.

Generation, recap and preview each passed the strict **synthetic localhost
HTTP/SSE fixture through the production client**. Preview admission requires
fresh F/E/C/D and exact human B, excludes displaced A and refuses obsolete generated
B. These checks qualify context custody and application behavior, not real-model
quality. No real model was executed.

### Inspected native screenshots and evidence

All three PNGs are unaltered **prototype views**, visually inspected after download.
Review shows the exact entering/leaving explanation, Mara's blue Bible fact and
timeline. The preview-state PNG leaves the proposal controls below the viewport;
it is not visual proof of readable proposal text or saved manual B. Pending state
and preservation are instead supported by the native UI and SQLite assertions
above. Accepted PNG visibly shows the accepted notice and exact canonical B,
with Bible and timeline still present. No layout or screenshot pixels were changed.

Artifact **11363753551**, `eidetic-scene-order-native-9429daa`, is 264854 bytes,
expires **2026-10-08 18:20:17 UTC**. ZIP SHA256:
`c2c7c21fafe30284dbb88b39ba8001b8efc7825be19b0861b26ad4876344c15a`.
Downloaded files are preserved at
`/workspace/scratch/scene-order-native-37354081654/`.

| File | SHA256 |
| --- | --- |
| eidetic-scene-order-review.png | 9c0e314a2828a7c7c3473d1f3c7f6716eb7a54fe5b097cc9e80661bce5f66b63 |
| eidetic-scene-order-preview.png | 7d7e3080c004b4daa73586cdaac0a11627c931b36054fe347039285b14c6305f |
| eidetic-scene-order-accepted.png | d3fad50ea8c29d791db099771c5e9533e276ca00d08081db6b1ee8d2ac36d0ed |
| capture-evidence.json | 69a3143e63c4938a81ab85556b0446901bf2a34c30f45413868f0fceec08089a |
| app-sanitized.log | 8bbab417cee51a2efe04a8579be25506b968b9de6bf0f2b27ee6c0dbce678109 |

Parent handles PRs, reviews, merges and Library delivery. No Library IDs are
claimed: the earlier official prepared-upload helper failed during tool-list
startup before any write. There were no external reviewer requests, credential
changes or paid services.

### Preserved qualification attempts

Each attempt passed the same 116 core / 396 server tests. Application source never
changed after 9429daa; fixes were limited to qualification setup/navigation.
Failure artifacts and unaltered captures remain in matching local scratch folders.

| Run | Qualification | Actual failure and separate correction |
| --- | --- | --- |
| 37346181993 | beee2d97219936bc93d7a02f8f56c5ceaee0653b | Fixture omitted /v1; no accepted HTTP request or GUI launch. Artifact 11361006580. Corrected in 141ff3d. |
| 37347395869 | 141ff3d1b959a6e5f4899dbdc160a490d80b5ee0 | Exact generation context admitted, newly created B not persisted. Canonical creation/legacy mirror mismatch identified in completion source. No GUI launch. Artifact 11361382681. Save/reopen fixture correction in da54a85. |
| 37348942331 | da54a856b46e7de712bf46e0ff7d660904b4b63f | Exact native B save succeeded; fragmented disclosure rectangle remained behind Bible sidebar. No preview/acceptance qualified. Artifact 11361543736. Native left-wheel correction in 6711d1e. |
| 37351241936 | 6711d1e797b7c2709862b1d3398af2282669c74c | Inspected PNG proved scrolling returned pane to start; B remained farther along narrow columns. Artifact 11362729031. Existing keyboard splitter control correction in 4fcd79e. |

The running 37348942331 walkthrough was resumed rather than duplicated. Successors
started only after the preceding run completed. The main ancestry merge was pushed
only to the feature branch and did not restart or cancel the active qualification.

## Next concrete authoring priority, reported before implementation

**Generate a screenplay for a newly created canonical scene without save/reopen.**
This follows the plan's SQLite authority/backend ownership decision and shared
manual/agent timeline editing plus conversational authoring milestones M4/M5.
Evidence is distinct from deferred project-switch recovery:

- `command_service_timeline.rs:create_timeline_node_at_admission` records creation
  in SQLite, sends EnsureNode and publishes TimelineChanged/HierarchyChanged,
  without inserting that node into the legacy `state.project` timeline.
- AI admission uses `ai_service::active_sqlite_project`, so the actual hosted
  37347395869 synthetic provider accepted the newly created B's exact context.
- `ai_generation_runtime.rs:successful_generation_metadata` still calls
  `state.project.timeline.node_mut`. A missing mirrored node emits `node not found`
  and returns before generated-script persistence. This source path explains the
  observed admitted-but-unpersisted output; the artifact itself records setup
  failure, not that error string. Public save/reopen made the same frozen source
  qualify successfully.

The next bounded slice should reproduce this through public services, then bind
completion metadata to canonical node existence/placement under existing session,
command/history and stale-result guards. Check create-then-generate without reopening,
retime/delete during generation, locks and intervening human text before making
replacement claims. Reuse existing mechanisms; no duplicate mirror synchronization
store, new dependencies or broad project recovery rewrite. **No implementation of
this next slice is included in the present checkpoint.**
