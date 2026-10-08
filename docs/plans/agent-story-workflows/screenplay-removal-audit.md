# Saved screenplay block removal: implementation admission

## Verified base and non-overlap

Product branch `feat/manual-screenplay-removal` starts at public PR25
`c5f358710d5a27f776c82a394b28d46e186f9c01`, tree
`a27a93791de9adc3882a2cd2da58af3621bc9732`, directly above PR24
`b17bfbc2506a67762e1cf0ae9dcfad81c933b049`. GitHub check runs for PR25
confirm Linux, Windows, frontend and traceability passed (run37764315165).
Linux/Windows ran571server +123core; frontend562tests. Its CodeRabbit
conversation reports a full clean review of its three changed files.

PR24 already fixes known-empty arc-description applicability, including name
changes while description is empty, a remaining empty tag, and deletion/absence
receipts. PR25 preserves the exact selected SQLite path. Neither repair is
repeated here. All previous49worktrees and original untracked evidence remain
preserved; this work uses a new isolated50th worktree.

## Concrete application gap

`ScriptBlockEditor.svelte` exposed Edit/Save and `ScriptBlockComposer.svelte`
appended user-authored blocks. `commandApi.ts`, native registrations and core
contracts had no manual block-removal command. Writers could remove prose only
by saving an empty block or replacing a whole segment through an agent patch.
Neither expresses deletion of one exact block.

The script-generation plan already calls for constrained block/span patches,
including deletion of unlocked blocks. Existing `script_segment_replace.rs`
soft-deletes omitted blocks and spans; `script_impact_projection.rs` understands
deleted consumed blocks and retains historical input excerpts. The feature
exposes these existing owners to explicit manual authoring. It keeps the timeline
clip and segment, other blocks, history, Bible facts and graph canon intact.

## Exact product write set reported before source edits

- Core: `contracts/script_document.rs`, `contracts/mod.rs`, `contracts/README.md`.
- Server: `lib.rs`, `command_service.rs`, new `script_block_remove.rs`,
  `script_block_remove_tests.rs`, `script_block_remove_impact_tests.rs`, `README.md`.
- Native: `commands/object_script_story.rs`, `lib.rs`, `commands/README.md`.
- UI library: `scriptTypes.ts`, `commandApi.ts`, `README.md`.
- UI stores: `scriptDocumentProjection.svelte.ts`, `scriptBlockEditSession.svelte.ts`,
  `README.md`.
- UI editor: `scriptBlockEditDraft.svelte.ts`, `ScriptBlockEditor.svelte`,
  `ScriptPanel.svelte`, new `ScriptBlockRemoval.svelte`,
  `scriptBlockRemoval.svelte.test.ts`, `README.md`.
- Plans/evidence: parent `plan.md`, this audit, and a new qualification report.

Native qualification belongs to a separate branch using the existing ten-file
manual-fact fixture boundary. No embedding/model dependency or credential change.

## Acceptance gates

Explicit confirmation captures exact saved block revision. Stale text/ABA and
locks refuse before a committed mutation; the writer recaptures rows and retained
membership. Removal is a UserEdit with a deleted-block history revision preserving
exact original text, deleted spans, and sparse segment-membership delta. Retry
uses the original command ID and replay precedes missing-block/version checks.

The existing per-block authoring session owns removal confirmation and any lost
acknowledgement receipt. An editing draft cannot enter removal. Unknown failure
keeps an immutable receipt and visible retry even if a ScriptChanged refresh has
already removed the block. Project-session retirement prevents retargeting.

Affected saved scene material remains unchanged until fresh targeted preview and
separate explicit acceptance. Old previews refuse after source removal. Fresh
current memory excludes removed prose while impact causes retain consumed text.
Unrelated material and timeline placement remain intact. Synthetic responses
qualify mechanics only, never real-model writing quality.
