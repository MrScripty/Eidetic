# Consumed story-arc screenplay memory

## Outcome and source authority

Authored changes to an originally consumed, revision-owned tagged story-arc
field now identify the saved screenplay blocks that consumed it. The existing
targeted review shows original and current values, supplies current context to
the provider and preserves saved manual text until explicit acceptance. Accepted
updates install current consumed lineage through the existing dependency store.

Application source `292630b91b6677684279f852fdfb7257b8c97aa8`, tree
`c7d7c16b38d4c0a52a87e5d98eae40a20f6580f5`, is published on
`feat/screenplay-arc-memory`. Production milestone
`d0967efabc9ef908dccf5626b0dd9ef7b5811a67`, tree
`039e1da68c05ce9b647ea8f2a67ad1479098b1ca`, has identical application code;
292630b adds four ownership/ledger documentation updates only. The branch stacks
on PR17's separate completion repair `f05fa888245db0fff1dad5a3165b0c1d3e07539c`.
Verified main at implementation admission was
`25895e7bcf215e5a013dd7c4a98873e9b7412b8a`. Main and earlier evidence were preserved.
Parent owns PRs, review, merges and Library delivery.

## Verified missing feature and ownership

The prior Bible field, relationship and name propagation and manual screenplay
edit/preview/accept workflows were already complete. ArcDetail already saved
typed metadata through the canonical command and sparse StoryArc history. The
screenplay prompt consumed tagged arc name, type and nonempty description, but
generation receipts and semantic endpoints could not retain that context. This
bounded gap is recorded in the existing workflow plan and
[consumed-arc-memory.md](../plans/agent-story-workflows/consumed-arc-memory.md).

The change captures tags, exact canonical prompt fields and owned per-field
history in the same snapshot as generation custody. It retains those fields in
existing generation-command JSON and existing semantic dependency tables.
Template fields with no history retain exact values with an explicit missing
revision; legacy missing receipts stay unknown. Color changes, unconsumed fields
and today's tags cannot manufacture historical consumption.

Targeted proposal bindings retain original/current fields and owned deletion
custody. The writer recaptures custody before proposal storage and acceptance.
Late normal generation retains its actual historical inputs, even across ABA,
and can arrive already needing review. Clearing a consumed description keeps
the empty current value. A live originally consumed arc removed from current
tags refuses preview. No parallel memory store, migration, embedding dependency,
new model dependency or project-switching recovery was introduced.

## Deterministic qualification

Exact application checks pass: core120, server504, UI492 across83 files, strict
all-target server Clippy, rustfmt, typecheck0 errors/0 warnings, lint, format,
production build, traceability and conventional precommit gates.

Eighteen added server regressions cover affected-scene selection, independent
field clocks, original/current evidence, explicit acceptance, late generation
and replay, unknown legacy/template receipts, unconsumed empty descriptions,
forged values/history drift, clearing/deletion, ABA and deletion/restore ABA,
source edits during preview, target edits, both lock kinds and failed streams.
Five added UI cases verify evidence rendering and canonical event refresh while
retaining unrelated screenplay drafts, placement intent and pending proposals.

The documentation boundary gate first reported missing core AI/contracts/UI
ownership READMEs. The source documentation milestone supplies them; normal
committed-source traceability then passes. Earlier compile/test/Clippy and QA
failures remain sealed in the manifest alongside final passing logs. Local
ONNX403 was not bypassed. The hosted route uses the existing pinned dependency,
standard no-download admission and actual native desktop build.

## Hosted native qualification

Successful run: https://github.com/MrScripty/Eidetic/actions/runs/37557179034

- Application source: `292630b91b6677684279f852fdfb7257b8c97aa8`.
- Qualification head: `a90ce8d2a4c5885c3daad1adf45de5ac87d11c31`, tree
  `206b1789799cb396ed853b1a7148039680a10b71`, on
  `test/story-arc-memory-native`. The application source is an ancestor and
  the workflow verifies the twelve-file QA-only delta. No frontend injection,
  scripted DOM state, direct project database writes or edited captures.
- Job: `112585981839`, success. Hosted core120/server504/UI492, driver54 and
  no-download5 pass, alongside standard native builds, strict all-target
  server Clippy and all UI checks. The fresh Vite assets belong to the checked
  source; a Rust executable hash alone does not establish frontend authority.
- Artifact: `11455274592`, `eidetic-story-arc-memory-native-292630b`.
  Original ZIP SHA256:
  `4c8a1779061134b420deb2ce2f30203ad8d8f8b19849094cd6199972fe688027`.
- Download file ID: `file_00000000fd1081f795758c6f1f557bbd`.
- Untouched originals:
  `/workspace/scratch/story-arc-native-success-37557179034/`.

