# Last Train: native writing and story-memory walkthrough

Completed native working demonstration. The original pending capture visibly
shows the full synthetic proposal and decision controls alongside Bible, timeline
and the unrelated open draft. Explicit acceptance commits only B and preserves
that draft. No application-source change was needed.

## Frozen source

Frozen reviewed feature source is `42c3c0b94fdc18146b3e9ef128acb36de4399d67`, tree
`21f9432d2f14b104ab90bcfe0b4c7e87be201b63`, on `feat/timeline-story-memory`. Verified merged main
`25895e7bcf215e5a013dd7c4a98873e9b7412b8a` has the same complete tree, with
parents `74fd438f3ab4cf40734e094b4b2a62fc67d58b07` and the frozen feature head.
Parent owns postmerge CI37544288693. The demonstration is not restarted or
rebased; its application bytes also match merged main.
Its application implementation is `8c1956f7afa18a2a3ea3a1732fb8eca58a1129b3`.

Separate QA source `67b1efb116ccae29e86bc201513e315cb430ee01`, tree
`fc21be6d0e9fb185512d459d93996ed0f4995e05`, descends from the reviewed head
through initial QA `f86d306d5e67fdd8a7a8aa65955d48a1c0437632`. All application and dependency blobs are unchanged. Its eight added paths
are the dedicated workflow/fixture/driver/tests and the four frozen predecessor
QA helpers from `443ddc2635a24405679e0e74e130f84d24e09270`. The native workflow
checks this scope before building. QA and evidence branches are separate from
main and the reviewed feature; neither feature branch nor PR is mutated.

## Exact authoring scenario

Public services create a fresh **Last Train Authoring QA** project, its six-scene
timeline, one blue umbrella Bible fact, and four supporting screenplay blocks.
A starts empty and B ungenerated. The native application performs these actions:

1. Select A, choose **Write screenplay**, and type/save an original cafe scene
   ending with `MARA\nKeep the last train for us.\n\n`.
2. Connect the labelled localhost synthetic HTTP/SSE fixture and click **Generate**
   on B. The provider requires the exact saved A/F/C/D context and blue fact.
3. Save exact manual B: `EXT. STATION - NIGHT\n\nMara waits beside Eli. Preserve this station beat.\n\n`.
4. Edit C from the midnight departure board to
   `INT. TICKET OFFICE - DAWN\n\nThe departure board reads morning.\n\n`, then save.
5. Leave an unrelated F edit open:
   `Unrelated draft: Eli pockets a brass whistle.\n\n`. Its saved text remains
   `EXT. PLATFORM - NIGHT\n\nEli keeps the gate open.\n\n`.
6. Select E, type **180** and **210** seconds and choose **Apply placement**,
   changing A,F,B,C,D,E to A,F,E,B,C,D.
7. In Bible, type/save `Mara's umbrella is amber.` over the blue fact. Inspect
   B's manual-source, Bible and entering-E/displaced-A review causes.
8. Preview B's targeted update. Before explicit acceptance, all saved screenplay
   remains unchanged and F's draft stays open. The provider requires F/E/edited-C/D,
   saved manual B and amber fact; it excludes displaced A, old C and unsaved F.
9. Inspect the complete visible paragraph:
   `Synthetic preview: After finding her ticket, Mara arrives with her amber umbrella for the morning train.\n\n`.
   Click **Accept update**; verify only B changes, old review clears and F's exact
   editable draft remains open with its saved revision unchanged.

## Gates and evidence limits

Local qualification-driver **50 tests**, locked fixture check and actual public
fixture execution pass. Execution verified four supporting blocks and empty
A/B, without GUI or model-quality claims. Rust formatting, frontend formatting,
lint and typecheck (zero errors/warnings), traceability and commit-message gates
pass. The workstation lacks desktop GTK development dependencies; normal hosted
GTK/WebKit qualification provides the native route. No ONNX download bypass,
model, credential, schema or application-code change is introduced.

