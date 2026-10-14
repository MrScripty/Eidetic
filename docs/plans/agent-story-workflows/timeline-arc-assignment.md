# Manual timeline arc assignment

## Verified gap

The accepted base b8227132 preserves Notes, exact Bible text and missing screenplay draft custody. It descends from verified PR9 successor25b860d12fe37a3538a40616cc02ab4a81370863 (tree d22b036891aed7eb66fc94059aa1b5c1d9aab85a). Existing core tag/untag methods, SQLite node_arcs and rendered clip colors have no manual selected-clip assignment command or UI. Move/resize, placement, blade split and title/Notes already work. The roadmap explicitly leaves arc membership separate.

This operation assigns story arcs to a clip on its current story-level track. It does not change hierarchy or move the clip to another story level.

## Canonical operation and review

The selected editor shows an arc checklist with explicit Save and read/discard controls. Drafts are retained per clip through navigation/panel disposal in the same project session. Unavailable assigned IDs remain visible. Fresh saves respect clip locks; uncertain original saves can still replay after a later lock.

SetTimelineNodeArcsCommand validates the exact canonical assigned-ID set and its owned arc_ids field clock inside the existing history transaction. It validates live requested arcs, rejects duplicate/no-op/stale requests, writes existing node_arcs, and records a sparse TimelineNode arc_ids revision. Replays return the original command's owned receipt even after a newer assignment. There is no second membership database or canonical frontend state.

Generation captures the assigned-ID read in its existing target receipt, alongside canonical tagged arc fields. The existing graph receives a TimelineNode membership dependency. Known baseline/empty membership remains distinguishable from missing legacy receipts. Impact uses the actual recorded generation or accepted proposal receipt, so first assignment, last removal and edit/restore ABA are reviewable. Unrelated clips and presentation changes do not invalidate this membership read.

Targeted proposal bindings retain original/current membership, original/current arc fields and exact screenplay input revisions. Prompt and UI show original/current names/IDs, including current '(no arcs)'. Historical removed arc names are labelled historical evidence; removed arc descriptions are excluded from current guidance. Withdrawal of live consumed arc context is allowed only when known consumed membership and a validated current owned membership transition prove the removal. Missing legacy consumption and corrupted unlogged tag changes stay refused.

Saving and previewing preserve saved screenplay and manual drafts. Existing explicit Accept replaces only the selected generated block, validates current binding under the writer and retains membership custody for the accepted successor. Stale preview/acceptance and locks preserve canonical material.

## Verification and limits

Nine focused backend regressions cover first/last membership, accepted successor, exact original replay, atomic duplicate/missing/no-op refusals, stale save, pending generation/preview/accept ABA, legacy unknown, unrelated clips, lock-after-lost-ack recovery, forged ownership, corrupted tag fixture and historical field ordering. The provider callback test checks last-removal prompt evidence. Thirteen UI tests cover explicit intent, arrays/receipt custody, session ownership, conflicts and rendered evidence.

The local rendered run uses actual AppShell, Bible inspectors, selected clip editor, screenplay panels and timeline, with normal public SQLite services through a test IPC adapter. Seven screenshots and raw command/provider transcripts cover unsaved checkbox/navigation custody, uncertain save after lock, conflict retaining intent, pending targeted review, stale acceptance, explicit target-only acceptance and last removal from the accepted successor. Manual saved blocks remain byte-for-byte unchanged; an exact Unicode/whitespace manual screenplay draft survives the entire flow.

All synthetic responses are labelled. This is instrumented Chromium/public-service qualification, not native runtime or real-model quality qualification. A compiled Linux service adapter and stable post-run source/Vite-module byte pairs are retained; module receipts are not capture-time attestation. No reopen after acceptance was exercised. Hosted native qualification remains for the parent workflow; no paid CI, main merge, external review, credential change or ONNX403 bypass was performed.

Evidence lives in /workspace/scratch/timeline-arc-assignment-evidence. Parent owns PR/review/merge and Library delivery. Frozen PR27/28 and previous accepted checkpoints remain unchanged.
