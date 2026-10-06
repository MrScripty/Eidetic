# Story-memory integration readiness

Snapshot: 6 October 2026 UTC. This consolidates the completed line against fetched
`origin/main`; it adds no application behavior. The mechanically smallest route
is **one normal merge PR** from the repaired descendant plus its evidence and this report
into main. Main ancestry is already present. Independent review subsequently
identified two P2 child-plan defects in the preserved candidate; both now have
separate repair checkpoints and require repair-source acceptance. Source-review receipts and ordinary
exact-head PR CI still need to be closed before calling the whole line merge ready.
Parent owns PR creation, review, merge, screenshot acceptance and Library delivery.

## Exact frozen source and ancestry

| Role | Commit | Tree |
| --- | --- | --- |
| Current merged main, PR10 | `302851dbb5bf67cda922b4d79f70623e444891dc` | `becb86624608663a9e91ff629edd1080d6eada54` |
| Requested PR9 successor, already ancestral | `25b860d12fe37a3538a40616cc02ab4a81370863` | `d22b036891aed7eb66fc94059aa1b5c1d9aab85a` |
| Preserved tested application freeze | `122c71e123ee9ff34f27c5ad9a2f34cf4bc5bcd6` | `9796132f854ed298c27fd431e1ac1cccb8182ccd` |
| Preserved native qualification | `9b4c5ff563f9662489ceac3ea321c5f68fbc677a` | `4608cdf8e6d5055ca954601202567f2eac2830ad` |
| Preserved pre-repair candidate/evidence | `85bed2854a11b805b91c53b934f9fe75b62764f6` | `2571e92e113981b64806ab02f5616f4ab7b8d1a9` |
| Preserved prior service/native report | `b0d6d17c0d776f1c46e39c2590cfd2f944768412` | `f21aabb922f9b213c1bbd22810d033280ada2d20` |
| Preserved prior native qualification | `a0b46fedfdef03901ecd031f43c4bf7636627ce1` | `2a689f3b23506254655c6164a0c91ff9177f4657` |

`git merge-base main 85bed285` is exactly main 302. Left/right count is **0 / 32**:
no missing main commits and 32 candidate commits. Existing ancestry merge
`5b057efdf038a5959630270dea5c749075307844` has main 302 as its second parent;
its tree equals its first parent's tree. It repaired ancestry without replacing
application source. PR9 successor 25 is also an ancestor. No new merge, rebase,
cherry-pick, squash or history rewrite is needed to prepare the normal PR.

Selected checkpoints in ancestry order are scene-order 9429/441c, canonical
creation 9e8b/8941, membershipc633/2761, repair 3ebb/8ea7, context 4aae/8d27,
dependency 1e4a, nativea0b4/reportb0d6, application 122c, qualification 9b4c and
report 85be, then normalization/refusal d5ae81a, relationship receipt 9c95a97 and
new native qualification 1d036b0, report e93814c and readable qualification 4fa35c6. The [machine-readable manifest](story-memory-integration-readiness-manifest.json)
contains all 37 candidate commit IDs/subjects, all 112 changed paths and exact
Git blob IDs, plus the preserved 32-commit/107-path inventory. The three requested heads 122c/9b4c/85be remain frozen.

## Application versus evidence review surface

The preserved 85bed285 main-relative diff is **107 files, 9,719 added / 838 removed lines**.
At repaired qualification 1d036b0 it is **108 files, 10,488 added / 838 removed lines**.
Readable qualification 4fa35c6 retains those source files, the published report
e93814c and supported capture controls: **112 files, 12,798 added / 838 removed
lines** (47 application source, 25 regression tests, 5 dependency files, 16 native
qualification files, 19 documentation files). The final report update adds
documentation only. This is
a substantial accumulated line despite a one-PR minimum; review it by bounded
source units rather than treating it as just the newest child-planning change.
The following role table describes the preserved 85bed285 inventory.
Classification is by each file's primary role; production files can contain inline
tests, and documentation includes API/README guidance.

