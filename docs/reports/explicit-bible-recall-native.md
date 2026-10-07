# Explicit Bible recall native qualification

Application source: `8bd0da0daa22897996b8e60976bf2de63fffe4eb`
(tree `ccae6c03b8e7e34706ca29b05833d37acf456b6d`), branch
`feat/explicit-bible-recall`, normally merging verified main
`c4587c11911af355c2446d0befa5ac1cbde8f3e3` and original recall source
`14f757561be5cc833b408d4355386afc41a92255`. The initial branch base was
`8000f29e96517b1862f9cb091150a315c1f351ca`. The accepted
`25b860d12fe37a3538a40616cc02ab4a81370863` successor is an ancestor.

Read contract milestone: `d23c330744e8c53abd7b48199da3f73b0cf9af18`.
Implementation milestone: `975113011699f7d1596aebf38fee3c11173b8028`.
The final path fix `14f7575` preserves one contiguous relationship label through
source formatting; it changes only the recall component and its README.

## Concrete gap and resulting behavior

The prior audit found that `ReadBibleNeighborhood` delegates to the bounded
renderer projection: selecting a node does not expand its incident edge
endpoints outside the default 200-node prefix. `ReadBibleNode` supplies node
metadata rather than resolved field evidence. Existing manual Bible
edit/targeted-preview/explicit-accept and screenplay draft workflows were already
working and are retained.

The new explicit “Recall related story facts” inspector and `ReadBibleRecall`
agent tool call one shared domain projection. An exact selected Bible entity
returns itself and at most eight live one-hop neighbors, with at most 32 typed
paths preserving stored edge orientation. Incoming/outgoing filters respect
directed edges; undirected associations remain traversable in both directions.
Self loops do not invent a neighbor; cycles do not trigger further traversal.
There is no filler from the renderer prefix.

The read reuses SQLite, sparse history, temporal resolution and existing
field/name/relationship custody. Baseline facts retain owned field write IDs.
Timed facts retain the resolver's exact snapshot and assertion identity, with
separate snapshot metadata and field revision clocks. Names with unavailable
history stay explicitly unknown. The values, source receipts and global Bible
revision envelope come from one SQLite read snapshot.

Fictional time is optional and explicit. Unspecified time withholds timed values;
future-only and cleared fields remain unresolved; same-time disagreements refuse
the read. Relationship associations have no recorded fictional-time validity and
are labelled untimed. Connectedness is evidence, not a truth assertion.

The 32 KiB envelope budget omits whole field records with counts, retaining
identities, paths and their qualifiers. Oversized identity/path evidence refuses
instead of cutting values or qualification. Reads do not write generation
receipts, canon, history, dependencies, context links, proposals or saved script.
They do not change the renderer's selection rules or add facts automatically to
generation context. No embeddings, model downloads or new persistence dependency
were introduced.

## Guards and local qualification

The disposable inspection cache guards exact request/query, selected entity,
editor session, request order and prior revision floor. Bible command admission
and completion, Bible events, selection changes and cleanup revoke outstanding
reads and displayed evidence. Late success/error cannot replace newer work.
Existing screenplay draft, placement-intent and proposal owners remain intact.

Combined-source local gates pass: core122, server511, UI503 in85files; strict core/server
all-target Clippy; Rust formatting; UI type checking0errors/0warnings, lint,
format and build; traceability; source driver39 and no-download admission/tests5.
QA adds seven native driver regressions (46 total). The public fixture smoke check
returns only Mara and Beach House at Bible revision221, with 205 unrelated
entities and Beach House outside the default 200-node prefix.

The full Rust run initially found17existing project-lifecycle failures from the
sandbox's read-only default home data directory. The existing tests pass with a
writable workspace `XDG_DATA_HOME`; no production change or assertion removal
was made. Full local desktop compilation still depends on Linux system libraries
absent in this workspace; hosted qualification uses the standard native setup.
The existing ONNX403 route is not bypassed.

Deterministic regressions cover >205nodes/no filler, type/direction/undirected
paths/cycles, neighbor/edge/byte budgets, separate source clocks, temporal
boundaries/withholding/clear/conflict, deletion/history inconsistency, WAL
snapshot consistency, actual shared agent read, unchanged generation context,
saved text/pending proposals/dependencies and stale frontend responses. The
manual edit session retains exact unsaved text and its saved revision.

