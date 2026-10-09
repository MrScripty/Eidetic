# Consumed ancestor Notes review gap

Status: implemented, source-tested and independently accepted for the bounded synthetic native gate. Based on qualified Notes prompt
source `c5fad5cbca6f1d3ba42bcdb66c833ac1d513dbf4` and provenance head
`840f459cf34ea1856c5eaf127698663ff07d22ff`, on separate
`test/ancestor-notes-review-gap`. Baseline reproduction is preserved at
`c2659fa409c7043d9a831f0b1b03ed92d49b9e74`; QA-host changes remain separate.

## Evidence before implementation

`timeline-notes-review.md` explicitly leaves ancestor/sibling prose as a separate
follow-up. `build_generate_request` includes the actual ancestor chain, and
`prompt_format::build_chat_prompt` supplies each ancestor's Notes in CONTEXT
HIERARCHY. `GenerationInputs` and `GenerateScriptBlockCommand` carry the selected
target receipt, but no consumed ancestor Notes receipt. Existing
`timeline_notes_lineage::from_target`, `recorded` and `dependency` account for the
selected target only. The targeted proposal binding likewise carries one selected
Notes input. Ancestor prompt consumption therefore has no owned Notes dependency.

Two real server regressions use canonical SQLite, the existing context attachment
and generation writer, and public `set_timeline_node_notes`. The fixture selects
an actual Scene under an Act, saves exact ancestor Notes
`  Mara conceals the witness — 雨.\n\n  `, verifies that exact value appears in the
actual formatted prompt, and proves distinct unrelated Act Notes do not appear.
The saved screenplay output is explicitly synthetic. There is no model call or
claim about narrative quality.

- Before implementation, changing the consumed Act Notes to
  `  Mara reveals the witness — 雨.\n\n  ` failed precisely at
  **“Exact ancestor Notes supplied to generation changed without downstream scene review”**.
  Prompt consumption, fresh-before-edit review state and saved-material
  preservation assertions pass first. The test also requires the eventual review
  cause to identify the actual ancestor and original exact Notes.
- Changing the unrelated Act Notes passes and preserves the review projection.
- Both compare every saved segment, block, span, lock, placement and block revision;
  the public edits do not replace saved screenplay.

Local locked server execution compiled through the existing prepared dependency
setup. No ONNX download bypass, alternative library, new cache, embedding
dependency or credential/configuration change was introduced. Raw baseline logs
stay outside Git under `/workspace/scratch/notes-prompt-preview/`.

Current locked all-target suite: **123 core / 556 server pass**, with **zero ignored**
cases. Full UI **558 tests / 96 files** and production build pass. Strict
all-target/all-feature core/server
Clippy, Rust format, decision traceability and **5 ONNX policy tests** pass.

## Reproduce and maintain

The baseline commit marked the positive known-gap regression ignored in ordinary
CI and executed it explicitly to prove failure. The implementation removes that
annotation: the same public-command regression is now an ordinary passing test.

```sh
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_consumed_ancestor_notes_edit_marks_saved_scene_for_review \
  -- --exact --nocapture
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_unconsumed_act_notes_edit_preserves_saved_scene_review_state \
  -- --exact --nocapture
```

## Implemented bounded propagation

The generation read retains only nonempty ancestor Notes actually supplied to generation, with their
owned field revision captured before provider I/O. It reuses command/proposal JSON,
existing semantic dependencies, field-history validation and impact projection.
Original/current ancestor evidence is shown in the existing targeted review, preserving
all saved/manual text until explicit acceptance, with stale/ABA and draft guards.
Accepted proposals carry the newly consumed receipts through the
existing lineage mechanism. Legacy missing receipts remain unknown; never
backfill consumption from current prose.

Do not broaden this slice to sibling prose, names, beat types, new hierarchy or
membership, project switching, Pumas publication or model-quality evaluation.
Twelve lineage regressions plus the two runtime boundary cases exercise exact
identity, equal Notes on distinct ancestors, stale initial reads, duplicate/forged
evidence, delayed generation, acceptance lineage refresh, clearing, pending
storage/acceptance ABA refusal, unrelated and non-Notes changes, locked manual
text, legacy absence, deletion and provider failure. Existing read-model schema
and acceptance flow are retained; there is no automatic regeneration.

Actual native run37721620953 passes at frozen product10a2b43 / QAc09b992. Root independently reviewed the source, visually inspected all five captures and verified artifact/UI source hashes, with no blocking finding for this bounded synthetic gate. The unrelated Act control gains no new ancestor-Notes cause; pre-existing causes remain. See `ancestor-notes-qualification.md` for exact identities, tests, captures and preserved attempts. Real-model quality remains unqualified. Selected Notes prompt run37713651568 and its source-bound artifacts stay preserved. Publication is a source-only stacked draft; parent retains PR19-22/main merge decisions.