| Primary role | Files | Scope |
| --- | ---: | --- |
| Application source | 47 | Core contracts, canonical generation, scene-window/Bible scopes, shared context/agent reads, child-plan receipt/acceptance, UI owners/events |
| Application regression tests | 25 | Actual service/SQL/harness tests plus UI controller, cache and SSR tests |
| Dependency/build admission | 5 | Cargo.lock, Pumas preparation pin, ordinary CI metadata gate, gate script and four gate tests |
| Native qualification infrastructure | 14 | Four source-bound workflows, two setup examples, native AT-SPI/X11/HTTP drivers and driver tests |
| Documentation | 16 | Plans, ledger, seven qualification/repair reports, core/server/UI READMEs |

All **77 application/test/dependency files** have identical blob IDs at 122c,
9b4c and 85be. All changes after the application freeze are these eight paths:

```text
.github/workflows/child-plan-memory-native.yml
docs/plans/agent-story-workflows/execution-ledger.md
docs/plans/agent-story-workflows/plan.md
docs/reports/screenplay-child-plan-memory-qualification.md
scripts/qualify-bible-fact-propagation.py
scripts/qualify-child-plan-memory.py
scripts/qualify-screenplay-authoring.py
scripts/test_qualify_child_plan_memory.py
```

These later changes qualify the real native UI and record evidence. The shared
authoring-driver change retries only the exact observed retired-WebKit negative
child-count error, inside the existing bounded poll. Other errors remain visible.
They do not repair or alter the application's story-memory behavior.

Dependency1e4 is separate from application review: Pumas changes from
8444b50d to **a94fd92021f27fdeedb6e2de6e01c41c250ef576**, tree
`4a6977b88ed532089807183e218a959ac02b724d` (0.7.0). This is a substantial SDK
successor, not a tiny loader patch. Actual used APIs compiled unchanged; normal
and all-features resolution enforce dynamic loading/disabled linking without
ONNX download/copy features. No SDK was provisioned, no ONNX403 workaround was
used, and successful runs unset ORT_SKIP_DOWNLOAD/DOCS_RS/SDK path overrides.

## Verified review checkpoints and remaining source review

