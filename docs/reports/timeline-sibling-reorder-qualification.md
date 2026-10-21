# Atomic timeline sibling reorder

## Gap and scope

Based on accepted arc-assignment source `1f45e152fe68b9dd34d8058a8c1064ba6de3437f` (tree `ac1cb02fad05d636d8b4dd4c99a89b8d0cb176a6`).

The M4 authoring plan calls for shared validated manual/agent timeline mutations. Existing drag/exact placement moves one clip and already supports consumed scene-order memory. There was no atomic adjacent-sibling reorder command/editor. `Timeline::children_of` orders by `(sort_order, start_ms)` while screenplay context follows chronological segment placement. Moving one clip alone does not author both placements or their hierarchy order. This milestone adds that concrete operation; it does not claim scene-order lineage was previously absent or complete the deferred split-containment work.

## Authored change and review

The selected-clip editor provides **Move earlier/later**, a named pair preview and **Apply reorder**. Choosing a direction does not write. For the rendered fixture, B: Setup initially occupies 150000–300000 ms and A: Complication 300000–480000 ms. Moving A earlier produces A 150000–330000 and B 330000–480000. Each duration and the original gap/outer span are preserved.

The core plans both subtrees before mutation, translates descendants by checked exact integer offsets, swaps root `sort_order`, and rejects different parents/levels, nonadjacent clips, overlap, inconsistent hierarchy/chronological adjacency and invalid child containment. Equal order values use the existing chronological tie-break. Content locks retain their AI-regeneration meaning: manual structural reorders preserve locks and remain allowed.

The server uses existing command/object-revision history, canonical nodes and `timeline_script_placement`. Both roots and all changed descendants receive sparse placement/order revisions; source-bound screenplay segment geometry updates in the same SQLite transaction. Block text, spans, locks, block revisions and unrelated material remain untouched. Existing continuity dependencies identify affected generated scene material and supply the normal targeted preview/explicit acceptance workflow. No parallel reorder state, embedding dependency or automatic screenplay replacement was added.

The backend-owned sibling read includes all sibling identities, names, placement/order and owned node clocks, plus the parent's structural membership clock. The writer checks that receipt and the full canonical timeline inside its transaction. Insert/delete ABA invalidates the old read. Recorded-command replay precedes current existence/lock checks, and the returned pair receipt is validated against the original stored command and sparse fields rather than reconstructed from current placement. Pending generation, preview and acceptance retain their existing stale/ABA guards.

The UI keeps per-clip intent in the existing project session, with a privately copied immutable original pair/command for uncertain retry. Changed saved siblings retain draft intent and block stale fresh applies. Successful explicit reads discard the draft; they cannot discard an uncertain command. A confirmed acknowledgement survives asynchronous projection refresh or a later lock clock. Saved screenplay and an exact whitespace/Unicode manual draft survive navigation and downstream review.

## Validation and qualification boundary

Final maintained checks pass **127 core / 613 server** tests (zero ignored), **634 UI tests / 110 files**, strict core/server all-target Clippy, Rust/Tauri formatting, ESLint, Svelte diagnostics (0 errors / 0 warnings) and the static production build. New focused cases comprise 4 core, 7 server and 13 UI tests.

Meaningful unit cases cover unequal durations/gap, nested translation, malformed/overlapping clips, equal hierarchy order, near-maximum safe UI timestamps, content locks, whole-sibling clocks, membership ABA, original retry after later reorder/deletion, immutable transport copies, project-session ownership, pending generation refusal and pending acceptance ABA. The rendered qualification runs actual AppShell/Bible/timeline/screenplay components through a compiled local adapter to public command/projection services and real SQLite. Its six final captures show unwritten pair intent, uncertain-but-committed state, retained conflicting intent, pending targeted review, stale acceptance, and explicit target-only acceptance.

All provider responses are labelled synthetic fixtures. This is local Chromium/public-service qualification, not native Tauri/Bevy qualification or real-model execution/quality. The test adapter manually refreshes normal projections because it has no native event transport. Raw local source/current Vite module bytes were measured after the final run; they are not capture-time source attestation. Earlier local probe transcripts/captures are retained separately, including the corrected acknowledgement display and harness readiness/projection-refresh issues. No ONNX403 bypass, paid CI or Pumas identity-policy change occurred.

Exact final source/tree, test totals, independent review, screenshot hashes and supporting evidence are bound in the external handoff/manifest under `/workspace/scratch/timeline-sibling-reorder-evidence`. Parent owns native qualification, PR/review/merge and Library delivery. Deferred split containment, project-switch recovery and optional embeddings remain outside this operation.
