# Known-empty tagged arc description applicability

Separate `feat/arc-description-applicability`, based on accepted PR23 provenance
`e9abf3efae6f6dc9e9d4b610cb4e01f817d487f7`. PR19–23/main remain unmerged.
Baseline `1feeee8cbd79fce71b90aed78f5d95caeb504d44` reproduces the boundary.
Implementation now retains applicability through existing generation/proposal
history and graph dependencies. Native functional qualification passes run37726307798 against product cac013f /
QA1989909. Independent source/visual review remains for the parent; real-model
quality is unqualified. See `arc-description-qualification.md`.

The consumed-arc roadmap explicitly leaves previously unconsumed empty
descriptions separate. ArcDetail already commits exact typed descriptions through
the public metadata writer. Generation omits an empty Description from supplied
fields, so filling it creates no old description dependency. The legacy
`unconsumed_empty_description_is_not_backfilled_after_an_edit` test protects that
honest unknown behavior and must remain.

Two new real runtime tests select a template-tagged Scene, publicly commit an
owned empty Description, attach canonical generation context, prove the actual
prompt omits that description, and persist explicitly synthetic saved output.
The public writer then fills exact `  Mara chooses exile — 雨.\n\n  `.
Fresh canonical prompt inclusion and saved-material preservation pass, but the
positive fails at **“Known-empty tagged arc description became supplied without
precise downstream Scene review”**. A control publicly repeats empty clearing and
fills an untagged arc; it passes and preserves saved material/review state.
Raw logs remain outside source Git. No model call or local ONNX403 bypass.

The positive was temporarily ignored only in baseline CI and executed explicitly
with `--ignored --exact`; implementation removes that annotation. The same case
now passes as an ordinary test. The legacy no-backfill test remains unchanged.

```sh
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_known_empty_arc_description_entry_marks_saved_scene_for_review \
  -- --exact --nocapture
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_known_empty_arc_clear_and_untagged_description_preserve_saved_scene \
  -- --exact --nocapture
```

## Bounded decision

Capture applicability evidence for known-empty Description fields on actually
tagged arcs in the same generation snapshot. Retain exact arc/field identity,
empty value and owned field clock through existing command/proposal JSON and
semantic dependencies. Omitted prose is not consumed prose; missing legacy
receipts and unowned history remain unknown. Do not backfill from current tags.

Filling an applicable known-empty description should identify affected Scene
material, show original omission/current exact text in existing targeted review,
and supply the current direction to its preview. Saving and previewing preserve
manual text, spans, locks, placement and drafts. Only explicit acceptance updates
the chosen block and refreshes its actual consumption. Keep writer recapture,
stale/ABA refusal and negative clear/unrelated controls. Qualify actual ArcDetail
typing with Bible, timeline and screenplay visible, then obtain independent
acceptance before publishing a source-only stacked draft. Synthetic responses
must remain labelled; real-model quality is unqualified.

No new canonical store, schema, provider or embedding dependency. Arc assignment
editing, broader tag membership, hierarchy inheritance, fictional-time inference,
automatic Bible extraction and project switching stay outside this slice.

## Implemented source contract

`arc_description_applicability` uses optional `StoryArcFieldInput` collections,
empty Description values and owned history. Missing receipts and unowned reads
remain unknown. Generation admission validates the selected tag identities and
omitted fields under the writer before saving. Each owned applicability receipt
creates an existing revision-bound ScriptSegment-to-StoryArcField dependency,
with an explicit omitted-prose rationale. Impact checks that original applicability,
current assignment and newly available nonempty prose; no broad current-state
backfill is performed.

Targeted review captures original omission and fresh current fields. The provider
receives exact new description and manual target text. Proposal storage and
acceptance recapture the same receipt; clear/restore ABA refuses old work without
writes. Clearing withdraws the entry cause. Explicit acceptance refreshes the
chosen block's ordinary consumed Description lineage; another consumer keeps its
own precise cause and exact text.

Maintained regression coverage includes real graph-row assertions, public
Arc metadata entry/clear/untagged controls, forged/duplicate/missing receipts,
legacy unknown history, unrelated metadata, multiple consumers, locked manual
spans and partial provider failure. UI evidence distinguishes omitted prose from
consumed fields and renders exact Unicode/whitespace and revisions. Synthetic
test outputs are labelled. Native qualification must exercise ordinary ArcDetail
typing with Bible, timeline and screenplay visible before publication.
