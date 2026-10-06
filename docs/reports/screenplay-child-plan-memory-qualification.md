# Manual screenplay memory in child timeline planning

## Source and missing criterion

Preserved parent checkpoints b0d6d17c and native a0b46fed are unchanged. New
implementation is `122c71e123ee9ff34f27c5ad9a2f34cf4bc5bcd6`, tree
`9796132f854ed298c27fd431e1ac1cccb8182ccd`, branch
`feat/screenplay-child-plan-memory`. This descends from verified PR9 successor
`25b860d12fe37a3538a40616cc02ab4a81370863`, tree
`d22b036891aed7eb66fc94059aa1b5c1d9aab85a`. Parent owns PR integration and delivery.

The actual public manual screenplay create/edit services followed by child prompt
attachment reproduced `saved_manual_text_present_in_child_prompt=false` on the
baseline. GenerateChildren had no canonical script inputs; BeatEditor immediately
applied generated children. This misses the existing manual-memory read/projection
and timeline planning criterion. The baseline assertion/log and test source are
preserved under `/workspace/scratch/child-plan-memory-audit/`.

## Implementation

Child generation consumes the existing main-document bounded screenplay selector
and records exact text/revisions, placement epoch and parent/subtree versions in
the existing creation-command JSON. Known empty and legacy unknown remain distinct.
There is no new schema, parallel memory canon or SDK dependency. The existing
ai_service public facade delegates to a focused child generation owner.

Non-leaf clips, including scenes, can preview child proposals and exact saved
evidence. Only explicit **Accept timeline plan** calls the existing timeline
command. The same writer transaction validates the durable reviewed material and
consumed receipt before changing children/status. Replay remains first. Source
text ABA, relevant membership/notes changes and forged proposals refuse with
rollback; distant unselected text edits remain compatible. Saved screenplay is
never rewritten by child acceptance; existing manual draft owners are unchanged.

## Exact local execution

On implementation 122c71e1: **432 full server, 119 core, 423 frontend tests**;
strict normal server all-target Clippy, frontend typecheck/build and traceability
pass. Ten focused child memory/store tests and twelve UI controller/render tests
exercise public services, actual synthetic HTTP JSON adapter, delayed and pending
stale guards, material matching, command replay, immutable retry custody and
explicit acceptance. Logs are in `/workspace/scratch/child-plan-memory-audit/`.

Rust ran locked/offline with writable XDG isolation and the verified Pumas
`a94fd92021f27fdeedb6e2de6e01c41c250ef576` sibling. ONNX download and SDK override
flags were unset. Normal precommit checks passed. Push excluded only the local
full desktop workspace test hook, because this container lacks GTK/WebKit; hosted
native qualification below runs the actual desktop build and service suites.

The exact local gates were:

```sh
cargo test --offline --locked -p eidetic-server --lib
cargo test --offline --locked -p eidetic-core
cargo clippy --offline --locked -p eidetic-server --all-targets -- -D warnings
npm --prefix ui test
npm --prefix ui run check
npm --prefix ui run build
scripts/check-decision-traceability.sh
python3 -m unittest discover -s scripts -p 'test_qualify_*.py' -v
python3 -m unittest discover -s scripts -p 'test_check_onnx_no_download.py' -v
```

Sixteen Python HTTP/native-driver admission tests pass. These validate fixture
admission only and do not qualify GUI behavior or model quality.

## Native qualification

Separate workflow `child-plan-memory-native.yml` binds the exact application SHA
above and permits only qualification/docs changes. It uses standard hosted GTK,
WebKit, AT-SPI/X11 and the actual desktop executable with the production HTTP
provider boundary. It repeats native Bible red-to-blue fact Save, preserved draft,
targeted preview and explicit screenplay acceptance, then manually saves screenplay
train departure text from midnight to morning, reviews a pending child plan,
refuses stale acceptance and explicitly accepts a fresh plan. Captures and receipt
are verified on final qualification
`9b4c5ff563f9662489ceac3ea321c5f68fbc677a`, tree
`4608cdf8e6d5055ca954601202567f2eac2830ad`.