Repository merge records confirm [PR9](https://github.com/MrScripty/Eidetic/pull/9)
at successor 25 and [PR10](https://github.com/MrScripty/Eidetic/pull/10) at af603,
with main 302 the latter's merge. PR10's frozen description records independent
bounded acceptance of its Bible feature/native qualification. This is evidence
for the merged baseline. The API returns no PR10 review submissions; PR9 has a
COMMENTED bot review 5414565661 on older 580da245, not a final-head APPROVED review.
Do not conflate recorded parent/peer acceptance, GitHub review states and test passes.

| Completed checkpoint supported by available evidence | Exact bounded scope / provenance |
| --- | --- |
| Accepted authoring composition | Parent acceptance of a84dfc21 placement repair and e8dbfffa feature chain, with 12/17/7 independent tests, recorded in the execution ledger's Accepted authoring composition candidate; ancestry retained in PR9 |
| Merged consumed-Bible propagation | PR10 frozen description records independent source/native acceptance at application fe7fa590 / review af603e76; merged tree is current main |
| Scene-order predecessor accepted | Ledger's Immediate canonical scene generation successor records peer acceptance of its predecessor application/native evidence (9429daa / 441c2a), while narrowing earlier HTTP admission claims |
| Bounded 201-node preview-value repair accepted | Parent-provided session re-review acceptance of repaired 3ebb752 / report 8ea7153; scope is missing entered values, restoration and stale/ABA refusal. No standalone acceptance receipt was found in this checkout |

No new PR exists for this line. Parent supplied a completed independent
final-source planning review of 122c71e identifying two P2 defects; that is a
review checkpoint requiring repairs, not acceptance of the repaired successor. Available repository/session evidence
does **not confirm completed independent final-source acceptance** for the whole
canonical-creation/membership source delta, context-stack 4aae/8d27, dependency 1e4,
or the repaired child-planning source 9c95a97. Separate repair-source acceptance
is not evidenced here. The 201-node repair acceptance does not establish acceptance
of every other changed file. If parent reviews completed privately, attach their
exact frozen SHA/scope/receipt to the integration review; absent receipts here are
not evidence those reviews never happened.

Remaining source-review units are therefore:

- Canonical completion/target guards and scoped Bible/scene-window changes beyond
  the accepted merged baseline and bounded repair; verify preserved invariants.
- Shared context-stack and structured agent-read bridge, including clock/event
  refresh, legacy unknown versus known-empty and historical tool-result custody.
- Pumas 0.7 dependency/lock/feature-union admission compatibility. Native HTTP
  execution does not establish SDK inference, packaging or Windows runtime loading.
- Child planning's exact saved-script/parent/subtree receipts, durable material
  matching, writer-transaction admission, replay and uncertain-ack UI custody.

## Narrow repair checkpoints after independent review

| Repair | Exact source / tree | Actual evidence |
| --- | --- | --- |
| Proposal normalization and definite refusal recovery | `d5ae81a6c0cb8c4a1c2184da4aa1e8fd2b398374` / `8eea39bfdedd03ea7c921a89ad431a2e8ce11cd3` | Three actual HTTP/AppState/SQLite regressions fail on 122c71e, then pass; full server 435/frontend 429, controller plus SSR 18, strict server Clippy |
| Relationship receipt custody | `9c95a9714e8be5c2be71fd62ff049d71092ad8b3` / `ec16af1d5eb36dc75155968224dd347d9e63c1f7` | Actual public acceptance wrongly records a newly added outside-to-child edge on d5ae81a; repaired full server 442 passes, strict server Clippy |
| Fresh native qualification infrastructure | `1d036b0091b45595f94ed84cf91a7e4d6dd9fa55` / `1d948840a5d1803b151346bd8d4c269072f42b68` | Only a new source-bound workflow and synthetic provider/driver tests differ from repaired application; successful native execution receipt follows below |

Storage already canonicalizes proposed names, outlines, locations and references.
Generation now returns those exact durable children from the existing projection,
so review and acceptance see the same material. It introduces no second normalizer.
Acceptance still refuses any changed material, even an added outline newline.
Only the two exact writer-transaction refusal messages with native `conflict`
provenance release UI retry custody; lookalike strings, internal failures and
other conflicts remain uncertain and retry the identical command/payload.

Relationships touching replaced descendants include incoming/outgoing external
edges and internal edges. Existing creation-command JSON now binds canonical edge
state plus latest committed-order revision receipts, including deleted identities
for add/delete ABA. Transaction recapture excludes only acceptance's in-flight
event. Old absent receipts remain unknown and require fresh review. Reviewed edges
can be removed by explicit acceptance; unrelated and parent-only edges remain.
Seven new service tests preserve complete timeline, exact screenplay, pending
proposal and four history-table counts on refusal, and exercise replay. Every
public edge command uses timestamp zero, proving committed ordering rather than
timestamp order. No native edge-drawing walkthrough is claimed.

## Behaviors coexist in the same final application

| Behavior | Evidence and limits (fresh repaired native run plus full service suite) |
| --- | --- |
| Manual screenplay and explicit replacement | Actual full-server manual workflow reads exact saved edits with a stale legacy mirror; native runs edits through real controls. Bible preview and child preview preserve saved canon; only explicit screenplay acceptance replaces its target |
| Timeline placement / scene-window continuity | Repaired full suite executes the public scene-order command/review/acceptance and window/ABA tests. Historical native 37354081654 visibly qualifies E entering/A leaving at its frozen predecessor; no new drag/resize walkthrough is claimed for122c |
| Bible memory / affected scene review | Repaired full suite includes scoped membership, 201-node missing-value refusal/restoration, stale/ABA and lineage tests. Native6 edits the exact red tagline to blue, retains saved text and a 41-character draft, shows downstream review, previews and explicitly accepts only B |
| Saved screenplay in context and agent reads | Repaired full suite executes five shared projection tests, two actual structured agent-loop/history tests and the public AppState manual/context read. Repaired429 UI tests include clock/event/late-read guards and graph-detail SSR. Inspector GUI and real-model interpretation remain unqualified |
| Manual screenplay affecting downstream timeline material | Native6 saves exact 80-character midnight text, reviews pending children, then saves 83-character morning text. Old acceptance refuses without changing children/status/text. A fresh request consumes current inputs; explicit acceptance creates Morning departure, clears child review and renders its Beat clip |

This is one compiled and tested application tree, not an inferred composition of
separate branches. Existing shared SQLite commands/history, revision/provenance,
main-document selector and proposal mechanisms remain the owners. No parallel
memory store or embedding dependency was introduced. Accepted timeline children
do not rewrite the saved screenplay, so its existing Needs review notice correctly
remains after the manual source change.

## Actual execution and visual evidence

Final [readable run 37396627562](https://github.com/MrScripty/Eidetic/actions/runs/37396627562)
passes at qualification **4fa35c6**, source **9c95a97**: **119 core / 442 server /
18 driver**, four no-download gate tests and normal native builds. The seven
untouched, individually inspected PNGs include the [readable writing-area overview](/workspace/scratch/story-memory-readiness/native-37396627562/eidetic-story-memory-readable.png):
full exact saved morning screenplay, previously accepted B screenplay, blue Bible
fact, accepted Morning departure beat and remaining Needs review are visible in
one frame. Only existing panel resize, focus and native scroll were used; source,
saved text and timeline acceptance remain unchanged. Earlier six-frame captures
retain their own clipping bounds. The qualification report records actual glyph
bounds and every image hash. ZIP artifact **11384360136** SHA256
`2654969cc230ebfa796b426a2b6b4da5a3bc65e3031164c69f535a3692d22386`; receipt SHA256
`a2bd27d6f729e305c80e169431a4af48cb916ef964cd168cb4017a5ba5c5dae7`. Application binary remains
`71d711be42f3351e7fcd2a6628b1ed2fe4bb62f2fea3a2cf6295a8b091eaccde`. All model responses are synthetic.

Repaired source 9c95a97 has **442 full-server / 429 frontend** tests passing
locally and strict server all-target Clippy, with the same UI source qualified
by the normalization checkpoint. Fresh [native run 37395200220](https://github.com/MrScripty/Eidetic/actions/runs/37395200220)
passes at qualification 1d036b0: normal actual desktop/public-fixture builds,
**119 core / 442 full-server**, 16 driver and four no-download gate tests.
The same Bible/manual screenplay flow now consumes raw padded names, outline
newlines, empty locations and padded/blank references, recovers after definite
stale refusal and explicitly accepts the fresh canonical children. Final UI
clears review and displays Morning departure at `(212,883,108,11)`.

Artifact 11382778877 ZIP SHA256 is
`86ecc1fc912244751406a03044f8c13f65ebf33c4ab6ed74f0c9a2ca1776f209`.
Receipt SHA256 is
`e0e89fc7e3090043def001c2bb283c6eb7a3c5013b067f78fb75ea507999387e`.
Binary SHA256 is
`71d711be42f3351e7fcd2a6628b1ed2fe4bb62f2fea3a2cf6295a8b091eaccde`.
All six original PNGs were hash verified and individually viewed; parent image
acceptance is separate. The [repair qualification report](child-plan-review-repairs-qualification.md)
links every image/hash and states exact fixture, clipping and coverage limits.
Relationship drawing remains service-qualified rather than native GUI-qualified;
inspector GUI, SDK inference and live-model quality remain unqualified.

Preserved application 122c has **432 full-server / 119 core / 423 frontend** tests
passing locally, normal strict server all-target Clippy, frontend typecheck/build,
traceability and normal precommit checks. The ordinary full-desktop pre-push hook
was excluded only locally because GTK/WebKit is unavailable; hosted builds below
use standard native dependencies. Older module-only counts/compile-only Clippy
remain historical, not substitutes for these executed suites.

[Final native run 37391688210](https://github.com/MrScripty/Eidetic/actions/runs/37391688210),
job 112037996504, passes at qualification 9b4c with application guard122c:
normal desktop and public-fixture builds, 119 core/432 full-server tests,
16 driver tests, four no-download gate tests and the complete real Tauri flow.
Its final capture waits for review removal, enabled Replan Beats and actual native
Morning departure timeline geometry `(212,883,108,11)`.

Artifact 11381572325 ZIP SHA256 is
`37d6fb515b25d3ba4a14fb82a337d96adf1d9bc50b3cdbcd73aa3f6e119949cd`.
Receipt SHA256 is
`2501ba19c2f0b4d9f77aef23f0514bdd2b025765f2243958fa71bbe63f2ec15e`.
Native binary SHA256 is
`6641c47e1f08620737d3d84217082d7bea2b30d47d8d69c0affe0cf0a2c07a1e`.
Six untouched PNGs and read-only observations are preserved in
`/workspace/scratch/child-plan-memory-audit/native-37391688210/`; parent screenshot
acceptance remains separate from the author's hash verification/viewing.
The final [qualification report](screenplay-child-plan-memory-qualification.md)
lists all image hashes and preserves four locator failures plus the earlier
canonical-pass screenshot taken before frontend acknowledgement.

Prior [native 37379727832](https://github.com/MrScripty/Eidetic/actions/runs/37379727832)
at preserveda0b46fed executes425 server/119 core and the Bible native flow on
unchanged application 8d2719d with the separately admitted dependency. It closes
the old AppState execution gap for the context bridge; it does not show the
context-stack inspector GUI. Its frozen b0d6 report and artifact remain preserved.

All five final provider requests are predefined **synthetic localhost HTTP/SSE**
responses through the production client, labelled real_model=false. This qualifies
transport, canonical memory, UI/command and acceptance mechanics. It does not
qualify real-model story quality, SDK inference or live Pumas runtime. The 1440x960
frames include Bible, timeline and screenplay; existing horizontal script overflow
clips portions of text, with exact typed bytes/revisions verified independently.

## Smallest normal integration route

1. Parent uses a new integration branch at this report's repaired descendant of 85bed285,
   preserving the frozen 122c71e/9b4c5ff/85bed285 refs and existing history.
   At qualification 4fa35c6 main is 0 commits ahead / candidate 37 ahead; the
   final documentation checkpoint adds one commit. Main302 is already ancestral.
2. Close the bounded source-review units above and attach any completed private
   receipts. Parent inspects all seven readable-run images, including the unretouched
   readable overview, with their explicit limits. Earlier captures remain preserved.
3. Open one normal PR to main. Its ordinary CI must pass on the exact PR head:
   Linux/Windows `cargo clippy --workspace --all-targets --all-features`,
   `cargo test --workspace --all-targets`, format, frontend lint/format/typecheck/
   tests and traceability. The Linux native job does not replace that matrix.
4. Parent performs a normal merge that retains existing commits. If main advances,
   merge current main into that new branch, inspect the actual delta and repeat
   affected/exact-head checks. Avoid rewriting the three preserved checkpoints.

The two documented review repairs are included; no extra main merge is necessary
at this snapshot. Repair-source acceptance and ordinary PR CI remain gates.
A bounded stack (8ea7,8d27,b0d6, then the repaired final candidate) can reduce each review diff if parent wants
separate review units; it requires more PRs and is not the smallest mechanical
route. Do not cherry-pick isolated source commits and omit their ancestor guards,
dependency admission or source-bound evidence.

The remaining product criteria are explicit bounds rather than new work in this
consolidation: complete Bible/affect/arc-description binding for child proposals,
durable pending-review UI recovery, timed facts/relationships/unproven relevance,
feature-length interruption/recovery coverage and live model/SDK quality.
Project-switch recovery stays deferred; embeddings stay optional.