The native fixture uses existing multi-cam tags. Driver aliases B and F denote
actual scenes **A: Setup** (0–60 seconds, tagged Witness) and **C: Beat**
(60–120 seconds, unconsumed support); these are not literal scenes named B/F.
Public services prepare the fresh Witness project. Actual X11/AT-SPI controls
generate the selected scene, save authored manual text, open an unrelated draft,
type the arc description, preview, attempt stale acceptance, reject and accept.
The unchanged Bible fact is `Mara's umbrella is blue.`.

The exact authored arc edit is `Mara conceals the witness's identity.` →
`Mara reveals the witness's identity.`. Original owned revision
`933e0cfe-557c-4912-9458-0c4414248727` advances to
`bc114562-ca58-4652-a334-494c87432e1b`. The generated receipt retains the
original value/revision. A: Setup's saved manual text stays
`EXT. STATION - NIGHT\n\nMara keeps the witness hidden. Preserve this station beat.\n\n`
through the edit and pending preview. C: Beat's exact unsaved draft stays
`Unrelated draft: Eli pockets a brass whistle.\n\n`.

Native NEW→OLD→NEW editing creates distinct owned revisions
`92ff0e53-e799-42df-acbb-b05ad1977f11` and
`29f861ec-53ad-4b3a-a76b-3016a67fa2b4`. Clicking Accept update on the old
preview visibly reports `screenplay proposal is stale; request a fresh preview`.
Canonical reads verify no saved screenplay replacement and retained draft.
After explicit Reject, a fresh preview again leaves all saved blocks unchanged.
Explicit Accept alone replaces the same A: Setup block with
`Synthetic preview: Mara names the witness for Eli.\n\n`, revision
`5211cd60-3da8-46f6-8b31-faf9973ccfc5`, preserving placement/membership,
refreshing its arc dependency to the latest owned revision and clearing review.
The support block, unsaved draft and Bible remain unchanged.

All ten original 1920×1440 PNGs were individually inspected and capture hashes
verified. Key files in the original directory:

| Screenshot | Visible evidence |
| --- | --- |
| `eidetic-story-arc-arc-edit.png` | Exact typed current description, affected-scene notice, saved manual text and unrelated draft. |
| `eidetic-story-arc-preview.png` | Bible/timeline/screenplay together; full original/current arc change and pending proposal with Reject/Accept controls. |
| `eidetic-story-arc-stale.png` | Actual stale refusal, original/current evidence, saved manual text and draft retained. |
| `eidetic-story-arc-fresh-preview.png` | Rejected old proposal and full fresh pending proposal with explicit acceptance controls. |
| `eidetic-story-arc-accepted.png` | Accepted proposal, replaced saved block, cleared review and exact unrelated draft still editable. |

Generated/manual-edit/draft/review/retained-draft captures complete the sequence.
The manifest embeds the original capture metadata without rewriting it and seals
the ZIP, raw official CI log/metadata, all local receipts and all46 source files.
All four production HTTP/SSE calls are labelled synthetic and real_model:false;
no real-model narrative-quality claim is made.

## Preserved failed qualification attempts

Run37555601615 / job112580990141 on QA head
`bbfa423b500a029bbc950d8241c911c6b27836b3` passed standard builds, strict Clippy,
core120/server504/UI492, driver53 and no-download5. The actual application
generated A: Setup, saved the exact manual screenplay, retained an unrelated
C: Beat draft and typed the Witness arc description from
`Mara conceals the witness's identity.` to
`Mara reveals the witness's identity.`. It showed the affected-scene notice
with Bible, timeline and screenplay visible. The provider returned the first
pending preview and the UI rendered its full original/current arc evidence.
The driver then called a nonexistent proposal helper and stopped before stale
refusal or acceptance. All six untouched PNGs were individually inspected.

Artifact11454756554 is preserved as file
`file_00000000026481f79770c7bdd2508cda`; ZIP SHA256
`95f39107564d8df6769ff682c480b1b8ff1d5ac19a5d6c6db604d5e05b1cb79b`.
Originals are under `/workspace/scratch/story-arc-native-first-37555601615/`.
Its raw metadata's generic prototype label and copied Last Train checkpoint
remain unchanged; the QA successor corrects them to the actual native Witness
workflow. These files are partial evidence, not a complete qualification.

Run37556882759 / job112585050361 on QA head
`3f60585e5c10adb7d0d0e5411880d02c4e5722b9` added the read-only pending proposal
reader but failed its new unit test before launching the application. Its
minimal test database omitted the script_blocks table required by the existing
read helper. The local log had the same failure; an earlier status update
misread it. QA successor `a90ce8d2a4c5885c3daad1adf45de5ac87d11c31` adds that
schema table and all54 driver cases pass locally. No application source changed.
Raw official logs and metadata of both failures are preserved.

## Practical limits

Only previously consumed fields with owned revisions produce these arc causes.
New tag membership, recap arc labels, affect and child-plan arc receipts remain
separate scope. Existing missing historical receipts are not reconstructed.
All provider responses used for qualification are explicitly synthetic through
the production HTTP/SSE client. Real-model narrative quality is unqualified.
