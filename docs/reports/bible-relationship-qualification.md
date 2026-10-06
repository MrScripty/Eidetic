# Consumed Bible relationships and targeted screenplay review

Bible relationships supplied to screenplay generation now retain their exact
original revisions. Editing a consumed relationship identifies the affected
generated material, offers a targeted preview, and requires explicit acceptance
before replacing its saved screenplay. Manual screenplay and drafts survive
source refresh. Existing dependency, revision and proposal storage is reused.

## Verified gap and source

The existing Bible field edit, preview and acceptance workflow was complete.
Inspection found that `ai_bible_context_prompt.rs` supplied relationship endpoints,
kinds and labels while `script_generation_lineage.rs` bound only `BibleFieldInput`.
There was no `BibleEdge` semantic endpoint or relationship impact cause. The
Bible graph UI offered adding and deleting edges. The bounded gap and criteria
were recorded in the existing agent-story-workflows plan before implementation.

- Maintained source: `e94cbffa88fd3f58665e55a22b1c5969c60d3d4e`, tree
  `a08355b922698f34169b85c97860060499004bb8`, branch
  `fix/bible-relationship-label-custody`. Its label-write implementation is
  `017e92ec207d3d0c35ae49a4f412d44b9ee7595d`, tree
  `9dd1173d6e457c34e9e504e5a7a55559f131526c`; the follow-up changes only
  three boundary READMEs. The separately tested out-of-scope repair is
  `aeacee85b24ce6eecc93b1709613632b17019821`, tree
  `58ec7090cac837a0e02706b682ea491a220db1bb`.
- Qualification: `c5b962c18e241e5ec865a643ff888a405ad896de`, tree
  `34c2f046b5b7447bd06b987d01143f4b105c2d4e`, branch
  `test/screenplay-bible-relationship-label-native`. Only the dedicated workflow
  and two qualification driver files differ; every application/dependency blob
  matches the pinned maintained source.
- Frozen absence source `85170c9542322e2e197efcd07e1c2cafac3c8f6a`, tree
  `a78f46971cfb606a23e43a38cbb579cc6560b6c4`, remains on
  `feat/screenplay-bible-relationship-absence`. Its successful qualification is
  `786dfbc41d41e324575503db2142af3ea22193d9` on
  `test/screenplay-bible-relationship-absence-native`.
- Original source `237d0481d9cbc0c84df81d767cbd364b8d817ce4`, tree
  `7915e0ac83b45e0043612ff3f9417d1d5af5b9d2`, remains frozen on
  `feat/screenplay-bible-relationships`; qualification
  `15bfd3a235a5b6847f2c4ef45dbb6d897625c59c` remains frozen on
  `test/screenplay-bible-relationships-native`.
- Source and qualification author/committer are
  `MrScripty <TheEnvironmentGuy@protonmail.com>`.
- Earlier source `d672f0b36557593a47e34849bd564ca8b244ef2a`, qualification
  `ae96213f6d93a5babcc3f8a2dbebe778cd1c0963` and evidence
  `6b313aa3e1aeebbbb1c4d60e2629cbc8a91c4383` remain unchanged.

## Revision custody and review

Resolver projection and revision capture share one SQLite read snapshot. Only
actually supplied relationships create `UsesFact` dependencies, deduplicated by
edge identity. Original endpoints, kind, label, direction and owned revision stay
in existing generation and proposal JSON. Late output binds its historical read
and can need review. Changed or removed consumed relationships produce review
causes; the preview discloses the original relationship evidence.

Preview recording and acceptance both revalidate inside the existing writer
transaction. Canonical and sparse-history ABA remain detectable. Forged event
ownership and inconsistent historical/live payloads refuse. Legacy absent
receipts remain unknown and old proposals require fresh review. Global Bible
projection clock changes alone do not invalidate a preview: exact payload,
field-membership, target, scope and input revision guards remain. If any previously consumed live relationship leaves the supplied resolver
context, preview refuses until that context is restored, including unchanged
inputs when another visible cause is selected. Removed relationships derive their own
review cause; new relationship membership is a separate documented gap.

The label editor captures its edge's owned revision from node detail's same
read snapshot. The label-only backend command verifies that expected revision
and active identity inside the writer transaction, updates only label, and
records a single label delta in existing history. It does not copy cached
endpoints, kind, direction or sort metadata. Deleted, recreated or changed
identities refuse with all rows unchanged. Save/delete/edit/cancel share a busy
guard; interrupted or removed-edge drafts remain visible for explicit review.

## Validation

Core **119**, server **471** including sixteen added relationship cases, frontend
**456**, qualification drivers **39**, and no-download gate **5** pass. Strict
all-target server Clippy, frontend typecheck with zero errors/warnings, lint,
formatting, production frontend build, Rust formatting and traceability pass.

