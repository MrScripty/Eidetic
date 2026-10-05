# Scoped untimed Bible context membership qualification

**Result:** implemented and locally qualified; new-source native UI and full
AppState execution remain unqualified. All provider responses used in executed
preview tests are explicitly synthetic. No real-model quality claim.

## Exact source

- Separate branch `feat/bible-context-membership`.
- Implementation `c63370a55eaa925a4ab684ea91463d480e360659`, tree
  `c2ef1e085fd946a9374b432d3bc5346a9a93755c`.
- Base/frozen prior feature head `8941f3ac61c7e1a823aa03a7ccb66ba8ff4f3e01`,
  tree `2fb2853264f0ff81b227273811e7cb2db34be16d`; its application source
  `9e8bd1c9d51250ac1c517945269278b7fe7e3d61`, tree
  `594a474ec14eb5d23353805287469558528dab6c`, is unchanged.
- Frozen native qualifier `776d16c6e1039641c00beba9dade31465c9f28fe`, tree
  `59c944b8bd3ba83c15c10be170cb049de0ce810e`, remains unchanged.
- Original verified PR9 `25b860d12fe37a3538a40616cc02ab4a81370863`, tree
  `d22b036891aed7eb66fc94059aa1b5c1d9aab85a`, is an ancestor. Existing manual
  authoring, timeline placement, stale-target and HTTP fixture mechanisms remain
  in use. Parent owns PRs, reviews, merges and Library delivery.

## Reproduction and bounded dependency rule

Before implementation, one isolated test imported the frozen application's actual
production modules. Authoring previously empty Mara `profile.motivation` increased
resolved baseline Bible inputs from one to two, but generated B had no review
cause. Editing its consumed tagline correctly produced exactly one cause. Exact
manual B and unrelated A/C survived both edits. Preserved evidence:
`/workspace/scratch/unconsumed-bible-audit/audit-evidence.json` and `test.log`.
The bounded rule was documented in the workflow plan before source changes.

An existing scene watches untimed fields only on entities proven relevant by
actual baseline Bible inputs in its latest generation, retained generation
provenance, or Direct UserSelected/AiSelected **node** assignments through the
existing context-influence owner. New default-list entities alone never broaden
old output scope. No text/name, relation, descendant or embedding inference occurs.

Canonical field presence is separate from actual supplied values. Optional
BibleContextScope in existing generation/proposal JSON records node IDs, eligible
field IDs and the existing append-only Bible/context epoch. A retained entity
outside the current resolver limit keeps its field-presence watch, without
fabricating BibleFieldInput values or UsesFact dependencies. Sparse historical
validation refuses incomplete, unrelated or forged receipts. New generations
record the membership anchor through existing semantic dependencies; impact is
derived by the existing projection owner. No new schema, database owner, model
dependency or embedding service was added.

Known legacy Bible inputs can identify additions only on their recorded entities
when sparse history proves the field absent/null at generation. Historically
nonempty omitted fields and unknown provenance remain unknown. Timed snapshot
keys, including tombstone history, cannot introduce new membership; later snapshot
creation alone does not remove an already watched baseline field. Broader timed
fact and graph semantics are outside this slice.

## Review and preservation

Manual fact population/clear/removal or newly relevant explicit assignment derives
an existing ContextChanged cause with exact entered/removed field labels. Bible
and context events refresh screenplay, review and cached generation context using
their existing owners. The UI distinguishes Bible membership review from
screenplay-window changes.

Saved human screenplay stays unchanged while the existing targeted preview is
pending. Only explicit acceptance replaces the reviewed block and refreshes its
lineage atomically; unrelated scenes and canonical facts remain unchanged. Entity
relevance survives accepted clears so restoring facts produces review. Late
generation retains captured historical receipts rather than rebinding to current
facts. Pending preview/acceptance refuses field/context ABA and intervening human
writes. Draft and proposal owners are preserved.

The shared Bible/context clock conservatively stales pending previews after
unrelated edits; those edits alone do not create Needs review. Pending proposals
created before this optional receipt require a fresh preview after upgrade,
because old bindings lack it; stored drafts/proposals and accepted legacy replay
remain intact. Existing outputs with unknown entity provenance are not backfilled.

## Executed local gates

