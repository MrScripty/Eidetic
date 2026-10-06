# Immediate canonical scene generation qualification

Application source **9e8bd1c9d51250ac1c517945269278b7fe7e3d61**, tree
**594a474ec14eb5d23353805287469558528dab6c**, on `feat/canonical-scene-generation`.
Frozen predecessor `441c2a29af6199b53b92f718df43d0ca916b1ce5` and all older
application/native evidence remain preserved. Parent owns PRs, review, merges and
Library delivery.

**Result:** immediate canonical creation → selection → exact notes/manual anchor →
saved generation without reopening is qualified in the actual native application.
Service tests qualify delayed stale-target rollback and human preservation. The
final native refusal-banner/history gate remains blocked by a GitHub-hosted
runner failure; do not describe the whole walkthrough as passed.

## Cause and implementation

Canonical creation writes SQLite and publishes projections without inserting new
nodes into the legacy project mirror. Admission already reads SQLite, but
successful_generation_metadata required the mirror node and used its range;
completion also marked HasContent before script persistence. Prior native run
37347395869 admitted the exact new-scene request but persisted no generated output.
Save/reopen masked the split. Cause and criteria were reported before implementation.

Completion now reads canonical metadata. Existing request/generation command JSON
carries optional ScriptGenerationTarget receipts using timeline/segment/output
revision IDs, exact admitted notes and placement. Writer-transaction validation
refuses deleted/retimed/locked/changed/ABA targets and changed/manual output.
HasContent commits with screenplay and lineage. Failure restores status from saved
canonical blocks without demoting existing manual screenplay. Existing session
identity/gate and history/replay guards apply. Legacy receipts remain absent; there
is no new schema, dependency, memory owner or mirror-refresh mechanism.

The existing validated create-child API had no editor control; timeline double-click
sends a parentless Scene rejected by core. Add Scene in the selected Sequence editor
reuses that API's backend-derived parent/level/range and selects the acknowledged
child. Editor session, current selection and mounted lifetime guard delayed
acknowledgements. Generation admission refusals clear streaming state and surface
through the existing error flow. Manually edited generated output refuses direct
regeneration and requires the existing reviewed-update route before replacement.
Existing manual edit, preview/accept, provenance and draft mechanisms are preserved.

## Tests

| Gate | Evidence |
| --- | --- |
| Actual hosted core | 117 passed in each of runs 37359633825, 37360892289 and 37362942784 |
| Actual hosted server | 406 passed in each run, including four new public AppState/paused production HTTP tests |
| Local production-module harness | 234 passed; imports real modules, not copied implementations; no full native execution claim |
| Frontend | 405 passed, including delayed child acknowledgement, selection change, retired same-ID session and unmounted editor |
| Qualification helpers | 12 passed; exact serialized wire identity/order/revision/text, swapped/extra/duplicate/obsolete refusal and paused response |
| Other checks | Strict all-target server Clippy, Rust format, frontend lint/format/typecheck/build and decision traceability pass |

The four hosted public-service tests are in
`crates/server/src/canonical_generation_service_tests.rs`:

- create/select/generate without reopening with the new node deliberately absent
  from the mirror; exact canonical metadata and manual anchors survive;
- notes edit/restore ABA during paused HTTP refuses persistence without extra history;
- retime, lock and deletion during paused HTTP refuse stale placement;
- a saved human output edit during delayed regeneration survives; canonical preview
  still reads it, and subsequent direct generation refuses before provider execution.

Six additional writer-boundary tests in `script_generation_target_tests.rs` cover
atomic status/lineage/history, manual append, output/notes/range/lock ABA, deletion
and forged target/placement. Optional receipt/legacy serialization passes core tests.

Local GTK/GLib headers remain unavailable. ORT_SKIP_DOWNLOAD is used only for
compile-only local server checks. Hosted native execution uses standard Ubuntu24.04,
locked Pumas/ORT and normal downloads. Local ONNX403 is not bypassed.

## Inspected native evidence

