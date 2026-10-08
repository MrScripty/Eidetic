# Consumed ancestor Notes review gap

Status: reproduced; bounded test milestone only. Based on qualified Notes prompt
source `c5fad5cbca6f1d3ba42bcdb66c833ac1d513dbf4` and provenance head
`840f459cf34ea1856c5eaf127698663ff07d22ff`, on separate
`test/ancestor-notes-review-gap`. No production or QA-host change.

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

- Changing the consumed Act Notes to
  `  Mara reveals the witness — 雨.\n\n  ` fails precisely at
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

Maintained locked all-target suite: **123 core / 543 server pass**, with the one
known positive reproduction ignored. Strict all-target/all-feature core/server
Clippy, Rust format, decision traceability and **5 ONNX policy tests** pass.

## Reproduce and maintain

The positive regression has an explicit `#[ignore]` naming this known gap, so this
test-only milestone does not turn ordinary CI red. It was executed explicitly and
failed as described; ignoring it is not implementation or qualification. Remove
that annotation when the feature passes.

```sh
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_consumed_ancestor_notes_edit_marks_saved_scene_for_review \
  -- --exact --ignored --nocapture
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_unconsumed_act_notes_edit_preserves_saved_scene_review_state \
  -- --exact --nocapture
```

## Bounded implementation contract

Retain only nonempty ancestor Notes actually supplied to generation, with their
owned field revision captured before provider I/O. Reuse command/proposal JSON,
existing semantic dependencies, field-history validation and impact projection.
Use original/current ancestor evidence in the existing targeted review, preserve
all saved/manual text until explicit acceptance, and keep stale/ABA and draft
guards. Accepted proposals must carry the newly consumed receipts through the
existing lineage mechanism. Legacy missing receipts remain unknown; never
backfill consumption from current prose.

Do not broaden this slice to sibling prose, names, beat types, new hierarchy or
membership, project switching, Pumas publication or model-quality evaluation.
Before calling it complete, add receipt ownership/forgery, ancestor deletion,
original-versus-current and delayed generation/preview/acceptance tests, then
qualify the real application UI using the established separate hosted native QA
route. The already passing selected Notes prompt run37713651568 remains closed.
