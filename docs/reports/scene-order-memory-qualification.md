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

The first hosted run [37346181993](https://github.com/MrScripty/Eidetic/actions/runs/37346181993),
qualification head `beee2d97219936bc93d7a02f8f56c5ceaee0653b`, built the actual
native runtime and passed **116 core / 396 server tests**. Its GUI setup stopped
before launch because the qualification example omitted `/v1` from its provider
URL. It produced no accepted provider calls and no screenshots. Failure artifact
11361006580 is preserved locally under
`/workspace/scratch/scene-order-native-37346181993/`.

Qualification-only correction `141ff3d1b959a6e5f4899dbdc160a490d80b5ee0`
(tree `a331c8153c5daa284676191b28a730c95f2ba842`) uses the existing configured
OpenAI `/v1` endpoint. Compile check and unchanged-source guard pass. Corrected
run [37347395869](https://github.com/MrScripty/Eidetic/actions/runs/37347395869)
also passed 116 core / 396 server tests. Its exact generation context was
accepted by the synthetic HTTP fixture, but setup stopped because the newly
created B was absent from the legacy project mirror read by generation completion.
No GUI launch or screenshots occurred. Artifact 11361382681 is preserved locally
under `/workspace/scratch/scene-order-native-37347395869/`.

The next qualification-only fixture saves and reopens the prepared project through
public services before generating, asserts B exists in the refreshed mirror, and
reports exact GenerationError events instead of a generic missing-output message.
This is normal qualification setup, not a mirror/recovery repair or a claim of
live generation immediately after canonical node creation. Application source
remains frozen at 9429daa. Final native UI results will be recorded after execution. No screenshot Library IDs are available: the previously attempted
official prepared-upload helper failed at tool-list startup before any write.
Parent handles PRs/reviews/merges and Library delivery.

### Native walkthrough successor after preserved checkpoint

Run 37348942331, source 9429daa and qualification head
`da54a856b46e7de712bf46e0ff7d660904b4b63f`, passed 116 core / 396 server tests,
prepared the actual six-scene reorder, launched Tauri and saved exact human B
through native input (revision 81b24ba1-0a82-42f2-a76d-a8f99f782d0a).
Synthetic generation and recap admitted the exact expected context. The driver
then timed out at the What changed disclosure: AT-SPI ANYWHERE left its fragmented
multicolumn rectangle behind the Bible sidebar, and native input hit that sidebar.
No pending preview or acceptance was qualified. Failure artifact 11361543736 and
unaltered screenshot are preserved at
`/workspace/scratch/scene-order-native-37348942331/`; binary SHA256
`ffe7f431c9f2f6722ca909b2fd33026e01e27b60db3f53df37362e101027a728`.

The successor uses ordinary X11 horizontal left-wheel input on a verified visible
saved Script block, then reacquires the disclosure without ANYWHERE scrolling.
This repairs the actual walkthrough interaction, without altering app source,
UI layout or screenshot pixels. Final preview/acceptance qualification is pending.
