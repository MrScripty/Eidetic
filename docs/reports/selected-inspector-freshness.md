# Selected inspector freshness on current main

The selected inspector retained B's **Notes written** caption after successful
generation and manual Save, and transiently retained E's **540–570** placement
summary after the canonical move to **180–210**. These are UI read freshness
issues. The original walkthrough independently established successful canonical
propagation, complete pending proposal visibility, explicit targeted acceptance
and unrelated F draft retention.

## Exact source

| Boundary | Commit | Tree |
| --- | --- | --- |
| Verified main | `25895e7bcf215e5a013dd7c4a98873e9b7412b8a` | `21f9432d2f14b104ab90bcfe0b4c7e87be201b63` |
| Application fix | `5ac9eb99ffb155878d5dcfda77baa01d60ef24f7` | `ba5c1d15e3ce002511190936441d7b322164ccf4` |
| Native qualification | `a8eea3490abc0bc0457357a11fa49b39f4eb9547` | `7d171bfa6faf8f3dbd04e67a20adc8c34eaeb315` |

The application fix has verified main as its direct parent and is published on
`fix/selected-inspector-generation-refresh`. The separate qualification branch
`test/selected-inspector-freshness-native` adds only eight workflow/fixture/driver
files relative to that application source. Its workflow checks this binding
before building. Original QA67b1efb and docs69a4e5b remain preserved.

## Reproduction and decision

Original native run37545170522 uses app42c3c0b, whose complete tree is byte-equivalent
to verified main25895e7. Its untouched manual-edit screenshot shows selected B's
stale caption; the placement screenshot shows E's stale summary, while later
frames show its correct committed range. No failure of saved canonical text,
proposal acceptance or draft retention is inferred from these labels.

`BeatEditor.svelte` derives the header status and `TimelinePlacementEditor.svelte`
derives the summary from `selectedNodeEditorProjectionState`. Successful
`persist_successful_generation` persists `HasContent` before publishing
`GenerationComplete` and `NodeUpdated`. Previously, those frontend handlers
refreshed timeline, screenplay and context stack but not the selected inspector.
Timeline/hierarchy events likewise omitted that read.

The baseline event/store regression fails seven of nine cases: four direct
caption/range freshness cases and three ownership cases whose required inspector
read never occurs. The original failed receipt is preserved. Tests use labelled
mock projection payloads; they are not model or GUI execution evidence.

The fix rereads the canonical selected editor through the existing coalescing
queue on matching generation/node events and on timeline/hierarchy invalidation.
A queued or in-flight event read can publish only while its original selected
node, editor session and event handler remain current. Existing request and
projection-version guards still apply; an obsolete error is also discarded.
Generation completion cannot clear a newer reopened session's streaming state.

This refresh does not patch durable fields or change selection/context request
semantics. Screenplay drafts, pending propagation proposals and placement draft
receipts keep their existing owners. Placement input and its captured revision
are not automatically rebased by a fresh displayed range.

## Local validation

- All **480 UI tests** pass, including 13 new event/store cases for refreshed
  status/range, retained screenplay draft/pending proposal/placement intent,
  unrelated node events, queue coalescing, old projection versions, navigation,
  late errors, session reopening and teardown.
- Source precommit typecheck reports **0 errors and 0 warnings**; lint, formatting,
  decision traceability and conventional commit checks pass. UI production build
  passes. An initial test-fixture undefined-array-access diagnostic was corrected
  before the source commit; its original log is retained.
- **52 native driver boundary tests** pass. The final observer requires the exact
  selected scene heading and native idle Generate control within the actual top
  inspector viewport; it checks the canonical HasContent status with a read-only
  query and requires separate visual review of the untouched caption image. It
  does not claim badge glyph verification through AT-SPI. The placement summary
  remains an exact native control check. Hidden and saved-screenplay matches are
  rejected.
- Commits use MrScripty as author and committer. Local prepush desktop checks are
  unavailable because the installed environment lacks `glib-2.0.pc`; `LEFTHOOK=0`
  skips that hook for publication. The ordinary hosted locked native route is
  required. No ONNX403 bypass, alternate runtime or new model download is used.

Sealed local log receipts are in the companion manifest. Source and qualification
worktrees are clean; only temporary dependency symlinks were removed.

## Hosted native qualification

