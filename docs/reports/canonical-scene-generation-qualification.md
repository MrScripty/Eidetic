# Immediate canonical scene generation qualification

Application source `9e8bd1c9d51250ac1c517945269278b7fe7e3d61`, tree
`594a474ec14eb5d23353805287469558528dab6c`, on feat/canonical-scene-generation.
Frozen predecessor `441c2a29af6199b53b92f718df43d0ca916b1ce5` and all prior native
artifacts remain preserved. Parent owns PR/review/merge and Library delivery.

## Cause, criteria and implementation

Canonical timeline creation records SQLite and publishes projections without
inserting new nodes into the legacy project mirror. Admission already reads SQLite,
but successful_generation_metadata required the mirror node and used its range;
completion also marked HasContent before script persistence. Prior native run
37347395869 admitted the exact new-scene request but persisted no generated output.
Normal save/reopen masked the split. Cause and criteria were reported before changes.

Completion now reads canonical metadata. Existing request/generation command JSON
carries optional ScriptGenerationTarget receipts using timeline/segment/output
revision IDs, exact admitted notes and placement. Writer-transaction validation
refuses deleted/retimed/locked/changed/ABA targets and changed/manual output.
HasContent commits with screenplay and lineage. Failure restores content status
from saved canonical blocks without demoting existing manual screenplay. Existing
session identity/gate and history/replay guards apply; no new schema, dependencies,
parallel memory owner or mirror-refresh mechanism.

The existing validated create-child API had no editor control; timeline double-click
sends a parentless Scene rejected by core. Add Scene in the selected Sequence editor
reuses that API's backend-derived parent/level/range and selects only the acknowledged
child. Existing editor session, current selection and mounted lifetime guard delayed
acknowledgements. Generation admission refusals are visible and clear streaming state.

Acceptance: immediate public/UI creation → canonical selection/notes/manual anchors
→ generation without reopening; exact unrelated and target manual blocks preserved;
delayed node/notes/placement/lock/deletion or human output cannot be overwritten;
existing revision/ABA, draft and targeted review mechanisms remain supported.

## Local gates

- 117 core tests, including optional target receipt/legacy history serialization.
- 234 tests importing actual production modules: six new target boundary tests
  alongside existing screenplay/Bible/context/timeline/history regressions. No
  copied implementation or full native server execution is claimed by this harness.
- 405 frontend tests, including selected-parent command acknowledgement, changed
  selection, same-ID retired session and unmounted editor ownership.
- Strict server all-target Clippy and compile-only public-service tests/example;
  Rust formatting, frontend lint/format/typecheck/build and traceability pass.
- 12 qualification helper tests pass, including exact wire order/identity/revision,
  stale duplicate refusal, acceptance slots and paused second HTTP response. Native
  UI is not executed by these tests.
- Four public AppState/paused production HTTP tests are ready for hosted execution:
  immediate create/select/generate with truly absent mirrored nodes; notes ABA;
  retime/lock/delete; and a saved human edit during delayed regeneration plus
  still-readable canonical context. No provider credentials or real model used.

Local GTK/GLib headers remain unavailable. Compile-only server checks use
ORT_SKIP_DOWNLOAD; actual native execution must use the standard hosted locked
Pumas/ORT route. ONNX403 is never bypassed. No native success is claimed yet.

## Native qualification gate: pending execution

Dedicated test/canonical-scene-native workflow uses standard Ubuntu24.04 native
build/test and real Tauri AT-SPI geometry/scroll/X11 input. Its unchanged-source
guard binds application behavior to the exact checkpoint above. Setup prepares
only existing A, Sequence and Bible; GUI Add Scene creates and selects the target.
The same native project is opened once before creation, with no reopening afterward.
Exact notes/manual B anchor are saved, Generate persists output, then a second HTTP
response pauses while native input saves an exact human replacement. Resumed output
must refuse replacement, preserve history and show the human canonical target.

HTTP responses are explicitly synthetic. The new fixture compares the complete
serialized canonical screenplay section including ordered block identities, text
and revision IDs; it rejects swapped/extra/duplicate/obsolete text and permits only
one initial generation, one recap and one delayed regeneration. Fixture tests cover
those wire guarantees. They do not execute the desktop or qualify real-model quality.
The predecessor report now narrows its older membership-only wire claim without
changing its frozen source or evidence. No offscreen proposal screenshot claim.

Native screenshots will be unaltered prototype views, inspected after execution.
Source/run/artifact IDs, revision IDs, hashes and actual visual limits will be recorded
only after the hosted run completes. No Library IDs or uploads are claimed here.


### Preserved first hosted attempt

Run 37359633825 / job 111930684321 on qualification
624893ed749c32e8993a799430c4c0ba7d4997a3 passed the unchanged application guard,
117 actual core / 406 actual server tests (including all new paused production HTTP
service tests), then stopped before GUI launch. Qualification setup incorrectly
read save response field path; the public response contains saved. No provider
calls or screenshots occurred. Artifact 11365819243 and its evidence/log remain
preserved under /workspace/scratch/canonical-scene-native-37359633825/.
The separate qualification-only successor reports the existing database owner's
active path. Application source remains exactly 9e8bd1c; GUI execution is pending.