First run `37384803651`, job `112015332687`, passed normal native builds,
no-download admission, 119 core and 432 full-server tests. It failed during native
project opening: retired WebKit accessibility enumeration returned a negative child
count (`ValueError: __len__() should return >= 0`). This run made no model calls
and does not qualify the feature UI. The verified failed ZIP is artifact
`11378196235`, SHA256
`73b43c05733564d93bceb2e3112c278d7cd92c3f3b6c80f75f635988d6b0e219`, preserved
with the job log under `/workspace/scratch/child-plan-memory-audit/`.
Qualification-only corrections locate rendered paragraphs by an excerpt while
checking full exact draft/saved text, and reacquire only that exact retired-object
failure inside the maintained bounded poll. Other ValueErrors still fail visibly.
Application source stays unchanged. Second run `37385874471`, job `112018899012`, again passes the normal native
build, no-download gate and 119/432 full service tests. The actual native Bible
red-to-blue Save, draft/saved text preservation, targeted preview and explicit
screenplay acceptance all complete. Exact manual screenplay Save adds "The train
leaves at midnight." The production HTTP child request admits both exact saved
blocks and the blue Bible fact, records a pending plan, and the untouched failure
screenshot visibly shows the proposed Midnight departure and downstream screenplay
Needs review. The overall native run fails its broad text locator (a hidden ancestor
matched before the visible proposal), so stale child refusal/fresh acceptance remain
unqualified on this run. Artifact `11379746698` verified ZIP SHA256
`a1a3f64592ce7a1bfd3225f397c22f76f2b6d40ae839ae5476bf69e32f935d0e` is preserved
under `native-37385874471/` alongside the job log. Qualification-only correction
matches the exact proposal name and stale error's alert role. Application source
is still unchanged. Third run `37387509611`, job `112024368234`, again passes build/service gates
and reaches the same pending proposal. The exact inline-name locator also fails:
WebKit exposes the strong inline name within the list item's text range. Its
verified artifact `11380430006`, SHA256
`a79cf120716b49d7f01755a5423deb3eef279367eb54b1bc066e307baeb6393d`, is preserved.
The two proposal failure captures have identical image digest
`ea61ffe9481d02dd05efa09a37782165a7a419a15a26fa01a3e6583750d5bf25`.
The next driver scopes text reads to the named review section, reusing the
maintained screenplay-section pattern, and records that native accessibility
subtree. Application source remains unchanged. Fourth run `37388944475`, job `112029138626`, passes all build/service gates,
then qualifies native pending child review with unchanged canonical children and
screenplay. Its native accessibility receipt confirms the proposal name is part
of the list-item text range. Exact native Save changes the departure to morning;
actual Accept timeline plan visibly returns the expected context-changed refusal.
The role-specific alert locator fails even though the full exact refusal is visible
in the untouched screenshot. Artifact `11380357837`, verified ZIP SHA256
`c2a239e3d1fb90a6349073ce455813aff1fcccd528cdfef2c6c79c092e36f589`, is preserved
under `native-37388944475/`. Final correction reads the exact visible refusal text
using the maintained native find helper. Application source remains unchanged.
Fifth run `37390511975`, job `112034199344`, passes the complete native input /
canonical service sequence: real Bible Save and screenplay preview/acceptance,
exact midnight and morning screenplay edits, pending child review, refused stale
acceptance, fresh model request and explicit canonical timeline acceptance.
All five production HTTP fixture requests are admitted with exact current inputs;
all carry real_model=false. Saved screenplay bytes/revisions and the blue fact
remain unchanged by child acceptance. Artifact `11380893805`, verified ZIP SHA256
`5951478ad104bd520be0046549eccebe1ff3e0ca64a6eee8d89cf1fda6692766`, is preserved.
The final PNG caught the frontend at Accepting before its projection refresh;
this run proves canonical acceptance but does not qualify the settled final UI.
The final driver now waits for review removal, enabled Replan Beats, and native
Morning departure timeline geometry before taking the accepted image. No
application code changes. Final run `37391688210`, job `112037996504`, **passes** at qualification
`9b4c5ff563f9662489ceac3ea321c5f68fbc677a`, tree
`4608cdf8e6d5055ca954601202567f2eac2830ad`. The unchanged application is 122c71e1.
Normal desktop/fixture builds, all-features no-download admission, 4 gate tests,
16 driver tests, 119 core and 432 server tests all pass. The real Tauri native
walkthrough completes Bible red-to-blue Save, retained draft/saved text,
review/preview/explicit screenplay acceptance, exact midnight-to-morning Save,
pending child review, refused stale acceptance, fresh pending plan and explicit
canonical timeline acceptance. The final capture waits for UI acknowledgement,
review removal and the actual Morning departure Beats-track text geometry
`(212, 883, 108, 11)`. The writer's saved screenplay bytes/revisions and blue fact
remain unchanged by timeline acceptance. The existing screenplay Needs review
remains because timeline acceptance does not rewrite its stale saved material.

