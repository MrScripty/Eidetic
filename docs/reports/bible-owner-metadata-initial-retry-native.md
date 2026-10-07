# Bible owner metadata and initial detail Retry qualification

Product source `c09db543f10b663381379f0532ab0d34ad939b62`, tree
`016da4f54d34faa384b9744d5e071e72c23ea8c6`, repairs the two confirmed
final-head review findings on predecessor `322be2607ab4206410667ca48d4b6d22cd96394a`.
Passed [hosted native run 37662915335](https://github.com/MrScripty/Eidetic/actions/runs/37662915335) qualifies the bounded initial-detail Retry interaction. Generic owner metadata is covered by server regressions.

## Persisted owner metadata

Generic SetBibleField acceptance now resolves the actual `field.part_id` and
preserves its stored name and order, just as manual-bound acceptance must.
Merged built-in projections can retain the stored field while exposing default
part identity/name/order, so their part metadata cannot replace the owner row.
The owner must be live and match the target node/key. Only an ordinary proposal
for a projected default part with no persisted owner and no persisted field ID
may materialize schema defaults. Missing bound, deleted or mismatched owners
refuse acceptance; SQL errors remain strict. No writer/CAS contract is expanded.

The generic regression seeds a custom stored part ID/name/order under the
built-in environment key, demonstrates the merged projection mismatch, and
checks actual owner identity/name/order plus field identity/order after acceptance
both with and without an explicit target field ID. A bound regression preserves
custom owner metadata; a deleted-bound-owner regression retains the pending
proposal and history counts without materializing defaults. Existing ordinary
unpersisted acceptance remains covered. Test-only SQL seeds stored metadata
differing from today's schema validation; this does not claim an ordinary native
UI flow for authoring arbitrary names under built-in part keys.

## In-place uncached Retry

An initial detail-load failure without a cached projection now exposes an ordinary
Retry saved facts button and accessible error. Retry calls the existing owned
refresh rather than remounting the inspector. Pending retry displays Loading;
only an admitted current node/session/version response restores the detail.
The prior detail, draft, field Save and read-owner guard modules remain byte-identical
to predecessor `322be260`. Cached failure/pending Save gating and late acknowledgement
draft preservation retain their established regressions.

## Current-source tests and review

546 UI tests in 94 files; 123 core and 532 server all-target tests; strict all-target
all-feature Clippy; typecheck with zero errors/warnings; lint, formatting, build and
normal commit hooks pass. Independent exact-source reruns pass 7 generic acceptance,
9 bound fact-proposal and 13 UI tests (11 compiled-client, two rendering tests).
Independent QA review admits exact `981d9d94b8c371351bc00136f25ea6b7ee433397`, tree
`c1569f75bd5d588b6df28cadcef60a1cecfd9bb7`, as the same enumerated nine-file
qualification boundary, and reruns 46 Python tests successfully.

## Targeted native qualification

Successful run `37662915335`, job `112934698095`, uses the existing hosted no-download route
and an explicitly labelled selected-read fault. Ordinary Mara selection
produced an uncached error and enabled Retry in both inspectors. One ordinary left Retry started a held read, cleared the shared error and
showed Loading in both inspectors. Release invoked the actual public GET and restored saved fields in both inspectors.
The exact motivation draft `"  Recovery draft — 雨.\n\n  "` became saveable after verification.
Save was never submitted; canonical facts, screenplay, placement and history stayed unchanged.
The right motivation stayed `"UNCONSUMED sibling: preserve the witness."`, and both saved
taglines stayed `"Mara's umbrella is red."`. Existing Needs review notices remain visible;
Retry did not create a new review cause or run preview/acceptance.
Four labelled synthetic fixture HTTP/SSE calls seed existing scene material; no
analysis or additional provider calls are permitted. This target does not rerun
the predecessor's full reconciliation/acceptance/restart walkthrough.

## Preserved evidence and limits

The original full walkthrough remains sealed at `1c1f240d61ee2331b188aff01b41fc5ccf4e3bdb`
and bound to `e5a09ede159a6511694896fdb5d4382e51f4418d`. Original captures, archives,
failed attempts and manifests are unchanged. Current-source native scope covers
only the new initial-read Retry interaction. Generic and manual-bound custom owner
metadata is covered by server regressions. No real-model quality, backend CAS,
unsaved-draft restart or project-switch recovery is claimed. Parent coordinates
external re-review and merge; this task requests neither.

## Original archive and custody

The original ZIP for artifact `11503180647` is exactly 610,177 bytes and contains
four 1920×1440 PNGs. ZIP SHA256 is
`05ce91b6fa227661df142f304072b45fe96989770c39959727f4d2193556c528`,
matching GitHub's artifact digest. All seven archive members are byte-identical
in the preserved originals. The runtime receipt binds the owned PID `25021`
and native binary SHA256
`4221a84a8a897470c1260feb60a6407f3050cb9c381e0ae3c77852d831846480`.

All local and hosted aggregates pass at the frozen source: 546 UI/94 files,
123 core, 532 server, strict Clippy, type/lint/format/build, no-download gate/five
checks and 46 qualifier tests. Normal source, QA and evidence commit hooks pass.
Independent review personally inspected all four originals, verified archive/PNG
hashes and dimensions, nine committed served-source hashes and the QA config,
and recomputed 20 recovered-fact and 16 final-draft nonwhite glyph rectangles
inside their textareas and left Bible pane. The exact final draft also has the
original successful native accessibility/typing receipt. The nine observed
source hashes include the new NodeDetail component; resolved fault-wrapper route
admission is true. No DOM/IPC injection, direct database writes or ONNX bypass occurred. The Retry/draft
sequence changed no canonical saved state; fixture setup uses explicit public writers.

Raw executable bytes, transformed served modules and SQL snapshots are not in the
archive. Their runtime identities and state comparisons are original driver/receipt
evidence, rather than artifacts independently redigested offline. Full-flow stale
acceptance, replay, restart, project switching and native custom-owner authoring
are outside this new target; prior full-flow evidence keeps its original source.

## Repository-size constraint and external delivery

The new source-Git delivery contains only this report, its hash manifest and six
small provenance/ACK files (23,173 bytes). New raw PNG collections, portable ZIPs,
PDFs and bulky logs are excluded. The exact original archive and all 29 source/native
records remain preserved locally, with unpublished local commits protected by
`preserve/bible-initial-retry-local-originals-02f2047`. Their hashes are retained in
the external-originals manifest. No history was rewritten, no existing evidence
was deleted, and no previously public artifact was moved.

The four original PNGs are flagged as lossless qualification originals: their
archive identity, screenshot hashes and exact glyph-geometry evidence must be
preserved. They remain outside newly published source Git. Ordinary display
images should use JPEG quality 85; no display derivative is introduced here.

[Hosted run/artifact](https://github.com/MrScripty/Eidetic/actions/runs/37662915335)
provides artifact `11503180647`, expiring **2026-10-10 18:16:31 UTC**. Parent-owned
Library/external delivery must preserve the original bytes and hashes before expiry.
Local extracted originals are at
`/workspace/scratch/bible-initial-retry-native-37662915335`; the full 29-record corpus
is at the local preservation root recorded in `external-originals.json`.
No permanent external upload or public-artifact migration is claimed by this task.

Existing full-walkthrough reports still reference already-tracked historical raw
artifacts at their original public commits. Those dependencies are retained;
removal or migration needs a separately verified plan.

## Immutable small provenance links

- [Source review ACK](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/source-review-ack.txt)
- [QA admission ACK](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/qa-review-ack.txt)
- [Native review ACK](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/native-review-ack.txt)
- [Current source hashes](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/source-hashes.json)
- [Hosted custody](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/custody.json)
- [External/local original inventory and delivery dependency](https://github.com/MrScripty/Eidetic/blob/1108344201819318456d1b5b2434fb1725cfe80d/docs/reports/bible-owner-metadata-initial-retry-provenance/external-originals.json)

## Original screenshot inventory (external/local)

| Original | SHA256 | Visible state |
| --- | --- | --- |
| `manual-fact-15-initial-detail-error.png` | `f23194c7965b5c1498956a8e4318fec16520c195a96f15a9fd12ba7aed42197b` | Uncached error and enabled ordinary Retry in both inspectors. |
| `manual-fact-16-initial-detail-pending.png` | `95a3e7272bb32e1441e64b044a8577783d796563adbf135325aa548bafce241d` | One left Retry starts shared Loading in both inspectors. |
| `manual-fact-17-initial-detail-recovered.png` | `2c51f36a6f8f7235dab79da874141e6bf5060da0c49376883d8991fc770b0869` | Actual public GET restores saved red fact and motivation. |
| `manual-fact-18-initial-detail-draft.png` | `3b077d7dbe0a2283197bdf35671ff680de2dc00151273e2343e0e4f92b663a26` | Exact unsubmitted left draft has enabled Save; right motivation unchanged. |
