# Bible recall inspector lifecycle native addendum

Application source: `4c468d6386bcad44117fe7e4439202cdde52a0f7`, tree
`6f546f9d75bffe6baef70912f7635996f6acb244`, directly succeeding
`8bd0da0daa22897996b8e60976bf2de63fffe4eb` on local branch
`fix/bible-recall-inspector-lifecycle`.

Qualifier: `3cccc2b683903e5ec45531fa990a45043adb524e`, tree
`135725a8746492d23c2d15bff38afc2aae2ad8c8`, published separately as
`test/bible-recall-lifecycle-native`. Its entire delta from frozen application
source is eleven allowlisted qualification files; production component, store,
transport, domain and canonical-command files remain byte-identical.

## Source repair and deterministic qualification

Component cleanup previously called mutation invalidation. Disposing an inspector
could clear another inspector's shared evidence and leave the same anchor with
an invented “Facts changed” notice. The successor retains inspector owners by
anchor and editor session. Releasing one owner preserves current shared evidence;
the final release revokes work without inventing mutation status. Genuine
invalidation and the revision floor survive same-anchor remount. Releases are
idempotent; obsolete anchor/session owners cannot revoke current work.

The component lifetime effect tracks only its node ID, using `untrack` for shared
owner/projection reads. Query controls revoke inspection work without claiming a
canonical write. The store README's four-cell contents row now has two cells.
Only six UI source/test/README files differ from `8bd0da0`; canonical storage,
draft and proposal owners are unchanged.

Local successor gates pass: **513 UI tests in 85 files**, including ten added
regressions; lint, format, typecheck 0 errors/0 warnings, production build, Rust
formatting, decision traceability and commit hooks. Independent review found no
blocking defect and independently passed all 21 affected tests. Retain/release
tests directly exercise ownership; SSR does not execute client effects, motivating
this actual native lifecycle addendum.

## Instrumented native scope

