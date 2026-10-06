# Pending timeline plan recovery

The editor can reopen a stored pending timeline proposal after Close or selection
away and back within the current project. Review saved timeline plans reads the
existing `projection_child_plans` endpoint; it lists every Pending record for the
selected non-leaf clip. Review plan restores only the proposal the writer chooses,
including its original canonical children and exact saved-screenplay evidence.
Accept timeline plan remains a separate guarded timeline command. Reads never
generate or apply material, replace saved screenplay, or infer newest from zero
creation timestamps. No new schema, proposal store or model dependency.

## Source

- Base: merged PR11 `325f505481369e91f6cfdc125a27b2bf3b2a4384`, tree
  `45546e91b6e4c5d64d301a733c473945f1b911a9`.
- Implementation: `511fd8928888844b4f094513b1449b866ecd2f37`, tree
  `7a7dabd4bf724ad602b9830a6786e25ba3fc109e`.
- Frozen source: `7f0b4e502bb668e8ef4a8ab29487bdf1b8cdeeae`, tree
  `c98794dc8aa3f1b5a93d6992234068150e5d5588`, branch
  `feat/recover-pending-timeline-plans`. The follow-up changes two directory
  READMEs only; runtime and tests match the implementation checkpoint.
- Qualification: `6c6439586ccb62d8a26831c8d425c55d52930cc0`, tree
  `f4049a17b1a78c276c1991068faf59c3d0dbb4c8`, branch
  `test/child-plan-recovery-native`. All 574 application/dependency blobs match
  frozen source. Only four qualification/workflow files differ.

The documented gap was explicit in agent-story-workflows/plan.md:958. The existing
review controller cleared its preview on owner changes and Close. The backend
already reconstructed canonical proposals, status and original inputs from the
existing creation command; the frontend had no reader. Existing authoring,
Bible propagation, stale-placement and relationship repairs were retained.

## Local checks

- Core119, server444, frontend445 pass. Two new actual synthetic HTTP/service
  tests recover multiple stored proposals through the public projection in a
  fresh AppState. Exact normalized material and original receipts survive;
  only explicit acceptance changes the chosen status. Replay preserves later
  manual text/history. A recovered stale receipt refuses without timeline,
  screenplay, Pending evidence or four history-table count changes.
- Frontend controller, projection transport and SSR checks cover read retry,
  empty/multiple proposals, Pending/parent filtering, selection ABA, session and
  unmount retirement, generation/read exclusion, original evidence, explicit
  choice and uncertain acceptance retaining its exact command/payload.
- An actual client-compiled Svelte controller test exercises reactive proposals.
  Removing only `$state.snapshot` reproduced DataCloneError; the fixed controller
  reopens exact original material and explicitly submits canonical acceptance.
  This test does not claim DOM or native UI qualification.
- Strict server all-target Clippy, frontend typecheck (zero errors/warnings),
  lint, formatting, production build, whitespace and complete branch decision
  traceability pass. Qualification-driver26 and no-download-gate5 tests pass.
- Resolved locked all-features Cargo metadata passes the maintained no-download
  gate. The Pumas pin and dependency graph are unchanged. No local ONNX download
  bypass. The unavailable local full GTK/WebKit pre-push launcher was excluded
  via `LEFTHOOK_EXCLUDE=test`; hook configuration is unchanged. Hosted native
  qualification uses standard GTK/WebKit, display dependencies and normal builds.

## Native qualification

