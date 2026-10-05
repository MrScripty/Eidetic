# Scene-order screenplay memory qualification

Application source: `9429daa53de9d7ce6679f0b166b1f736f332892a`, tree
`17b166b08d10e0c3a07047eb289c2a5f576f7de2`. Branch
`feat/scene-order-story-memory`, based on frozen Bible head
`af603e7682417fd49dbaac02e75142fd1b9d0d61`. The Bible branch is unchanged.

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
Acceptance refreshes actual input lineage. Old absent scope is not backfilled.

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

## Hosted native qualification gate

Dedicated push-only branch `test/scene-order-native` and workflow
`.github/workflows/scene-order-native.yml` use the existing standard Ubuntu 24.04
GTK/Webkit/Xvfb and locked Pumas/ORT build route. They are independent of frozen
Bible and previous authoring workers. The source guard permits only qualification
and documentation changes relative to the exact application checkpoint above.

The public fixture creates six proper hierarchy scenes and exact authored blocks,
generates B through the production client, then moves E from 540000–570000 to
180000–210000. Order changes from A,F,B,C,D,E to A,F,E,B,C,D. B's complete window
changes from A,F,C,D (before its own generation) to F,E,B,C,D; A leaves and E
enters. Fixture generation and recap responses are explicitly synthetic.

The real Tauri GUI then manually saves exact B text
`Manual B: retain this station beat.\n\n`, shows the entering/leaving cause with
Bible and timeline present, previews from E/F/C/D plus exact human B, and requires
explicit Accept update. Read-only SQLite checkpoints verify unrelated blocks
remain exact. Normal native scrolling reveals the accepted canonical target.
Screenshots are unaltered **prototype views**. No GUI drag/reorder or real-model
quality is claimed: reorder setup uses the public native range service before
GUI launch; manual edit/preview/acceptance use native GUI input.

Hosted run, artifact, screenshots, exact revisions and hashes will be recorded
after execution. No screenshot Library IDs are available: the previously attempted
official prepared-upload helper failed at tool-list startup before any write.
Parent handles PRs/reviews/merges and Library delivery.