[Run 37573408905](https://github.com/MrScripty/Eidetic/actions/runs/37573408905),
job `112636877757`, passed at exact qualifier `3cccc2b` against source `4c468d6`.
Hosted **core122/server511/UI513**, strict core/server all-target Clippy, UI checks,
49 native-driver regressions and five no-download checks pass. The standard
GTK/WebKit/AT-SPI/X11 and locked Pumas/ORT setup remains in use; ONNX403 is not
bypassed. No further native run or application-source change was needed.

The ordinary sidebar Arcs/Bible tab transition removes/remounts the inline
production inspector while the right inspector remains. This is a normal app
interaction. Closing the final ordinary inspector also clears selection, and
the regular recall API has no delay control. Two explicitly labelled compiled
test-build seams therefore isolate the remaining lifecycle cases:

- A wrapper around the unchanged production `BibleRecall.svelte` controls final
  disposal/remount without changing its node ID or editor session. Actual Svelte
  client effects and the unchanged shared store run in the native Tauri window.
- Only the recall store's API import uses a qualifier completion wrapper. It
  first executes the original desktop/domain read, then holds completion. Success
  returns that exact evidence; the error branch is a labelled synthetic error.

These controls are installed solely by `scripts/recall-lifecycle-vite.config.mts`.
Normal app builds do not install them. No DOM/IPC injection, direct database
writes, edited screenshots, browser-preview substitution or fabricated recall
values are used. This qualifies production component lifetimes in an instrumented
native host; it is not an uninstrumented release qualification.

The driver starts this explicit Vite config at localhost5173 before the unchanged
desktop launcher. Raw evidence records host PID `24236`, exact arguments/cwd and
config SHA256 `b463e938a57396278c8c72bde4a70dcf8d9700191708b10b1c2842ff60d41dd0`.
Local served-module admission verifies both scoped imports and the original
component import. Hosted native receipts and held-read behavior require those
seams to be active. The inherited sanitizer removes URL-bearing launcher lines,
including the reuse message; independent socket-owner enumeration is not claimed.

Desktop window `2097155` belongs to PID `24278`. The running Rust binary SHA256 is
`9fbfffdd76197adce8240ab9d9cd8530fd3e94f7ecce2acbb621d9bf22a3ec3b`, matching the
prior `8bd0da0` run because production changes are UI-only. UI custody therefore
uses the source/qualifier/config and actual native lifecycle receipts rather than
inferring a UI version from the binary digest alone.

## Actual execution and preservation

The existing fresh public-service fixture and native authoring setup establish
saved manual B, an unrelated unsaved F draft and a pending targeted proposal.
Three production-client localhost HTTP/SSE replies for generation/recap/preview
are labelled synthetic (`real_model:false`). Recall executes no model. No real
model narrative-quality claim or acceptance claim follows.

Native checks establish:

1. With two inspector hosts and actual recalled revision222, switch away from
   Bible: one host remains and revision222 stays displayed, without invalidation.
   Switch back: both hosts remain on the same anchor and evidence persists.
2. Hold a real 1000ms recall completion at revision222. Dispose the final
   production components and remount the same anchor. Release actual success:
   no projection, pending state, error or invented fact-change notice returns.
3. Repeat with a real unspecified-time read at revision222, releasing a labelled
   synthetic completion error. The remounted inspector remains clean.
4. Explicitly recall, then use the ordinary Bible field Save to change exactly
   **“Mara's umbrella is amber.” → “Mara's umbrella is copper.”**. The old owned
   fact revision `46d1fb8a-b170-4595-b5bd-c228ca6687a2` becomes
   `1eb58322-f514-4abb-966a-61a12eab354b`. Actual final disposal/remount retains
   the genuine notice and withholds old evidence.
5. Explicit 1000ms recall displays copper plus the real Rain assertion at
   revision223 and clears the genuine notice. This read writes no history.

The saved manual B text remains exactly:

```text
EXT. STATION - NIGHT

Mara keeps the witness hidden. Preserve this station beat.

```

Its block is `script.block.61ea91cb-05e7-43cb-983c-ebe9e6baea1a.generated`, saved
revision `8cda6f97-864c-4d4b-9084-da6178235558`. Saved scene text/revisions/placement
are compared after each lifecycle case. Exact F draft
`Unrelated draft: Eli pockets a brass whistle.\n\n` stays visible and unsaved.
Pending proposal `script.review.aa3e1650-a557-40b4-81bf-ecdccd77fee5` retains every
database column and status throughout. Its amber synthetic preview predates the
copper edit; preserving it does not qualify acceptance against changed facts.
No Accept action is clicked.

## Original artifact and individual visual inspection

[Artifact 11461888724](https://github.com/MrScripty/Eidetic/actions/runs/37573408905/artifacts/11461888724),
`eidetic-bible-recall-lifecycle-native-4c468d6`, is1,200,498bytes and expires
10 October2026 at05:04:13UTC. Downloaded file
`file_0000000000248230b9ea3f8a7633bd81`; untouched ZIP SHA256 is
`6fd0edf201c76c46c6fff85513c09dccadb8d82665725271464daae87478fb51`, matching
GitHub's digest. A first download disconnected without returning bytes; the
ordinary retry succeeded. No qualification run was repeated.

All six originals were individually viewed at original detail. Each is1920×1440;
PNG hashes match capture metadata and every extracted member matches original
ZIP bytes. Files remain in
`/workspace/scratch/bible-recall-lifecycle-native-success-37573408905/`.

| Original capture | Observed native state | SHA256 |
| --- | --- | --- |
| `eidetic-bible-recall-pending-review.png` | Actual amber/Rain revision222, manual B, F draft and synthetic proposal with explicit acceptance controls; labelled QA host. | `6b6e07a41faee7718937a7a56b05384f9070199ea03b7ccfdef82971d2c4e46c` |
| `eidetic-bible-recall-lifecycle-concurrent.png` | Both inspector hosts after ordinary sidebar remount, unchanged revision222 and no false notice. | `a1b15629d5430284ecf4b3f8aab6f2612e72080dfc1a58c83c57062445f2ef9f` |
| `eidetic-bible-recall-lifecycle-delayed-success.png` | Same-anchor remount after final disposal and released actual success: empty evidence, enabled recall, no false notice. | `38824c184fe5b7dce92425d45c6ea5bc1806f3101b9383aee76464d3bfb0231e` |
| `eidetic-bible-recall-lifecycle-delayed-error.png` | Released labelled synthetic error cannot publish error/evidence or a mutation notice. | `2f6376e9596b27b2e989bc43ca686e7d9894e1d0b20d87c4afb1e9860978f712` |
| `eidetic-bible-recall-lifecycle-genuine-invalidation.png` | Copper canonical field and genuine Facts changed notice persist after actual component disposal/remount. | `da8926883d9914a5ee696d0d081c760086360b0a9f4e1d0a190f7e9b7830c0a9` |
| `eidetic-bible-recall-lifecycle-fresh.png` | Explicit current copper/Rain evidence at revision223; genuine notice cleared. | `28d083fcca4e587604244482e89972922ecf462a9859ee865ab1d68cc8581459` |

Bible, timeline scene tracks, saved manual B, F draft and pending review are visible
in every capture. The QA panel occupies the lower-right timeline corner without
covering the A/F/B clips. Direction/kind selector text remains faint against light
backgrounds, the already recorded visual limitation. No binding or lifecycle
defect was found in the originals; polished selector contrast is not claimed.

## Separate evidence custody

`bible-recall-lifecycle-manifest.json` seals full raw native metadata, official
run/job/artifact metadata, original ZIP/files, source and QA deltas, local gates,
served-module admission and limitations. Manifest SHA256:
`a467d437f39c7621e0324db0f48be29821a87a1e7d90811ae058b2457972a42c`.

The original application `8bd0da0`, qualifier `cf9830b6` and evidence
`12a3fa980dd01a389cc11d1c22b7c23cde8092c8` remain unchanged. The original manifest
retains SHA256 `45663c0118f1111d71e83e5a45b2847e89810a94975d05f8c3fe55eb0bbae60b`.
These originals are not relabelled as successor qualification. PR19 still points
to `8bd0da0`; no PR update, merge, manual thread resolution or CodeRabbit request
is part of this evidence milestone. Parent retains review/merge/Library delivery.
