# Manual screenplay memory in child timeline planning

## Source and missing criterion

Preserved parent checkpoints b0d6d17c and native a0b46fed are unchanged. New
implementation is `122c71e123ee9ff34f27c5ad9a2f34cf4bc5bcd6`, tree
`9796132f854ed298c27fd431e1ac1cccb8182ccd`, branch
`feat/screenplay-child-plan-memory`. Parent owns PR integration and delivery.

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

Fourteen Python HTTP/native-driver admission tests pass. These validate fixture
admission only and do not qualify GUI behavior or model quality.

## Native qualification

Separate workflow `child-plan-memory-native.yml` binds the exact application SHA
above and permits only qualification/docs changes. It uses standard hosted GTK,
WebKit, AT-SPI/X11 and the actual desktop executable with the production HTTP
provider boundary. It repeats native Bible red-to-blue fact Save, preserved draft,
targeted preview and explicit screenplay acceptance, then manually saves screenplay
train departure text from midnight to morning, reviews a pending child plan,
refuses stale acceptance and explicitly accepts a fresh plan. Captures and receipt
are pending execution. All model responses are explicitly synthetic.

## Bounds

This slice binds canonical screenplay and parent/subtree planning state; complete
Bible/affect/arc-description receipt binding remains a later criterion. Durable
pending-review UI recovery, project switching, automatic story fact extraction,
SDK inference, live Pumas runtime, context-stack inspector GUI and real-model
quality are unqualified. Existing viewport constraints will be reported alongside
native captures. Original review/native branches remain frozen.