Service tests cover canonical label/kind/direction/endpoint and delete/recreate
ABA, sparse-history ABA, historical reads, duplicate/forged inputs, late output,
delayed preview, legacy absence, actual public generation/edit projection,
unrelated off-scope edits, same-command acceptance replay and all-row rollback
when a deterministic writer trigger changes the source after outer validation.
Unrelated edge edits both leave current material clear and permit an otherwise
current preview to be accepted. These are service tests, not a native concurrent
multi-user authoring claim.

The first local full suite encountered 17 read-only default-XDG fixture setup
failures; the task-local writable-XDG rerun passed all 464 tests. Initial fixture
assertion corrections and both outputs are retained in the manifest's local
receipts. No build-time ONNX download or ONNX403 bypass was used. Locked
all-features metadata retains Pumas `a94fd92021f27fdeedb6e2de6e01c41c250ef576`,
tree `4a6977b88ed532089807183e218a959ac02b724d`. No new dependency or schema.

The focused deleted-edge reproduction fails on frozen source237d048: after a
deletion preview, recreate/delete was silently accepted because both current
revisions were None. The successor records per-identity absence revisions from
the existing owned history in existing proposal JSON. It watches every absent
consumed edge, including when another cause is selected, without inventing value
consumption. All-row stale refusal, fresh explicit acceptance, preserved manual
text and missing-legacy-absence refusal pass. Both failed and repaired logs are
retained. No schema, store or dependency change.

A 206-node/two-consumed-edge regression fails on `85170c9`: selecting visible cause B
allows live edge A to disappear from the preview read. The separate repair reuses
current-generation UsesFact revision bindings to require all live inputs in the
new read. Changed and unchanged omitted inputs refuse both new preview and
acceptance of an earlier pending preview without any row or manual-text change.
All 467 server tests and strict all-target Clippy passed at that milestone.

Independent review also reproduced `85170c9`'s cached full-edge UPSERT recreating a
deleted edge or replacing newer kind/direction. Four real-store regressions now
cover label-only/history preservation, two-connection structural/deletion races,
sparse owned history, label ABA/delete-recreate and idempotent replay. Client-
compiled rune controller tests exercise interrupted save/delete, simultaneous
handler attempts, original revision dispatch, orphan draft retention and fresh
explicit save. API/store tests protect label-only IPC and newer cached responses.
These are executed store/client-controller tests, not a native multi-user race.

## Actual native qualification

