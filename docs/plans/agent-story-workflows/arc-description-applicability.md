# Known-empty tagged arc description applicability

Separate `feat/arc-description-applicability`, based on accepted PR23 provenance
`e9abf3efae6f6dc9e9d4b610cb4e01f817d487f7`. PR19–23/main remain unmerged.
This is a reproduced boundary, not yet implemented or native-qualified.

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

The positive is temporarily ignored in ordinary baseline CI and executed
explicitly below; implementation must remove that annotation and make the same
case an ordinary passing test.

```sh
cargo test --locked -p eidetic-server \
  ai_generation_runtime::runtime_tests::public_known_empty_arc_description_entry_marks_saved_scene_for_review \
  -- --ignored --exact --nocapture
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