[Run37550726323](https://github.com/MrScripty/Eidetic/actions/runs/37550726323)
/job112565288553 succeeds on qualificationa8eea349 with application5ac9eb9.
[Artifact11452044400](https://github.com/MrScripty/Eidetic/actions/runs/37550726323/artifacts/11452044400)
contains eight original1920×1440 PNGs, the raw receipt and sanitized app log.
The standard locked GTK/WebKit desktop and public-service fixture builds,
core120/server486/UI480, driver52, no-download5/feature-union, frontend0/0,
lint/format/build gates pass. The actual native walkthrough completes in about
44 seconds with zero retired-object polls. Window2097155 belongs to PID22187.

The Rust binary hash remains `4a51fac11fee64f7d1b9e926a11145afe6cb5391b66387af2f31b34012553d80`.
This UI-only change is bound by the workflow's unchanged-source check and the
checkout's freshly started Vite server, not by the unchanged Rust binary alone.
The exact fixture uses public services to seed four supporting screenplay
blocks, empty A and ungenerated B. X11/AT-SPI performs all subsequent writing,
Generate, manual Save, placement, Bible Save, preview and acceptance. The driver
makes read-only canonical queries and does not inject JS/CSS/IPC or write the DB.

All eight images have verified original hashes/dimensions and were individually
viewed. B visibly shows **Has content** in both generation and manual-edit frames.
E's first placement frame visibly shows **180–210 seconds**, both exact input
values and the retained F draft. Its Write screenplay surface is ready for E.
The summary is also an exact bounded native-control read. Content badge glyphs
were verified visually; the receipt explicitly retains
`caption_machine_verified=false`. No successful AT-SPI badge-text claim is made.

| Original capture | Concrete visible evidence |
| --- | --- |
| `eidetic-story-authoring-writing.png` | Exact new A screenplay draft and timeline before Save; Arcs is selected. |
| `eidetic-inspector-generation.png` | Selected B, green Has content badge and labelled synthetic generated screenplay. |
| `eidetic-story-authoring-manual-edit.png` | Selected B still Has content; saved human B, edited morning C and source-change review. |
| `eidetic-story-authoring-placement.png` | Selected E's180–210 summary/inputs, moved source material, B review and open unsaved F draft. |
| `eidetic-story-authoring-review.png` | Bible amber fact, saved manual B, exact E placement, F draft and Bible/source/window review causes. |
| `eidetic-story-authoring-preview.png` | Complete labelled synthetic proposal, Reject/Accept controls, saved manual B, amber fact and F draft in one frame. |
| `eidetic-story-authoring-accepted.png` | Explicit acceptance, canonical proposed B, review cleared and F draft retained. |
| `eidetic-story-authoring-retained-draft.png` | Consecutive retained-draft checkpoint, byte-identical to accepted. |

The native exact Bible edit is **Mara's umbrella is blue. → Mara's umbrella is
amber.**, revision `0c95bb2e-54d3-413a-be70-3b521b853e44`. Generation revision is
`ef836cea-ec54-4e26-808f-265680a83582`; manual B Save is
`2736641a-7db4-4fe5-b8f8-f50f4b306301`; midnight-to-morning C Save is
`9d5f1aa5-0f44-41a2-8263-6193276ab298`. E placement command
`f647314d-d184-47b6-a78c-6e2e7622a0a7` retains the original540–570 receipt and
commits event `3ad567b7-d19f-42d5-a44f-83f487d12120`.

Pending proposal `script.review.3fd7dcf4-ab36-4a1f-93a6-c305670bb7b6` leaves all
saved screenplay unchanged and the F draft open. Its full89 non-whitespace
character bounds are inside the actual Script viewport. The production HTTP/SSE
client consumes exact saved F/E/edited C/D/current human B and amber Bible;
it excludes displaced A, old midnight C and unsaved F. All three provider
responses are labelled **synthetic** with `real_model=false`.

Explicit native acceptance alone changes B to revision
`68054743-617a-414a-8dfa-c05f743dc532`, refreshes its consumed Bible revision to
the amber event, clears review and keeps the F draft and every other saved block.
F's saved revision remains `c8e32a4e-84b2-4f69-956a-9181f654895a`; the exact
unsaved draft is **Unrelated draft: Eli pockets a brass whistle.**

Five new captures (writing/review/preview/accepted/retained) are byte-identical to
the earlier37545170522 counterparts. The new generation capture matches both
preserved badge-observer failure images. Manual-edit and placement are distinct
new bytes showing the corrected feedback. Original capture bytes were never
changed. The existing fixture's E support block and its No content node badge
coexist; node-status derivation is retained, with this fix scoped to freshness.

The success ZIP has SHA256
`7904f1960845c8cada10c2cbfd637e0dd3cac5aa62f54f017b574190038d6646`
and file ID `file_000000003fb881f5ba682cc553510ef6`. It is preserved locally at
`/workspace/attachments/fdd79535-bab9-4277-b1ed-901ed822ce00/eidetic-inspector-freshness-success.zip`;
untouched extracted originals are under
`/workspace/scratch/inspector-freshness-native-success-37550726323/`.
Hosted expiration is **2026-10-10T00:22:34Z**. The companion manifest seals45 files,
full raw native receipt, production file hashes, CI metadata and visual-review
limits. All seven earlier walkthrough images still have their original hashes.

No real-model quality or new model download is claimed. Graph-memory,
revision/provenance/dependency and explicit acceptance behavior retain their
existing implementation. Source/qualification are published and clean; the
parent owns PR creation, independent review, merge and Library delivery.

## Preserved first attempt

Qualification9d064fd, run37547569746/job112555124863/artifact11452040838 passes
the standard native builds, core120/server486/UI480 and driver51/no-download5
gates, then times out looking for an individual Has content accessibility object.
Its untouched failure image visibly shows selected B's corrected green caption
with the exact synthetic generated screenplay. The exact label observer did not read
the badge. Qualificationab180802 attempts native Text substring/character bounds
with the heading and text confined to the top editor. Its separate
run37549233841/job112560485346/artifact11452910720 again passes native builds and
core120/server486/UI480/driver51/no-download5, then times out at the same badge.
Both untouched failure images show the correct selected B caption and are
byte-identical (6bd218cc...). The Text exposure explanation was an inference
that the second attempt did not establish. Final qualificationa8eea349 separates
read-only canonical status and native idle controls from visual caption review;
its receipt explicitly says caption_machine_verified=false. No application
change is made to accommodate the observer; source5ac9eb9 remains unchanged. Both failure ZIPs and all four original captures are
preserved, hash-verified and viewed. Neither attempt reached manual B/C Save, E placement,
F draft, Bible edit or preview/accept and does not qualify those actions.