[Hosted run 37545170522](https://github.com/MrScripty/Eidetic/actions/runs/37545170522)
passed at exact QA67b1efb, job112547329212, artifact11450083908. Hosted core120/
server486/UI467, driver50, no-download5, typecheck0/0/lint/format/build and locked
feature-union checks passed, followed by normal desktop/fixture builds and the
complete native walkthrough. The app binary SHA256
`4a51fac11fee64f7d1b9e926a11145afe6cb5391b66387af2f31b34012553d80`
also matches the earlier frozen application qualification.

The earlier short Script pane capture exposed a qualification gap: native
accessibility `STATE_SHOWING` did not prove proposed text was inside the visible
pane. This driver uses normal window resizing and native scrolling, then requires
all non-whitespace paragraph character bounds and pending decision controls inside
the actual Script viewport. All seven original 1920x1440 PNGs were hash verified and viewed after download.
No screenshot was edited, cropped or replaced.
There is no CSS/JS/IPC injection, screenshot editing or database write by the GUI
driver. Public setup and real native typed actions are distinguished.

All provider responses are explicitly synthetic, with `real_model: false` in
request receipts and “Synthetic” in generated/proposed text. This demonstrates
application context custody and authoring behavior, not real-model quality.
Parent owns PR/review/merge and Library delivery. No new feature PR is requested.

## Preserved first attempt and bounded QA correction

Initial QA `f86d306d` / run37541801584 / job112536318347 passed normal native
builds, core120/server486/UI467, no-download5 and driver48. Its native session
typed/saved the exact original A, clicked Generate for B through production HTTP,
and saved manual B. It then timed out looking for C's raw multiline block.
The untouched failure capture visibly contains C's heading and action paragraph.
`ScriptView` renders those as separate text nodes; the predecessor helper matches
an excerpt in one descendant, so a full raw multiline substring cannot match.
This is a QA lookup error, not evidence that C was absent or its edit was refused.

Correction960da719 changes only the QA driver/tests: use the distinctive rendered
action line to locate formatted blocks, retaining exact raw textarea and SQLite
checks for all typed/saved bytes. A regression reproduces the separate heading/
action boundary. No application change or blind rerun is used. The failed run
qualifies only the completed writing/generation/manual-B subset, not downstream
review, preview, retained F draft or acceptance.

Preserved first artifact11449617605, ZIP SHA256
`069b4d2bfe3003712af194c5fcaa9782d6c46895a801023d9d5dcc6b0e7be1a5`,
is extracted untouched at `/workspace/scratch/story-authoring-native-failed-37541801584/`.

## Observed writing feedback issue

The preserved first native failure image shows selected B's inspector caption
**Notes written** after successful generation and exact manual Save, while its
canonical screenplay is visible and its timeline marker is green. The generated
block's saved revision is recorded in that attempt's receipt. This is a concrete
feedback mismatch, not evidence of lost screenplay. `BeatEditor.svelte` displays
the cached selected-node projection; `serverEventHandlers.ts` refreshes timeline,
screenplay and context on `generation_complete`/`node_updated`, but does not read
the selected inspector again. The completion runtime sets `HasContent` and emits
those events. This identifies a narrow selected-inspector refresh follow-up;
reselection recovery is not separately claimed as native evidence here.

No application fix is applied during independent feature review. The bounded QA
lookup correction remains distinct from this observed feedback issue, and the
visible pending proposal qualification is now complete; this feedback issue
remains a documented separate follow-up.

## Preserved second attempt: visible source review already worked

QA960da719 / run37543498566 / job112541855381 passed all build/core120/
server486/UI467/no-download5/driver49 gates. Native C edit saved exact DAWN and
morning text at `24b72885-18d8-44a3-8005-dd5f563bf386`; saved manual B remained
at `48078671-ef27-43f4-bb9e-17a1fc2e69d9`. Its untouched failure capture visibly
shows B's **Needs review**, **Source screenplay text changed.**, and the consumed
old midnight excerpt. The QA lookup timed out; the application did display review.

The predecessor scrolling helper stops at the first matching accessibility object,
including a hidden select option. Correction67b1efb changes only QA driver/tests:
read a matching visible object bounded inside the real Script pane and record its
name/text/role/bounds. Its regression places a hidden option before visible cause
text. All exact typed/saved/context, pending and acceptance checks remain required.
The second failed attempt qualifies its completed manual-edit/review subset only;
it never attempted E placement, amber Bible Save, F's draft or preview/acceptance.

Preserved artifact11450810081, ZIP SHA256
`9d1c9d55991d41a6d0b2db1d794c9a66e300fd155e0c49de913d9ccdced71a00`,
is extracted untouched at `/workspace/scratch/story-authoring-native-failed-37543498566/`.

## Successful native receipts and original views

Actual A Save revision: `5d08e398-4b16-4cde-8e92-08a5e3195167`.
Actual generated B revision: `dfb2ea0d-645f-4c92-9df5-98ef95c56cf6`.
Manual B revision: `c4d154a8-18d5-44dc-b3bb-15b03b05aeb8`.
Manual morning C revision: `bace3325-573e-4432-88ca-cec51dd3c038`.
Guarded E placement command: `6957b911-7faf-4f8a-b711-d1c74f366548`, event
`b3bc29dd-b0a2-4bea-96cc-18203c477fed`, with old 540000–570000ms and node
receipt `c00f7c53-297c-4bea-87e7-7a35eb27402f`; intended new range is
180000–210000ms. Exact amber fact revision:
`4c6d25cd-98eb-4ae8-8692-b4120fbb2837`.
Pending proposal: `script.review.56905ae5-5e0d-46b4-a8fc-934bce5cb6b9`.
Explicit accepted B revision: `661499b9-52e7-47b6-9674-c6de8b7910ff`.
F's saved revision `956eec82-50ec-4404-9257-1f36ba9d9b22` remains unchanged;
its exact open draft survives placement, Bible Save, preview and acceptance.
The driver checks every other saved block, target identity/membership/placement,
cleared review and refreshed consumption of the amber Bible revision.

Visible review list-item bounds separately prove the manual source, Bible fact
and entering-E/displaced-A labels inside the real Script viewport. Pending
proposal characters and both Reject/Accept controls are also inside that viewport.
All three actual production-client requests are accepted, streaming and explicitly
`real_model:false`. One bounded retired accessibility-object poll occurred; other
errors were not suppressed. Native walkthrough duration was about 49.3 seconds.

Untouched originals under
`/workspace/scratch/story-authoring-native-success-37545170522/`:

- `eidetic-story-authoring-writing.png`: actual Write screenplay field, exact
  original A and timeline before Save; Bible is not shown in this frame.
- `eidetic-story-authoring-manual-edit.png`: saved morning C, manual B and visible
  downstream source review; selected B's stale Notes written caption is retained.
- `eidetic-story-authoring-placement.png`: real 180/210 inputs, retimed E source
  and open F draft. This is a transient guarded read: the inspector summary is
  still 540–570 and the composer offers Refresh selected clip. Canonical receipt
  verifies the move; subsequent frames show the refreshed 180–210 summary.
- `eidetic-story-authoring-review.png`: amber Bible, exact placement, all three
  review causes, unchanged human B, saved morning C and open F draft.
- `eidetic-story-authoring-preview.png`: full **Synthetic preview** paragraph,
  **Proposed screenplay update**, Reject/Accept controls, unchanged manual B,
  current Bible/timeline and F's open draft all visible together.
- `eidetic-story-authoring-accepted.png`: **Update accepted**, the same paragraph
  in canonical screenplay, cleared Needs review, current Bible/timeline and the
  still-open unrelated draft.
- `eidetic-story-authoring-retained-draft.png`: final exact F draft check; this
  consecutive checkpoint is byte-identical to the accepted frame.

[Successful artifact11450083908](https://github.com/MrScripty/Eidetic/actions/runs/37545170522/artifacts/11450083908)
ZIP SHA256: `d32920da60799969cc10f6f7c118a51d95280125ffef0cd68cb729e8c041f15d`.
Companion `story-authoring-walkthrough-manifest.json` contains the complete receipt,
original file hashes, local logs, exact sources and preserved failed attempts.
The separately fetched hosted CI receipt is supplemental and not part of the
original ZIP. Retention ends 2026-10-09T23:22:40Z; parent owns Library delivery.
No unresolved blocker remains for this bounded demonstration. Inspector feedback,
fictional-time mapping, semantic extraction, embeddings, real models, broader
feature-length qualification and project-switch recovery are not claimed complete.