Passed: corrected source-bound run
[37424649728](https://github.com/MrScripty/Eidetic/actions/runs/37424649728).
Job `112141445299` passes standard native builds, no-download gate5,
qualification-driver26, core119/server444/frontend445 and frontend strict checks.
The real Tauri application runs through the maintained launcher and its normal
Vite development UI, using supported AT-SPI geometry/scroll and X11 input. The
production frontend build also passes. No injected JavaScript or IPC and no
edited pixels.

The native writer manually saves exactly `Mara's umbrella is blue.` over
`Mara's umbrella is red.`; saved screenplay and the exact 41-character manual
draft survive. Downstream Needs review, targeted preview and explicit screenplay
acceptance remain qualified. The writer then saves the exact 80-character
midnight source, generates a Pending child plan, closes it, selects another
scene and returns. The saved list and explicit Review plan restore its original
proposal and both exact recorded screenplay inputs. Every logical SQLite row,
including command/history/receipt state, hashes identically before/after:
`00fbea394c2f7aab740b6a7d90ef4c332852efc64edd798b03ce7e140fe404fd`.
Provider requests remain four; recovery does not regenerate.

After an exact 83-character manual morning Save, acceptance refuses the recovered midnight plan.
The native refusal is visible, the proposal stays Pending, and canonical children
and screenplay remain unchanged. Fresh generation consumes current morning text;
explicit acceptance creates Morning departure and preserves saved screenplay
bytes/revisions and the Bible value. The final frame uses the existing panel resize and native
scroll controls to show all three saved morning lines fully inside the writing
area, the blue Bible fact and accepted Beat together. B's existing Needs review
remains truthful; accepting timeline children does not accept a screenplay update.

Artifact `11394742805` (922158 bytes), ZIP SHA-256
`716e97337791c6170507e8f7e7c831f62fa062fcadfe0024338dafcaf4fe7247`,
is downloaded and safely extracted at
`/workspace/scratch/child-plan-recovery/native-37424649728/`.
Capture receipt SHA-256:
`062321bb40e53d8c2fcc390647fe26423bcb50405438bbbf2237cff9c0879321`.
Actual native binary SHA-256:
`71d711be42f3351e7fcd2a6628b1ed2fe4bb62f2fea3a2cf6295a8b091eaccde`.
Rust production code is unchanged by this frontend feature; the normal launcher
loads the new UI source. Source guard, fresh frontend gates and actual new controls
qualify that UI rather than relying on executable hash alone. All nine untouched
1440x960 PNGs match the receipt hashes and have been viewed. Source/trees,
artifact/receipt/log hashes and individual captures are in the adjacent manifest.

| Capture | Evidence |
| --- | --- |
| `eidetic-bible-fact-review.png` | Blue fact saved, affected screenplay Needs review. |
| `eidetic-bible-fact-preview.png` | Pending targeted screenplay proposal. |
| `eidetic-bible-fact-accepted.png` | Explicit targeted screenplay acceptance. |
| `eidetic-child-plan-pending.png` | Pending midnight proposal before recovery. |
| `eidetic-child-plan-recovery-list.png` | Stored proposal listed after Close and navigation. |
| `eidetic-child-plan-recovered.png` | Original midnight/B screenplay evidence restored. |
| `eidetic-child-plan-refused.png` | Exact native stale refusal, Pending proposal retained. |
| `eidetic-child-plan-accepted.png` | Explicit fresh acceptance, Morning Beat visible. |
| `eidetic-story-memory-readable.png` | Fully readable manual source, Bible and accepted timeline. |

The original run
[37423029259](https://github.com/MrScripty/Eidetic/actions/runs/37423029259),
job `112136412620`, passed all builds/service/frontend gates and the recovery
checks: original evidence, unchanged complete logical SQLite hash and four
provider requests before/after. It then timed out locating a visible stale-refusal
message after the expanded disclosure was scrolled. The failure frame and receipt
are preserved in artifact `11394610357`, ZIP SHA-256
`f9f0ddcc23beaf9ec09721a2c813baebc6e722fbe9a81a4f19a4beaa9c96f9bf`.
The successor changes only `ui.find` to supported AT-SPI `ui.reveal` for that exact
native refusal, and adds a hidden-error scroll regression. Error equality,
SQLite preservation checks and application admission remain unchanged. The
original failure is not converted to a passing product-qualification claim.

## Limits and ownership

Provider responses are labelled synthetic HTTP/SSE fixtures. No real-model
quality, SDK inference, inspector GUI, relationship-drawing or GUI restart claim.
Fresh AppState tests demonstrate SQLite/service durability; project-switch
recovery remains deferred. Full child-plan Bible/affect/arc-description source
binding remains a separate documented criterion. Existing screenplay/subtree/
relationship receipt admission and exact uncertain retries remain authoritative.

Parent owns PRs, reviews, merge and Library delivery. Original checkouts and
historical source/evidence branches are preserved. No external review requests,
credential changes or paid services.