| Gate | Result and practical boundary |
| --- | --- |
| `cargo test -p eidetic-core` | 118 passed, zero failed/ignored; optional receipt roundtrip and legacy absence |
| Actual production-module harness | 243 passed, zero failed/ignored; imports repository modules by path, no substituted persistence/proposal implementation |
| `cd ui && npm test` | 406 passed in 71 files, zero failures; Bible cause rendering and live context-event refresh |
| `ORT_SKIP_DOWNLOAD=1 cargo clippy -p eidetic-server --all-targets -- -D warnings` | Passed **compile-only**; no native runtime execution |
| Normal pre-commit hooks | Rust format, frontend lint/format/typecheck, decision traceability and conventional commit passed; Svelte 0 errors/warnings |
| Frontend build and successor traceability | Passed |

Nine new production-module tests cover legacy/scoped absent→present with both
existing null rows and entirely absent rows; synthetic targeted preview and
explicit acceptance; clear/remove and accepted-empty restoration; irrelevant,
candidate and Direct relevance; field/context ABA, stale requests, manual writes
and rollback; late generation custody; unknown legacy omissions; incomplete or
forged receipts; timed-key exclusion; scene-specific affected outputs; and retained
membership without invented value consumption outside the resolver window.

`canonical_generation_service_tests.rs` adds an actual AppState paused production
HTTP test: capture tagline while motivation is null, author exact
`Keep the station key` during I/O, release predefined synthetic output, preserve
manual anchors, and derive the new motivation review while retaining the original
receipt. This test and runtime capture assertions **compile but have not executed**
for this successor. Do not substitute harness counts for full server execution.

Frozen logs, harness manifest/imports and hashes are under
`/workspace/scratch/bible-membership-c63370a/`. Qualification receipt SHA256:
`497e55c10f35bd807e9cdbb064cace9d6a154b1371d43dc4bba282443d4b5762`.
The local full-workspace push test requires unavailable GTK/GLib/native dependency
execution; only that known pre-push hook was excluded for publication of this
tested branch. Local ONNX403 was not bypassed. No credentials, paid service or
external reviewer requests were used.

## Preserved screenshots and hosted limits

The prior [Bible native qualification](bible-fact-native-qualification.md) remains
source-specific proof of a real manual fact edit with Bible, timeline and Script
visible, retained draft, pending preview and explicit acceptance. Its inspected
unaltered PNGs in `/workspace/scratch/bible-fact-native-37337057557/` retain hashes:

| Prior screenshot | SHA256 |
| --- | --- |
| `eidetic-bible-fact-review.png` | `30c52c35ed4e8222dae17279cc61d871affd02fb097b5d12b6aafe13932e52b4` |
| `eidetic-bible-fact-preview.png` | `c090e141225b6952252cdb716a4ecdf11656e3ab6ddc93e185ab5fffab9b988a` |
| `eidetic-bible-fact-accepted.png` | `e56ec3ec991d285e64051823c0f39e28e8b7a8f1db4b5f9392c50e218086f365` |

All three prior canonical scene PNGs, evidence and sanitized log were rehashed
unchanged against the preserved attempt-2 receipt. See
[canonical native qualification](canonical-scene-generation-qualification.md)
for their source, exact hashes and visible limitations. They are historical
evidence, **not new-source membership screenshots**.

Final native refusal-banner/history qualification remains blocked after **two
zero-step runner failures** in run 37365296113: attempt 1 job 111948855213 and
[attempt 2](https://github.com/MrScripty/Eidetic/actions/runs/37365296113/attempts/2)
job 111956186116 both had runner ID 0, no steps/artifacts and cancelled jobs.
Attempt 2 public annotation reported hosted runner not acquired and internal error
correlation `be806b1d-dc07-4c96-9448-233db2cd458b`; it completed at 20:20:08 UTC.
Preserved receipt `/workspace/scratch/canonical-scene-native-37365296113/rerun-attempt-2-evidence.json`
SHA256 `15b9a7bb130294797dc7813b90dd4be39514c8575fbff561597f182310247b4b`.
No repeated rerun was requested while infrastructure was unavailable. This
successor has no hosted/native screenshots or real-model execution; those are
remaining qualification gates, not completed claims.
