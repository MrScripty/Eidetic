# Saved screenplay in timeline and agent context

**Result:** manual screenplay now reaches the existing timeline context stack and
agent graph-context read with exact text and revisions. Locally qualified through
production modules, the actual structured agent loop with a synthetic provider,
and frontend rendering/cache tests. New-source AppState/native UI and real-model
execution remain unqualified.

## Source and acceptance criterion

- Discovery/frozen review checkpoint `2761aacb8e70571729faf535d5cef407d01d36e8`,
  tree `c268d4937657b0323d521470503674e1aef6896f`, remains unchanged.
- Chosen criterion, reported before implementation: extend the plan's manual
  **Memory read / Projection propagation** acceptance into the existing timeline
  context stack and inspectable graph-context runs (objective acceptance 4).
- Work was parked to prioritize the independent membership node-limit P2 repair,
  then resumed after its tested/pushed checkpoint
  `8ea7153b2c0f236c85ec47c0422e4f8e58a078c3`, tree
  `be8155a48c6b7718b80fd080f8b3fdca10fdbac3`.
- Separate branch `feat/screenplay-context-stack`.
- Implementation `4aae549c04924ad2580be86c142c017a960f5a00`, tree
  `c5e296f584e435f9e0cf916532ecb6e3dc19a73e`.
- Parent owns PRs, reviews, merges and Library delivery. The separately reviewable
  [P2 repair](bible-membership-preview-limit-repair.md) is an ancestor, not a change
  to the frozen Bible membership checkpoint.

## Verified gap and implementation

The existing scene-generation and targeted review paths already consume exact
canonical screenplay. One offline source test at 2761aac saved exact BLUE umbrella
text and confirmed that reader sees it, but ContextStackProjection contained only
an old synthetic RED recap and no screenplay receipts. Both the native context
projection and agent ReadContextStack used that incomplete projection. Preserved
reproduction source/log/hashes: `/workspace/scratch/manual-context-stack-audit/`.
No native UI or model was executed by this reproduction.

Both consumers now share `context_stack_projection.rs`. It reads canonical
hierarchy, latest recorded distilled context, exact main-document screenplay and
the relevant revision clock in one SQLite snapshot. It reuses the existing
target/intersecting plus two preceding/following segment selection, retaining
source/document/block/segment IDs, exact text and separate write revisions. Legacy
absent screenplay evidence remains unknown; a fresh empty read is explicit.
Script/context revisions advance the existing envelope clock, including text ABA.

The existing agent graph-context manifest exposes its existing `read_context_stack`
tool. The actual structured provider/parser/harness/read tool persists the evidence
it read through existing tool-result history. Its description warns that recorded
summaries may predate authored text; no semantic freshness or model interpretation
is inferred. The run-history DTO has one definition beside its existing store and
remains re-exported by the service with unchanged public API/wire shape.

Existing script, context-assignment, node and timeline events refresh only an
already requested context stack through the existing queue. Request/version guards
preserve post-save evidence against older responses and cleared/navigation
continuations. Graph layer detail shows exact saved text alongside a labelled
recorded summary, distinguishes empty/unavailable evidence and withholds another
target's screenplay. No extra durable memory/cache owner or model dependency exists.

This bridge reads evidence. It never rewrites notes, recaps, Bible facts, screenplay,
locks or drafts. Existing downstream Needs review, pending targeted preview and
explicit acceptance remain responsible for generated replacement. No automatic
world extraction, relation inference, embeddings or project-switch recovery is
included. The membership P2 value-completeness guard remains active in this tree.

## Executed gates and limits

| Gate | Evidence |
| --- | --- |
| Core | 119 passed, zero failed/ignored; legacy absence/known empty wire distinction |
| Actual production-module harness | 283 passed, zero failed/ignored; imports actual persistence, context, proposal, agent and parser modules by path, with no AppState/type/storage shim |
| Frontend | 411 passed in 72 files; exact SSR text/custody, target distinction, event refresh and late-read preservation |
| Strict all-target server Clippy | Passed **compile-only**, ORT_SKIP_DOWNLOAD=1 |
| Checks/build and normal commit hooks | Passed; Svelte 0 errors/warnings, lint/format/Rust format and successor traceability |

Five new context-owner tests cover exact manual values/revisions and bounded
neighbors without summary rewrites; context-evaluation and text ABA clock changes;
known empty versus missing target/legacy unknown; a pinned WAL reader retaining
the entire old text/version snapshot after a concurrent manual commit; and manual
A -> context read -> actual synthetic preview -> unchanged pending B -> explicit
targeted acceptance with A preserved and refreshed lineage.

Two new agent tests execute the actual structured tool loop and history owner with
a labelled synthetic provider. The next provider turn receives saved text and its
revision; later writes/ABA change new receipts while historical tool results remain
exact. Missing targets produce failed runs, not successful empty reads. These
prove transport/evidence behavior, not real-model reasoning or quality.

The native AppState manual workflow test now also calls the public context-stack
service before/after an actual manual command with a stale project mirror and
checks exact saved text, command-response revision and increased version. It
**compiles but has not executed** for this successor. Harness counts must not be
reported as full server/native execution.

Frozen logs and actual harness imports are in
`/workspace/scratch/screenplay-context-stack-4aae549/`. Qualification receipt
SHA256 `fd889ebd9de1a68c7cd33fa8d1ca5e6f2fd9da3eb922f8b6923d3b331679e715`.
The combined 283-test harness includes the new real >200-node membership repair
regressions; the earlier 245-test repair remains separately qualified.

Local GTK/GLib/native dependency execution remains unavailable; local ONNX403 was
not bypassed. Only the known full-workspace pre-push hook was excluded for branch
publication. There were no external reviewer requests, credential changes, paid
services, merges or deployments.

Prior qualification receipts, logs and canonical native images were rehashed
unchanged. Previous [Bible membership](bible-context-membership-qualification.md)
and [canonical native](canonical-scene-generation-qualification.md) evidence remains
specific to its frozen application source. No screenshot qualifies this successor.
The prior final banner/history gate remains blocked after two zero-step hosted
allocation failures in run 37365296113; no repeated rerun was requested. New hosted
AppState/native UI and real-model execution remain separate qualification gates.