## Frozen native source

Qualification `cf9830b6b5df3dc56cd01ab6f8384039a33a0729`
(tree `5dcc824866352f3ee8ac198639911b8212c2e252`), branch
`test/explicit-bible-recall-native`, differs from application source only in:

- `.github/workflows/bible-recall-native.yml`
- `crates/server/examples/bible_recall_capture_fixture.rs`
- `scripts/qualify-bible-recall.py`
- `scripts/test_qualify_bible_recall.py`

The workflow verifies this complete allowlist and source ancestry before build.
It uses standard Linux GTK/WebKit/AT-SPI/X11 dependencies, Rust1.92, Node24,
locked Pumas/ORT dependency admission, the actual desktop binary and public
project/command/read services. There are no injected DOM/IPC calls, direct
project-database writes, edited screenshots or alternate ONNX download route.

## Successful native execution and original visual evidence

[Run 37568916076](https://github.com/MrScripty/Eidetic/actions/runs/37568916076),
job `112622899670`, completed successfully at qualifier `cf9830b6` against exact
application source `8bd0da0d`. Hosted core122/server511/UI503, strict Clippy,
UI checks/build,46driver regressions and5no-download checks pass. The final
qualification does not require another source change or another native run.

Original artifact: [11459778402](https://github.com/MrScripty/Eidetic/actions/runs/37568916076/artifacts/11459778402),
`eidetic-explicit-bible-recall-native-8bd0da0`,1,066,965bytes, expires
10 October2026 at04:02:42UTC. Downloaded file
`file_000000006d8481fd99b0597f64f1190e`; untouched ZIP SHA256
`aebef0fdb00ef2a842201676f22320739f7f2621eb3fc3349a917fdf116f12e5`.
The ZIP digest matches GitHub’s artifact digest. Every extracted member matches
its original ZIP bytes; every PNG matches raw capture metadata and is1920×1440.
All seven originals were inspected individually, without image edits. Original
files are retained in `/workspace/scratch/bible-recall-native-success-37568916076/`.
The owned native process was PID24017/window2097155. Recorded running-binary
SHA256 is `9fbfffdd76197adce8240ab9d9cd8530fd3e94f7ecce2acbb621d9bf22a3ec3b`.
Some deterministic views match earlier attempts byte-for-byte; original run,
PID, source identity and metadata are preserved separately for each attempt.

The graph canvas stays closed with Bible, timeline and screenplay visible.
Public services seed205unrelated entities; Mara’s linked Beach House lies
outside the renderer’s200-node prefix. Actual native controls execute Generate,
manual B Save, the F draft, each explicit recall and the exact Bible Save.
Recall comparisons retain all canonical commands/history/dependencies/proposals,
saved scene text/revisions/placement and the unrelated draft. The authored fact
change derives review only for the consumed generated B; supporting A/F saved
text remains untouched. Fresh recall is explicit; preview leaves a pending
proposal and requires user acceptance to replace saved text.

| Original capture | Individually observed state | SHA256 |
| --- | --- | --- |
| `eidetic-bible-recall-untimed.png` | Unspecified story time; Mara’s blue fact, directed path to Beach House, unresolved weather/description, untimed warning and omission counts visible. | `71ddccd394bc3425aaba193ee71d8dddb2f0f2904130cc0f9f02b5d63ae28f7b` |
| `eidetic-bible-recall-before.png` | 999 ms; baseline Dry visible, future description unresolved; saved manual B and unsaved F draft remain visible. | `265149a3579f0195b3e49950cc537b0a49795f68c5fa46c40f5c1bef8de3b551` |
| `eidetic-bible-recall-at.png` | 1000 ms; Rain and the opened Opening snapshot/field source IDs and revisions visible; typed directed path and temporal qualifiers visible. | `d88f722893ef90ab017c03cf87810485f062ea815d48c6e27f5d09126867ac2b` |
| `eidetic-bible-recall-fact-review.png` | Mara’s exact amber fact visible in Bible; B needs review with Bible fact profile.tagline changed; manual B and F draft retained. | `bdd46db945205a5ad4c1e0fb32c6921921cf07ea1d4ea212a416485b5571f23b` |
| `eidetic-bible-recall-invalidated.png` | Facts changed / Recall again notice visible; old recalled values removed; amber fact, B review and retained manual/draft visible. | `cc669dfb1ba0a3a940faa6d7b02282e270bb0fc2c47bbeb3c8d7d2b13446c715` |
| `eidetic-bible-recall-refreshed.png` | Explicit fresh recall shows revision 222, exact amber fact, Rain and the directed path; unresolved future description and omissions remain visible. | `b872fdc3d4356e5d1d72ffecc3b8260eb54e2dad8f15cc3e22b7f08142ca40f2` |
| `eidetic-bible-recall-pending-review.png` | Synthetic proposed paragraph plus Reject / Accept update wholly visible in Script; saved manual B remains in the adjacent column, F draft and refreshed Bible remain visible. | `03a960589cdcf9a2ece734588abd04035904de1ee566318206401b85d21dc260` |

Exact native Bible change: `Mara's umbrella is blue.` →
`Mara's umbrella is amber.` Baseline revision
`40093e72-bcfa-4501-990c-e0ced20fa54d` becomes
`899192d7-6a92-4388-9604-d3052fe5d562`. B’s retained manual revision is
`ee2c17c3-1672-415c-83fc-86a3e4f015cc`:

```text
EXT. STATION - NIGHT

Mara keeps the witness hidden. Preserve this station beat.
```

The retained unsaved F draft is `Unrelated draft: Eli pockets a brass whistle.`;
its saved text remains `EXT. PLATFORM - NIGHT` / `Eli keeps the gate open.`
Pending proposal `script.review.2bc4aabf-ea84-46b7-b784-9bdecea864bf` is visibly
labelled `Synthetic preview: Mara waits for Eli with her amber umbrella.
Preserve this station beat.` The original raw receipts preserve exact trailing
newlines and UUIDs. At1000ms the inspector separately displays snapshot
`qualification.house.opening` and field
`qualification.house.opening.weather`, each revision
`d26aa425-6b9e-4ef5-a6f8-cb34a7575fc3`. These clocks coincide in this fixture;
deterministic regressions additionally exercise separate metadata/field writes.

The three production-client localhost HTTP/SSE records are generation, recap
and preview, all accepted with exact expected saved/current context and
`real_model:false`. They exclude the unsaved draft and unrecalled neighbor from
generation/preview input. Recall itself executes no model. This qualifies domain,
transport, context custody and review behavior; no real-model narrative quality
is claimed. Accept update was intentionally not clicked, so this run does not
claim a new accepted screenplay or native stale-accept/ABA execution.

Visual limitation: native direction/kind selects have faint text against light
backgrounds. The captured run uses default Both / All kinds; other direction/kind
behavior is covered by deterministic tests. Facts, source revisions, temporal
warnings, omission counts, authored text, draft and review controls are readable.
This is functional workflow qualification with that recorded contrast limitation,
not a claim of polished selector presentation. Tested application8bd0da0 is
unchanged; no visual repair or repeated native run is introduced.


## Preserved earlier qualification attempts

Run37561728027/job112600279687 at qualifier1b064d8/application9751130 stopped
during UI tests before native launch. Source formatting had split the exact
relationship path into multiple text lines;497UItests passed and the exact-path
SSR assertion failed. There is no native artifact. The14f7575fix restores one
text expression and all498UItests pass after formatting.

Run37562158707/job112601642742 at qualifierc8cdc6f/application14f7575 passed
source identity, dependency/UI/driver/no-download checks and was cancelled during
native build when superseded by7f07fe2. The driver locator now distinguishes the
expandable recall control from its enclosing section with the same accessible
label. No native launch/capture occurred in that cancelled attempt.

Run37562396815/job112602455375 at qualifier7f07fe2/application14f7575 passed
the complete hosted build, admission and test gates, then launched the native
application. Actual Generate, exact manual B Save and unrelated F draft entry
completed. It timed out finding the recall disclosure because native WebKit
exposes HTML summary as `unknown`, while the driver accepted only button roles.
The original accessibility tree and failure PNG show the summary and its
same-named enclosing landmark separately. This is a QA locator failure; no
recall execution or downstream edit is claimed for that attempt. The original
1920×1440 PNG was individually inspected and its hash verified against the
untouched ZIP and raw metadata. Its two provider responses are labelled
synthetic. Qualifier9355ea5 includes the observed summary role, excludes the
landmark, and passes all43driver tests. Application source remains14f7575.

Run37564109458/job112607793716 at qualifier9355ea5/application14f7575 also
passed all hosted build/test gates. It opened the actual summary and executed
recall. The original failure capture shows Mara, linked Beach House, the exact
typed path and revision221. The unresolved paragraph was below the nested
sidebar viewport, so the driver's visibility-only wait timed out. The original
PNG was individually inspected and verified against its untouched ZIP and
metadata. No temporal-boundary, fact-edit or downstream-review completion is
claimed for this attempt. The corrected driver explicitly scrolls matching
result text into view and normalizes native line wrapping;44driver tests pass.

Run37565741532/job112612913882 at qualifier3fc609f/application8bd0da0 passed
the combined-source hosted gates (core122/server511/UI503). It stopped before
generation while locating the connected fixture status. The original capture
shows “Connected”; broad scrolling had selected a hidden enclosing accessibility
section first. There were no model requests and no screenplay generation, fact
edit or recall execution. Its original1920×1440PNG was individually inspected
and verified against the untouched ZIP/metadata. The driver now prefers visible
status text, restricts fact lookup/scrolling to actual paragraphs, and uses the
existing right Bible inspector so results fit without the left inline detail's
clipping. It also opens the timed fact source disclosure;45driver tests pass.

Run37567519302/job112618522966 at qualifier5896ae9/application8bd0da0 passed
all hosted gates and native Generate, exact manual B Save, retained F draft,
untimed/999ms/1000ms recall, exact timed source disclosure, manual blue→amber
Bible Save and visible affected-B review. It then timed out locating the
invalidation notice because its ARIA status role was excluded by the paragraph
locator. The original failure capture shows the correct notice and revoked old
values. All five original1920×1440PNGs were individually inspected and verified
against the untouched ZIP/metadata. No fresh recall or pending targeted proposal
is claimed for that attempt. Two responses are labelled synthetic. The corrected
text locator recognizes native status/text roles;46driver tests pass. Source
remains8bd0da0.

## Arc/main compatibility retained

The separate arc branch `feat/screenplay-arc-memory` is pushed at
`aa14271222dc37bb80056dc2388d9aeaaed97099`
(tree `104ce2797a55621036d7c54d6f5aa1bb64caebe6`).
Normal merge `de2c752a304ef696ab552129b9db6cc3dde8a840` merges exact main8000f29
and reviewed292630b. Its full tree equals reviewed292630b'sc7d7c16b tree.
The final aa14271delta from292630b is only `ui/src/lib/stores/README.md`, added
for current-main traceability. Application bytes remain equivalent.

The arc premerge current-target CI run37559251532 at aa14271passes Frontend112592548738,
Linux Rust112592548920, Windows Rust112592548954 and Traceability112592549504.
Local arc gates are core120/server504/UI492 in83files, strict Clippy, formatting,
type/lint/build, traceability, source driver39 and no-download5.

Original arc native qualification remains292630b/a90ce8d/run37557179034/
artifact11455274592, original ZIP SHA256
`4c8a1779061134b420deb2ce2f30203ad8d8f8b19849094cd6199972fe688027`,
file`file_00000000fd1081f795758c6f1f557bbd`. These receipts are reused on the
documented byte-equivalent application; they are not relabelled as a new native
execution at aa14271. Parent merged PR18 normally atc4587c1, parents8000f29
and aa142712, tree104ce279. The recall integration8bd0da0 preserves normal
ancestry; its only conflicts were README additions, both retained. Parent owns
PR18 postmerge CI and review/merge coordination; this task does not duplicate it.

## Delivery scope

This candidate supplies explicit inspection, not automatic generation-context
expansion or a new accepted world update. The existing targeted screenplay
preview and explicit acceptance mechanisms remain the replacement boundary.
Real-model quality is not claimed. Evidence is published separately. The parent
authorizes a duplicate-checked draft PR for feat/explicit-bible-recall after
evidence completion, and retains review, merge and Library delivery. No main
merge or manual external reviewer request is performed by this task.
