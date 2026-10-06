# Exact timeline placement: inspected feature choice

This independent branch starts provisionally from PR15 `ab25029e58cf21dab662b180629caf41795d5fdc`
(tree `30a7b0f746ca132bc70bfe601faef6df1f9e1749`). PR15 remains untouched. The parent then merged it as verified main
`74fd438f3ab4cf40734e094b4b2a62fc67d58b07` with the identical tree. This feature
was fast-forwarded to that main ancestor without discarding in-progress work.

The existing manual screenplay path already saves exact text and revisions,
feeds canonical generation/agent/child-plan context, derives downstream review,
and requires preview and explicit acceptance for targeted updates. Existing range
commands atomically update source-bound segment placement without changing block
text, and publish TimelineChanged plus ScriptChanged. Placement and continuity
receipts already identify affected generated material. Relevant service evidence:
`native_manual_creation_edit_save_reopen_and_retime_reach_exact_prompt_memory`,
`native_range_edit_publishes_script_change_only_after_atomic_success_and_not_replay`,
and the canonical generation, child-memory and impact-review suites.

The bounded user-visible gap is precise placement authoring. BeatEditor and its
header have no start/end form. StoryNodeClip clamps drag and resize against the
adjacentTimelineRenderClipBounds supplied by LevelTrack, so a writer cannot move
a clip past an adjacent clip through that gesture. The existing public range
command supports that edit, including the resulting screenplay continuity review.
This closes part of documented M4 manual move/resize parity; it does not redesign
track hierarchy, infer fictional time or add semantic extraction.

Expose start/end screen times and an explicit Apply placement action in the
selected editor. Capture the exact canonical range and existing node history
revision in the same read snapshot. Recheck that receipt under the existing
history writer before changing ranges, including ABA and rollback. Old callers
and recorded command signatures remain compatible. Errors retain entered times;
an uncertain acknowledgement retries its immutable command. Existing screenplay
draft owners and explicit targeted review/acceptance remain independent.

Qualification must show exact manual text, exact range authoring across a neighbor,
unchanged saved text/draft, downstream Needs review, preserved preview and explicit
acceptance. Synthetic production-client HTTP responses must be labelled. Standard
hosted native compilation must run without an ONNX download bypass.

## Local implementation gates

Baseline server485 passed. The implementation has server486/core120/UI467 passing, including the actual read/ABA/refusal/fresh-move/replay service regression, historical wire signature compatibility, exact input and immutable retry, selection/session retention and a compiled browser-rune proxy check. Strict server all-target Clippy and frontend typecheck0/0 pass. Native UI execution is now qualified separately at source8c1956f/qualifier443ddc2 in run37538195351. See exact-timeline-placement-qualification.md and its manifest; this report makes no model-quality claim.