[Run 37362942784](https://github.com/MrScripty/Eidetic/actions/runs/37362942784),
job **111941610253**, qualification **c383601bd8ac3c78fb112f2c0610006823ac1cef**,
tree **df17b6e27578e2d68f879ecf631b0c0f1eb273da**, passes the unchanged application
source guard and 117 core / 406 server tests. Actual Tauri binary SHA256:
`cd9bbf7c4e2de24092283a0893df798ec70f7f51c90872e8610210c4b6af5197`.

Public setup services prepare only existing A, Sequence and Bible before launch.
The real GUI opens that project once, connects the labelled localhost synthetic
provider, selects Sequence and invokes Add Scene. Native input automatically selects
the new canonical child, types and saves exact notes/manual screenplay, generates
immediately, then manually edits and saves output during a paused second request.
AT-SPI native geometry/scroll and X11 input are used; there is no injected JS/IPC,
SQLite write during the UI walkthrough, reopened project or edited screenshot.

| Canonical evidence | Exact value |
| --- | --- |
| New scene | `536678b8-4c3b-4ba6-b6a5-d05353ef0127` |
| Selected parent | `1ea0d387-8a02-4739-8b04-0236f9e3a393` |
| Parent-derived range/name | `0..600000`, `New Scene` |
| Exact notes | `Exact new scene notes: Mara waits beside the station.` followed by LF |
| Notes/timeline revision | `04a5e720-5239-4f17-a359-1f365e6c2f6b` |
| Manual B anchor | `Exact manual new-scene anchor B.` followed by two LFs |
| Manual anchor/segment revision | `75a3e417-0b84-47c2-b1b0-79d62b3194b0` |
| Generated revision | `8d8633a2-e293-4753-bc04-a06417291a39` |
| Exact existing A | `Exact existing authored scene A.` followed by two LFs; unchanged |
| Exact generated text | `Synthetic immediate output: Mara waits beside the station.` followed by two LFs |
| Exact saved human replacement | `Exact human replacement saved during delayed generation.` followed by two LFs |

The persisted generation receipt binds the exact node/range/notes and anchor segment
revision; output revision is absent at initial admission. Exact manual A and B
remain unchanged. The strict fixture admits exactly generation1 → recap →
generation2 through the actual production HTTP/SSE client. It compares the entire
serialized canonical section including ordered block identity, revision IDs and text:
initial `[manual B, manual A]`, regeneration `[generated B, manual B, manual A]`.
It rejects swapped/extra/duplicate/obsolete content. All three native requests pass
these comparisons. Responses are **synthetic**; real-model quality is **not qualified**.

After the delayed response resumes, the runtime logs:
`invalid value: generation target changed; saved screenplay was preserved, generate again from current context`.
The later failure capture visibly retains the saved human replacement. The driver
then times out while locating the refusal banner. The failed checkpoint did not
record the human revision or reach its final native history-count assertion;
service tests cover those rollback guarantees, but this run does not qualify them
through the complete GUI route.

All captures are unaltered **prototype views**, visually inspected with Bible,
timeline and screenplay together:

| Capture | Visible proof | SHA256 |
| --- | --- | --- |
| `eidetic-canonical-scene-created.png` | Exact notes, selected new scene, blue Bible field, manual B and A | `54829118ab8b3f15be9fc07da4bab90ad1787eae3173412ec10dac90f4bf23ad` |
| `eidetic-canonical-scene-generated.png` | Readable saved synthetic output, exact manual B and A, selected timeline scene, Bible | `2e51931274e49a1af5aee23e8d8f31d9814f1209e62d52978fb4b07b836a6256` |
| `eidetic-canonical-scene-failure.png` | Readable saved human replacement and retained B/A; no readable refusal banner | `27792d818ae72e37c13319c12dadb6e87566eb5d8253016b8d3406dbd9448ce3` |

Visual limits: the generated view still labels the selected editor Notes written
while saved output is readable. Selected-header freshness is not qualified here.
The failure capture does not establish whether the error is offscreen or absent.
No offscreen proposal/banner or real-model quality claim is made.

Artifact **11368330401**, 269925 bytes, expires **2026-10-08 19:40:53 UTC**.
ZIP SHA256 `8a5162defdb6c0852bc4c0186aa619cc09c7983f2de887d3128d4610313b39b8`.
Local preserved PNGs, capture-evidence.json and sanitized app log:
`/workspace/scratch/canonical-scene-native-37362942784/`.
Evidence SHA256 `e350ab147777ac74e2fcfb23e552ce65e77d4a9a146ad5571ad8035d52917e22`;
log SHA256 `d9b1d203d8565165625128e3e03e7a9501b9ea8ad6c8af0f777e3d549cdaa38f`.

## Remaining native gate and external blocker

Qualification-only successor **776d16c6e1039641c00beba9dade31465c9f28fe**, tree
**59c944b8bd3ba83c15c10be170cb049de0ce810e**, leaves application source unchanged.
It targets the exact production error banner with standard native top-edge scrolling
instead of a broad descendant-text match, records the saved human revision before
banner discovery, and collects focused accessibility evidence on failure. Whether
this resolves the locator failure or exposes a UI error-publication gap is pending.

[Run 37365296113](https://github.com/MrScripty/Eidetic/actions/runs/37365296113),
job **111948855213**, ends at **2026-10-05 19:59:04 UTC** without a runner:
workflow conclusion failure, job conclusion cancelled, runner ID 0, zero steps and
zero artifacts. Its public annotations say the hosted runner was not acquired after
multiple attempts and report an internal server error (correlation
`903f4ee5-6ada-417b-8ec9-f4fc8b514fc9`). No application test or UI step executed.
This is an infrastructure failure, not evidence about the banner correction.

GitHub's official Actions incident began 19:11 UTC; its 19:50 UTC update confirms
delays assigning hosted runners across configurations.
[Official incident API](https://www.githubstatus.com/api/v2/incidents/unresolved.json).
Retry this existing failed hosted job after runner service recovers, then inspect
actual banner/accessibility and canonical history before classifying any UI defect.
No duplicate retry is started during the incident. No credential changes, paid
service, external reviewer request or local ONNX workaround is used.

## Preserved earlier attempts

| Run / qualifier | Actual outcome | Artifact / ZIP SHA256 |
| --- | --- | --- |
| 37359633825 / `624893ed749c32e8993a799430c4c0ba7d4997a3` | 117 core / 406 server pass; setup reads wrong Save response field before GUI launch. Separate fixture fix uses canonical database active path. | 11365819243 / `98609504d20243e3690ff2fc792350f3386668851dcc56d8cc6f79cbdbcf4d7b` |
| 37360892289 / `bd2c6adab434f09979e946f634d371b93229ed6f` | 117 core / 406 server pass; native Add Scene creates/selects target. Driver searches Notes while inspected editable label is NOTES. Separate case-folded/native scrolling fix. No typing/provider calls. | 11366668121 / `d77b7c506f43e9c49fecd70e505b214fe80851b01ef2ee3b47de3d24425ab18f` |

Local evidence remains in `/workspace/scratch/canonical-scene-native-<run>/`.
Frozen predecessor and prior Bible/scene-order milestones remain intact. The older
scene-order report's membership-only wire claim is narrowed in this successor;
its immutable source/artifacts are unchanged. Existing Bible edit → targeted pending
preview → explicit acceptance is documented separately in the Bible qualification.
Project-switch recovery, embeddings and broader feature-length authoring remain
outside this slice. No PR, merge or Library delivery is claimed.
