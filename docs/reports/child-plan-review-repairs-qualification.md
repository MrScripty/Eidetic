# Child-plan review repair qualification

Two independently identified P2 defects in preserved source
`122c71e123ee9ff34f27c5ad9a2f34cf4bc5bcd6` are repaired in separate tested
milestones. Source/native/report checkpoints 122c71e / 9b4c5ff / 85bed285 remain
unchanged. These are acceptance correctness repairs, with no new authoring feature,
schema, SDK pin, model dependency or screenplay write.

| Role | Exact commit | Tree |
| --- | --- | --- |
| Normalization/refusal repair | `d5ae81a6c0cb8c4a1c2184da4aa1e8fd2b398374` | `8eea39bfdedd03ea7c921a89ad431a2e8ce11cd3` |
| Relationship receipt repair / combined application | `9c95a9714e8be5c2be71fd62ff049d71092ad8b3` | `ec16af1d5eb36dc75155968224dd347d9e63c1f7` |
| Fresh native qualification | `1d036b0091b45595f94ed84cf91a7e4d6dd9fa55` | `1d948840a5d1803b151346bd8d4c269072f42b68` |

Only the new qualification workflow, synthetic driver and its tests differ from
combined application 9c95a97 at qualification 1d036b0. The readiness manifest
verifies all 77 application/test/dependency blobs match; report publication adds
documentation only. The separately admitted Pumas a94fd920 pin remains unchanged.

## Actual reproduction and source behavior

The new normalization regressions execute actual public HTTP generation,
AppState, SQLite storage and public timeline acceptance. All three fail with
production generation from 122c71e (exit 101), because returned children differ
from durable children: padded name/multiline outline, empty location and padded/
blank ordered references. The repaired service passes these cases, including
three location variants, filtered characters/props, duplicate reference ordering,
pending status, explicit acceptance and identical-command replay. Exact saved
screenplay and consumed evidence retain trailing newlines. An altered accepted
outline with just one added newline still fails; counts/status/timeline/script
remain unchanged. Acceptance has not been normalized or weakened.

Storage retains ownership of proposal canonicalization. Generation now loads its
recorded durable children from the existing projection for presentation instead
of returning raw provider strings. The frontend recognizes only the two known
writer-transaction refusal messages with native `conflict` provenance. Those
refusals permit Close and fresh generation with a new command identity. Lost
acknowledgements, raw lookalike messages, internal errors and other conflicts keep
immutable payload/command retry custody. An exact retry that receives a definite
refusal releases that custody. Tests use the actual Svelte controller and SSR;
no injection into the native app is used.

The relationship reproduction also executes actual public commands. On d5ae81a,
a plan reviewed before an outside-to-child edge was added incorrectly records
acceptance; its returned canonical projection has removed that edge. The new
relationship test fails on that successful result (exit 101). The successor
records relevant edge canon and existing history revision receipts inside the
existing creation-command JSON. It includes both external endpoint directions,
internal descendant edges and deleted identities, so add/delete and exact-state
delete/recreate ABA cannot disappear. Existing committed event ordering handles
all public command timestamps being zero. Writer-transaction recapture excludes
only this acceptance's in-flight revisions. Parent-only and unrelated edges are
preserved. Old missing edge receipts mean unknown and require fresh review.

Seven new full-service relationship cases cover boundary directions, internal
edges, both ABA forms, delayed generation, old receipt absence, explicit removal
of reviewed edges, unrelated-edge ABA preservation and replay. Refused commands
retain full canonical timeline, pending proposal/evidence, exact screenplay and
counts in commands/change_events/object_revisions/object_revision_fields. Native
relationship drawing and long-running scene hierarchies are outside this evidence.

## Local executed checks

- Normalization checkpoint: full server **435**, frontend **429**, child controller
  **16** plus SSR **2**; actual child-service **10** tests. Strict server all-target
  Clippy passes. All ordinary precommit lint/format/typecheck/rustfmt/traceability
  and conventional-message checks pass.
- Combined relationship checkpoint: full server **442**, including **17** child
  service tests (10 prior + seven new), strict server all-target Clippy passes.
  Application frontend is unchanged from the normalization checkpoint; 429 tests
  apply to the same UI source. Normal precommit checks pass again.
- Qualification driver: **16** tests pass, including raw padded-name/newline/
  empty-location/reference fixture assertions. Local test logs and their 15 SHA256
  hashes are in `/workspace/scratch/story-memory-readiness/repair-local-test-receipt.json`.

