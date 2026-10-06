# Consumed Bible names and targeted screenplay review

Manual screenplay revision propagation is already implemented on merged main
`7a8172df78772f8f2b23bb8fe9a913aeb3b0bfd5`, tree
`14ee1b448bd39aff1c9774fd6cdd7a38443b5d0e`. The fresh audit ran 23 matching
manual-memory tests, including exact saved text in canonical generation and
structured agent context, affected-scene review, child planning and stale ABA.
The bounded missing feature is name provenance, separately documented in the
agent-story-workflows plan. The public-command reproduction supplied the prompt
header `- Mara [character] (Mara)`, persisted labelled synthetic screenplay,
renamed the Bible node to `Marisol`, and failed because review was absent.

Implementation `f933e5596b666d61d0eb5575e218daa364f96c5a`, tree
`3e701d4c48bb79ab85984c0e680841e99816e5fc`, now captures exact name inputs and
owned name revisions in the Bible snapshot. Existing `BibleNode` / `UsesFact`
dependencies, generation JSON, proposal JSON and writer transactions remain the
authorities. Parent/order metadata does not advance the name clock. Historical
reads remain bound after late generation; rename/restore and sparse-history
ownership drift cannot silently rebind. Changed/deleted names yield historical
review explanations. Preview retains all live consumed names, and explicit
acceptance rechecks custody before replacing only the targeted saved block.
Manual text/drafts and legacy unknown reads retain existing behavior.

Boundary documentation `ca69c3795bb47264bf53474b20062779144d43c0`, tree
`5ef3403077979eec73bdd137a851040e27494b9f`, adds three boundary READMEs.
Final maintained source `d59d503ccfb41871fcbbd9689932a84fe35c605f`, tree
`b4372541359e53ace1d0273fb8a9f9761b804630`, additionally refuses sparse deletion
history that disagrees with a live graph row, with its regression and boundary
documentation. Branch: `feat/bible-node-name-screenplay-memory`.
Qualification `4f2a75f671ec6dd1b9a27fd34ae87be8046233a0`, tree
`851da59b69b06454f6cce4fc5b4f6e0a1e4242fe`, branch
`test/screenplay-bible-node-name-native`, changes only the dedicated workflow,
name driver and driver tests. All application/dependency blobs match the source.

Local validation: core 119, server 482, UI 459, native qualification-driver tests
45 and no-download tests 5 passed. UI typecheck (zero errors/warnings), lint,
format, strict server all-target Clippy, rustfmt, diff and decision-traceability
checks passed. Resolved locked
all-features Cargo metadata passed the ONNX no-download feature-union check.
The public name-edit reproduction now passes. The workspace-wide desktop hook
fails at unavailable `glib-2.0.pc`; hosted standard native qualification supplies
GTK/WebKit normally, without ORT/ONNX bypass or credential changes.

## Hosted native result and screenshots

[Run37523579021](https://github.com/MrScripty/Eidetic/actions/runs/37523579021),
job112474823310 and [artifact11441696863](https://github.com/MrScripty/Eidetic/actions/runs/37523579021/artifacts/11441696863)
passed normal locked desktop/fixture builds and core119/server482/UI459/driver45/
no-download5. Actual native input authored the exact Mara-to-Marisol name change.
Saved text/placement and the exact manual draft survived. Preview retained the
name7286498b revision without changing canon. Name restore/re-edit produced
revision1af27813; accepting the old preview refused with every logical table
snapshot unchanged. The obsolete preview was explicitly rejected. Fresh native
review and explicit acceptance replaced only generated B, preserved manual A,
cleared review and refreshed name consumption to1af27813. Accepted block revision:
`4bf93e1d-0fb8-4581-8177-362e968d3d0e`.

Eight untouched 1440x960 captures have verified SHA256, window2097155 and PID22279.
All image bytes were viewed; name-review is byte-identical to the individually
viewed failed-run capture. Primary new captures are `eidetic-name-preview.png`,
`eidetic-name-stale.png` and `eidetic-name-accepted.png`, with Bible, timeline and
screenplay visible together. The accepted capture shows both exact saved scene
lines including Marisol. The fresh-preview capture retains older proposal history;
its fresh name receipt is established by durable binding and native acceptance,
not by claiming that the visible old receipt changed. Full hashes, IDs, paths,
receipt revisions and limits are in `bible-node-name-manifest.json`.

Original download: `/workspace/attachments/0266db36-edda-48c4-8ac9-40181d32f143/bible-node-name-success-native.zip`.
Extracted originals: `/workspace/scratch/manual-story-memory-audit/native-success-37523579021/`.
ZIP SHA256: `9cffc843e9eade7ccc369f409aeb19ca586490eaac56ced8f842eed5f01eca8b`.
Artifact retention ends 2026-10-09; parent owns Library delivery. No duplicate
upload or external reviewer request was made.

## Preserved failure and limits

Previous run37521680769/job112468964558 passed normal native builds and all
backend/UI gates, then failed at an AT-SPI retired None child after the exact
name edit. Artifact11440958869, original qualification4f797c3 and its failure
capture remain preserved. Qualification-only repair4f2a75f skips exact retired
None slots while retaining the existing 3000-node bound and unrelated errors,
with three regressions. The final run recorded17 retired-slot observations.
Superseded qualification6960fac0/run37521277899 remains preserved and is not
final-source evidence.

All five production-client HTTP responses (generation, recap, field preview and
two name previews) are explicitly synthetic. This proves context custody and UI
behavior, not real-model quality. Timed names, new-name membership inference,
automatic Bible extraction, embeddings, native multi-user races and project-switch
recovery remain outside scope. No unresolved feature or qualification blocker.
Parent handles PR/review/merge and Library delivery.