Artifact `11381572325`, verified ZIP SHA256
`37d6fb515b25d3ba4a14fb82a337d96adf1d9bc50b3cdbcd73aa3f6e119949cd`, is copied to
`/workspace/scratch/child-plan-memory-audit/native-37391688210/`.
All six PNGs were hash verified and viewed without modification. The native app
binary SHA256 is
`6641c47e1f08620737d3d84217082d7bea2b30d47d8d69c0affe0cf0a2c07a1e`.
The receipt binds the exact application/qualification sources and all five actual
provider requests; every request admits exact current context and is labelled
real_model=false. These are synthetic HTTP/SSE mechanics, not real-model quality.

The 1440x960 frame shows Bible, timeline and screenplay together. Existing
horizontal screenplay overflow clips portions of text; exact native typed text
and canonical byte/revision observations are in the receipt. The saved-evidence
details remain collapsed. No context-stack inspector GUI claim follows from these
captures. All model responses are explicitly synthetic.

## Final untouched artifact hashes

| File | SHA256 |
| --- | --- |
| app-sanitized.log | `ff20ea9ee6286c33901f89cdb635aadd6e57e46486d829f77650903851684938` |
| capture-evidence.json | `2501ba19c2f0b4d9f77aef23f0514bdd2b025765f2243958fa71bbe63f2ec15e` |
| eidetic-bible-fact-accepted.png | `5d43e50c5b35a4cb34ed2ff50d72f70344f6e1c507604c466c968c5b7acddfeb` |
| eidetic-bible-fact-preview.png | `8cccf43d5ff87e09e45533e6bd4a212d38db35ea44485672c0b4ac174e570e63` |
| eidetic-bible-fact-review.png | `0ccf59b0023e8a5a7dada99bccc3882bccf143aa95856ea48bae1e185ce94fb5` |
| eidetic-child-plan-accepted.png | `68e06fcaf1da6f8f8d2f5eade8874b29337dc6baa749c8009c5391667af1f618` |
| eidetic-child-plan-pending.png | `aac7c52786c7f242ce9381e21bd0c2ac95b23c35cd689fd95f4b5ecbb0396e98` |
| eidetic-child-plan-refused.png | `50beb0a50b01e0ef9623f19998a054aa934f2801a389c6fb234f900320b27f5f` |

Native job log: `a63281abfef659c123f4bdb2072a0382bc13201f7c5e86f048f2a3cdd08e2d9e`.

## Bounds

This slice binds canonical screenplay and parent/subtree planning state; complete
Bible/affect/arc-description receipt binding remains a later criterion. Durable
pending-review UI recovery, project switching, automatic story fact extraction,
SDK inference, live Pumas runtime, context-stack inspector GUI and real-model
quality are unqualified. Existing viewport constraints will be reported alongside
native captures. Original review/native branches remain frozen.