Normal Rust builds use the no-download dependency's own resolved features. No
ORT_SKIP_DOWNLOAD, SDK path, DOCS_RS or linking override bypasses ONNX403. The
local full-desktop pre-push test hook was excluded because GTK/WebKit is absent;
actual native builds/tests are hosted below. Original application and Pumas
worktrees are unchanged. No PR, merge, external reviewer request, Library upload,
credential change or paid service was performed.

## Fresh exact-source native execution

[Run 37395200220](https://github.com/MrScripty/Eidetic/actions/runs/37395200220),
job **112049404919**, passes at qualification **1d036b0** with application source
**9c95a97**: exact-source guard, no-download feature-union admission and four gate
tests, normal actual desktop/public-fixture builds, **119 core / 442 full-server**
tests, **16 driver** tests and the complete real Tauri walkthrough. The full job
log is `/workspace/scratch/story-memory-readiness/native-job-112049404919.log`.

The actual Bible field changes `Mara's umbrella is red.` to
`Mara's umbrella is blue.` while retaining the exact 41-character unsaved draft
and saved screenplay. Downstream review, targeted preview and explicit screenplay
acceptance execute through real controls. Exact 80-character midnight and
83-character morning screenplay saves then exercise child planning. Both
synthetic raw proposals contain padded names, trailing outline newlines, empty
locations and padded/blank references. Read-only canonical receipts verify
trimmed names/outlines, null location, ordered `Mara` / `Umbrella` references and
unchanged saved inputs. A stale child acceptance visibly refuses; fresh generation
recovers and explicit fresh acceptance succeeds. The final capture waits for
review removal, enabled Replan Beats and the actual native Morning departure
label at `(212,883,108,11)`. Existing screenplay Needs review correctly remains.

All five provider requests are predefined **synthetic localhost HTTP/SSE**,
`real_model=false`, through production transport. All six PNGs were hash verified
and individually viewed in their original 1440x960 form. Bible, script and timeline
are visible. Existing horizontal script overflow clips portions of saved text;
full exact typed bytes/revisions are verified separately in the native receipt.
Saved-evidence details remain collapsed, the context-stack inspector is unopened,
and no native edge-drawing walkthrough, SDK inference or real-model quality is
claimed. The Bible PNG pixel hashes match the prior unchanged UI; the fresh run's
source guard, different binary/PID, timestamps and receipt establish execution on
the repaired application. Parent image acceptance remains separate.

Artifact **11382778877**, size **617,697 bytes**, is verified against GitHub's ZIP
SHA256 `86ecc1fc912244751406a03044f8c13f65ebf33c4ab6ed74f0c9a2ca1776f209`.
Receipt SHA256 `e0e89fc7e3090043def001c2bb283c6eb7a3c5013b067f78fb75ea507999387e`.
Native binary SHA256 `71d711be42f3351e7fcd2a6628b1ed2fe4bb62f2fea3a2cf6295a8b091eaccde`.
Extracted originals and verification receipt are in
`/workspace/scratch/story-memory-readiness/native-37395200220/`.

| Untouched native image | SHA256 |
| --- | --- |
| [eidetic-bible-fact-review.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-bible-fact-review.png) | `0ccf59b0023e8a5a7dada99bccc3882bccf143aa95856ea48bae1e185ce94fb5` |
| [eidetic-bible-fact-preview.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-bible-fact-preview.png) | `8cccf43d5ff87e09e45533e6bd4a212d38db35ea44485672c0b4ac174e570e63` |
| [eidetic-bible-fact-accepted.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-bible-fact-accepted.png) | `5d43e50c5b35a4cb34ed2ff50d72f70344f6e1c507604c466c968c5b7acddfeb` |
| [eidetic-child-plan-pending.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-child-plan-pending.png) | `d67c306b9b8deccca96088576d10610d231ce384f2286db4373f0c27fb6d30b5` |
| [eidetic-child-plan-refused.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-child-plan-refused.png) | `4cc77308a8c0714c97179978bf146b202e90fdd49cd8a47b8d05ca7cde33e63b` |
| [eidetic-child-plan-accepted.png](/workspace/scratch/story-memory-readiness/native-37395200220/eidetic-child-plan-accepted.png) | `f9f7f25c089d52dbdd70012563b866d01700f5af7c46ba1355fd0b07241440e1` |

Sanitized app log SHA256 `66a81e2a34deab585b5e8c6ccd75bd0865d8b432570e513c3c293f69c25169a7`.


Parent owns independent repair-source acceptance, image review, normal PR CI,
integration and delivery. See the [readiness report](story-memory-integration-readiness.md)
for the accumulated source versus evidence review surface and smallest normal route.