[Run 37505367170](https://github.com/MrScripty/Eidetic/actions/runs/37505367170),
job `112412549433`, passed against exact maintained source `e94cbffa` and
qualification `c5b962c`. The normal locked desktop/fixture build, core 119,
server 471, frontend 456, driver 39 and no-download 5 gates passed. Artifact
`11432076755` is 980466 bytes, ZIP SHA256
`9e7ecf6504352c10ff1e82b12c9cd77f0ac9c5bcc88d47a33ccb53b96639e8d5`.
Receipt SHA256
`4ebdb348fd43c7ead002d3e7c0fbf464aa1d456d4006efb877b69f6607c36a5a`.
All nine original 1440x960 captures match receipt hashes and the owned window/PID;
all capture bytes have been visually reviewed. There is no failure in the receipt.

Actual native AT-SPI/X11 input authors the exact 85-byte manual screenplay,
generates B consuming the Red fact, edits `profile.tagline` from
`Mara's umbrella is red.` to `Mara's umbrella is blue.`, and verifies saved
screenplay, placement and exact 41-byte draft preserved. The existing field
preview stays pending until explicit acceptance changes only generated B.

The author then changes relationship `qualification.mara.eli` from
`Mara trusts Eli` to `Mara doubts Eli` through the guarded label editor. Both
endpoints, kind, direction, sort metadata, saved scripts/placement and exact
manual draft survive. The pending relationship preview discloses its original
Doubt label/revision while saved B stays Blue. Native Doubt/Trust/Doubt ABA followed
by explicit acceptance of the old preview produces the exact stale message.
All logical table row hashes/counts are identical before/after that refusal and
the proposal stays pending. Native Reject and fresh Preview record the newer
owned revision without modifying saved B; fresh explicit Accept replaces only B
with `EXT. STATION - NIGHT\n\nMara doubts Eli as they board the train.`,
refreshes its UsesFact revision and clears review. Exact manual A stays unchanged.

The final untouched capture shows the exact saved Bible label in its entry
(bounds 0,467,185,22; role entry; zero children), the Blue Bible field, both complete
saved B text lines inside the writing pane, and timeline tracks. The saved-label
editor is opened for inspection without Save. Targeted accessibility records
establish both hidden Edit controls and the showing exact entry; no assumption
about a plain span's text interface is required. Five provider calls are all
accepted, context-checked, streaming and explicitly `real_model:false`.

| Capture | Evidence |
| --- | --- |
| `eidetic-bible-fact-review.png` | Exact Blue fact and downstream review with Bible/timeline/script visible |
| `eidetic-bible-fact-preview.png` | Pending targeted field proposal |
| `eidetic-bible-fact-accepted.png` | Field proposal explicitly accepted |
| `eidetic-relationship-review.png` | Saved Doubt relationship; retained draft and downstream review |
| `eidetic-relationship-preview.png` | Pending relationship proposal and original revision disclosure |
| `eidetic-relationship-stale.png` | Exact stale refusal; saved Blue text and pending proposal preserved |
| `eidetic-relationship-fresh-preview.png` | Fresh pending proposal while canon remains unchanged |
| `eidetic-relationship-accepted.png` | Explicit acceptance replaces generated B only |
| `eidetic-relationship-readable.png` | Saved-label inspection, readable accepted B, Blue fact and timeline |

The machine manifest includes every capture hash, source/tree, original/fresh
read revisions, artifact/receipt hashes and preserved local test log receipts.


Preserved source `85170c9` passed [run 37501157593](https://github.com/MrScripty/Eidetic/actions/runs/37501157593),
job `112398623461`, qualification `786dfbc`. Artifact `11430641183` is 983729 bytes,
ZIP SHA256 `64a2fa695a4125974089ffecccc5fdc6b68b23ba6512d6a8039e5aed17aee6c5`,
receipt SHA256 `22b458f3ecf213a7a61f4e601fff3b432ee106ae97afe53bb8efb5b3e6e4fe3e`.
The exact stale message is an actual showing AT-SPI notification (role 101) with
zero children and bounds 313,379,375,16. The role-ALERT selector was incorrect;
shared reveal/wait truthiness can also discard childless accessibility objects.
The corrected observer records exact matching text nodes and returns a boolean.
The receipt proves all logical tables unchanged after old acceptance refusal,
then rejection of the obsolete preview, fresh original-revision custody and
explicit acceptance replacing only generated B. Exact manual screenplay/draft
survive. Five calls are deliberately synthetic. Nine unchanged 1440x960 captures
are hash/window/PID verified and visually inspected. This run does not qualify
the later label-write or out-of-scope source repairs. Its final saved-screenplay
lines are readable; the later qualifier additionally requires canonical Bible
label glyphs within the sidebar rather than an offscreen Edit control state.


## Preserved native failures

1. [Run 37476127400](https://github.com/MrScripty/Eidetic/actions/runs/37476127400),
   job `112311873606`, qualification `1df4b7d567b887b8489324f366aedc5b8c39b692`:
   all gates passed, then the driver timed out on an offscreen manual-card Edit
   button after completing the existing Red to Blue flow. No relationship edit
   occurred. Artifact `11420360569`, 413492 bytes, ZIP SHA256
   `51f4ae7190e7ae308d6631ea659e41f7386e6adf8e3014124e70605177506dd5`;
   four verified/viewed untouched captures. A qualification-only correction
   revealed the exact scoped button through AT-SPI before native pointer input.
2. [Run 37478851725](https://github.com/MrScripty/Eidetic/actions/runs/37478851725),
   job `112321376737`, qualification `396d2250fe1e63b6c86f1e95d94dde985ecead59`:
   all gates passed and the native Trust to Mara doubts Eli edit completed.
   Saved screenplay/placement, other edge metadata and exact manual draft were
   preserved. The driver timed out locating the draft container before any
   relationship preview. Artifact `11421300699`, 521574 bytes, ZIP SHA256
   `efc7ab7053a7c60639b2f8f843823bc9eb949b312cb0357e09372570c0deeee6`;
   five verified/viewed untouched captures. The final qualification-only
   correction requires the exact draft and unique enabled native Cancel button
   and opens the saved label editor for review inspection. Source stays fixed.

3. [Run 37482490140](https://github.com/MrScripty/Eidetic/actions/runs/37482490140),
   job `112333955333`, qualification `3f7047481dd9c3d473e1a56ec950736570e64e54`:
   all gates passed and the exact relationship edit plus draft preservation
   succeeded. After saved-label inspection scrolled the writing pane, a
   visible-only draft lookup timed out before preview. Artifact `11422652735`,
   528964 bytes, ZIP SHA256
   `8126cd7458b882729a08b98c9dfe34e5c8488a7820064565a4d5cfe7914e4b82`;
   five verified/viewed untouched captures. The final driver reveals the exact
   editable draft through standard AT-SPI scrolling and verifies its text before
   cancellation. Its regression test forbids the visible-only helper. No
   application source change.

4. [Run 37485011359](https://github.com/MrScripty/Eidetic/actions/runs/37485011359),
   job `112342689361`, qualification `ec69062111e44d558b810119fb93ad1547896195`:
   all gates passed. Exact draft revelation/cancellation and the relationship
   preview succeeded. The driver opened the older accepted Blue preview
   disclosure and timed out waiting for the new revision. Artifact `11424490179`,
   536216 bytes, ZIP SHA256
   `f5b5b846c339cd0d1f27122ba6da17117a67bc7796f984a2d04800d926a826ec`;
   five verified/viewed untouched captures. The final driver scopes receipt and
   decision controls to one exact proposed text with enabled Accept and Reject,
   excluding accepted/rejected history. Application source remains unchanged.

5. [Run 37487992013](https://github.com/MrScripty/Eidetic/actions/runs/37487992013),
   job `112352980786`, qualification `660153d71878f453a2b58b76512193af6ef2ac49`:
   all gates and the exact pending-preview locator passed. The strict pointer
   guard refused a disclosure fragmented partly outside the owned window.
   Artifact `11423889806`, 533601 bytes, ZIP SHA256
   `f02d2869e103ebee12dc389ea622a5b9a2f5087a2104aea35ee1b70436b96130`;
   five verified/viewed untouched captures. The final driver uses existing native
   panel resize and top-left scrolling, waits for fully bounded disclosure
   geometry, and preserves the pointer guard. Its final saved-text capture also
   requires both exact native glyph ranges within the writing pane.

6. [Run 37490179062](https://github.com/MrScripty/Eidetic/actions/runs/37490179062),
   job `112360527973`, qualification `15bfd3a235a5b6847f2c4ef45dbb6d897625c59c`:
   all gates, the exact pending disclosure and label ABA completed. The failure
   PNG visibly shows the exact stale-refusal message, still-pending preview and
   unchanged saved Blue screenplay. The broad accessibility lookup timed out before the all-row observation; this run
   alone does not qualify native ABA rollback. Artifact `11426305722`, 649654
   bytes, ZIP SHA256
   `399c4bec4402de85b6c9f5ed6c646f637dc71a8bdf411707ad7f0832bea916e2`;
   six verified/viewed untouched captures. The later role-ALERT observer also failed; the successful driver records the
   actual childless notification and preserves exact text/all-row checks.

7. [Run 37498130226](https://github.com/MrScripty/Eidetic/actions/runs/37498130226),
   job `112387843920`, qualification `73f6e55`, source `85170c9`: core 119/server 466/UI 453
   gates passed. The screenshot again displays the exact stale refusal and saved
   Blue text, but the guessed ROLE_ALERT observation timed out. The first 80-node
   snapshot omitted screenplay and could not establish actual error role.
   Artifact `11428938464`, 649604 bytes, ZIP SHA256
   `069657df0ccaf330841cfd160abdae354f38b971510febb8123a85eadc2e94f2`;
   six untouched captures verified/viewed. Its incomplete all-row/fresh-acceptance
   claims remain unqualified; the subsequent successful `786dfbc` run supplies that
   evidence. A transient leaf mock's own truthiness failure is also retained in
   local receipts and was corrected before the successful driver gate.

8. [Run 37503033300](https://github.com/MrScripty/Eidetic/actions/runs/37503033300),
   job `112404599630`, qualification `bad0bb6`, maintained source `e94cbff`: build/gates
   pass (core 119/server 471/UI 456/driver 39). Exact fact/relationship edits, pending
   review, all-row stale refusal, fresh custody and explicit targeted acceptance
   completed. An added final canonical-label glyph locator timed out, so this
   run is not labelled a fully successful qualification. The failure screenshot
   retains readable accepted B, Blue Bible fact and timeline. Artifact `11430973285`,
   977644 bytes, ZIP SHA256
   `33e337be935d9b65bfc9d418ea2d5189698a31e3f0336ab82dd6f9c9d73416c3`;
   nine untouched captures verified/viewed. The successor records all matching
   relationship text objects with role/child count/showing/bounds and opens the
   already-qualified exact saved-label editor for bounded read-only inspection.
   Application source stays pinned and identical.

## Limits and delivery

Provider responses are deliberately synthetic localhost HTTP/SSE through the
actual production provider client. Real-model quality, authenticated model
access and native SDK inference remain unqualified. Native input uses AT-SPI/X11
in the owned Tauri window with the normal launcher and Vite development UI;
the production UI build is checked separately. No injected JavaScript/IPC,
edited pixels, Git credential changes or unrelated user-data publication.

Timed relationship semantics, node-name provenance and new relationship
membership remain documented follow-ups. Sparse history/live graph divergence
refuses until a canonical graph command reconciles it. Project-switching
recovery and embeddings remain deferred. The final steering authorizes one normal draft PR after duplication/current-main
checks. Parent owns final CI/review, the review slot, merges and Library delivery.
No reviewer request or merge is performed.
