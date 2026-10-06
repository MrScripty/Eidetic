# Exact timeline placement and affected screenplay review

The missing user-visible slice was precise clip placement. The baseline already
propagated manual screenplay edits into generation, structured agent reads,
child planning and explicit targeted review. Range edits already updated canonical
screenplay placement and derived continuity impact. BeatEditor lacked start/end
inputs, while timeline gestures could not move past adjacent clips. The inspected
choice is recorded in `exact-timeline-placement-audit.md`; baseline server485 passed.

## Source and ownership

Application `8c1956f7afa18a2a3ea3a1732fb8eca58a1129b3`, tree
`764ae1c5b2be223d75c6d3ee726fb365085bd93b`, branch `feat/timeline-story-memory`,
descends directly from verified merged main
`74fd438f3ab4cf40734e094b4b2a62fc67d58b07`, tree
`30a7b0f746ca132bc70bfe601faef6df1f9e1749`. PR15 and its preserved predecessors
were not edited or merged by this feature task. Author/committer: MrScripty.

The selected editor now exposes exact screen-time seconds and **Apply placement**.
Its existing selected-node read captures the canonical range and owned node event
in one SQLite snapshot. The existing range writer rechecks that receipt under
write ownership, excluding only its own in-flight event. Changed/ABA input refuses
with rollback; replay precedes validation and preserves later edits. Known
history-free nodes and absent fallback evidence stay distinct. Legacy range
callers omit the optional receipt and retain their exact recorded JSON signature.

Transient per-node placement drafts retain entered times through panel/selection
changes. Definite native pre-commit refusals retain editable input. Unknown
acknowledgements retain immutable payload/ID for exact retry. Explicit discard/read
adopts current placement; a failed read cannot adopt an unread base. Existing
screenplay draft owners, placement synchronization, memory dependencies, pending
proposals and explicit screenplay acceptance remain the authorities. There is no
new schema, alternate canon, model dependency or automatic screenplay replacement.

## Executed gates

Local and exact-source hosted **core120/server486/UI467** pass. The actual public
read/ABA/refusal/fresh-move/replay regression preserves manual text and derives
downstream review. Eight new frontend checks cover exact milliseconds, invalid
input, definite refusal, immutable retries, failed reads, session navigation,
rendered controls and browser rune proxies. Legacy wire round-trip/signature
compatibility passes. Strict server all-target Clippy (`-D warnings`), frontend
typecheck0/0/lint/format/build, rustfmt, diff and traceability pass. No-download5
and locked all-features feature-union checks pass; no ORT bypass or new model.

Local public-service qualification setup also executed: original A,F,B,C,D,E and
synthetic generated B were intact, leaving the reorder for GUI input. Final
qualification-driver43 passes. Local desktop/workspace checks lack glib-2.0.pc;
the source push excluded only that unavailable pre-push hook. Ordinary hosted
GTK/WebKit desktop and fixture builds below passed without download overrides.

## Real native result

Qualification `443ddc2635a24405679e0e74e130f84d24e09270`, tree
`5335f1c1351e24952fc363dd0aa2705f19aeef64`, branch
`test/timeline-placement-native`, changes only the dedicated workflow, public
setup fixture, native driver and driver tests. A diff gate verifies the frozen
application source. [Run37538195351](https://github.com/MrScripty/Eidetic/actions/runs/37538195351),
job112524966283 and [artifact11447317950](https://github.com/MrScripty/Eidetic/actions/runs/37538195351/artifacts/11447317950)
passed native builds and all repeated gates.

Actual native Save authored `Manual B: retain this station beat.\n\n` at revision
`9d4a89a4-39fb-450e-802d-c47c638cd63b`. A separate exact draft
`Retained manual B placement draft.\n\n` remained open. Native selection of E,
typing start180/end210 and Apply placement moved E from540000–570000 to
180000–210000ms, changing A,F,B,C,D,E to A,F,E,B,C,D. Command
`5cfdf512-7e0f-48f1-b979-83c2bdc119f2` retained the original range and owned event
`e4e2ec6c-d434-40a7-9d90-9ed3592c20ba`. Saved block IDs/text/write events and the
manual draft stayed intact; only intended placement changed.

The writer explicitly cancelled that healthy draft, leaving saved B unchanged.
Native review displayed **Entered: SCENE E. Left: SCENE A.** The production HTTP
preview consumed F,E,C,D and exact saved manual B, excluding displaced A. Proposal
`script.review.8ef0c699-59dd-49cf-8863-a785358ea953` stayed pending without changing
canon. Only native **Accept update** replaced B, at revision
`f52f45e1-47c6-4934-951e-3330a7f29c70`; other authored blocks and intended E
placement remained unchanged. Needs review cleared and the accepted canonical
paragraph was visibly revealed. All three provider responses are labelled
synthetic HTTP/SSE; this proves context custody and application behavior, not
real-model quality.

## Untouched screenshots and limits

Four original 1440x960 captures were hash verified and viewed without editing.
Window2097155/PID22244 and binary SHA256
`4a51fac11fee64f7d1b9e926a11145afe6cb5391b66387af2f31b34012553d80`
bind them to the owned native application.

- `eidetic-timeline-placement-authored.png`: exact 180/210 inputs, selected E,
  retained manual draft and timeline.
- `eidetic-timeline-placement-review.png`: Bible, timeline, precise placement
  and the visible entering/displaced-scene review cause.
- `eidetic-timeline-placement-preview.png`: unchanged review view during pending
  preview. The proposed paragraph is outside this image's constrained Script
  pane; pending state and input custody are established by the native receipt.
- `eidetic-timeline-placement-accepted.png`: Bible/timeline/placement with visible
  accepted proposal and matching canonical synthetic paragraph.

ZIP SHA256:
`79bed196d95d74fdc226c1f2bedd7c6b40a049d69e9c068fb30ec47a708173a0`.
Originals: `/workspace/scratch/timeline-placement-native-success-37538195351/`.
Full IDs, hashes, local log receipts and limits are in the companion manifest.
Artifact retention ends2026-10-09; parent owns Library delivery.

Initial qualifier945c4e3/run37537216165 was superseded and cancelled before GUI.
Reading the shared block observation exposed a qualification-only mistake: its
tuples contain placement as well as text. Correction443ddc2 requires exactly the
intended range delta while preserving block identity/text/revision/membership,
with a regression and native command-receipt check. Application source is unchanged.
No failing or cancelled run is presented as qualified GUI evidence.

Placement ABA refusal is proved by the actual public-service regression, not a
claimed native GUI ABA sequence. Hierarchy/reparenting, fictional-time inference,
semantic extraction, embeddings, real models and project-switch recovery remain
outside this slice. No PR, external review, merge or Library action was performed.
No unresolved feature or functional native qualification blocker remains.
