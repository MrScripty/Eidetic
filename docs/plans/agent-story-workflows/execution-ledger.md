# Execution ledger

## 2026-10-02 — Baseline and M1 admission

Verified remote main at `ab5d75ce138c1b8e25cb5effc8f8800c0c6f9841` and created
clean branch `feat/agent-story-workflows`. Inspected graph/time projection,
harness/provider/store, embedding client, Pumas resolver, existing plans and
Coding-Standards. Existing graph memory and command history are retained.

M1 owns `crates/server/src/agent_workflow_harness.rs`, focused harness tests,
and this plan's records/index. Verification will test actual persisted run and
tool outcomes on provider failure, rejected manifest calls and tool failure.
Added the dedicated `agent_workflow_harness_tests.rs` owner and updated the
backend README to describe failure and cooperative cancellation semantics.

M1 implementation persists tool intent before execution, rejects invalid calls
without execution, records terminal provider/contract/tool failures and
cooperative cancellation, preserves completed call history, and permits a
completion turn after exactly the allowed tool calls. No automatic replay of
failed tools. If final history persistence fails, the returned error preserves
both the execution and persistence failure.

Verification on Rust 1.92.0: all 103 core tests and 237 server tests passed,
including eight focused harness tests. These used actual repository sources in
an isolated core/server workspace against the unchanged Pumas 0.6 pin
`8444b50df28c3e2bd8db58fb3645fa4dd8664b27`; native renderer/Tauri targets were
excluded to fit the cloud environment. Scoped rustfmt and core/server all-target clippy with `-D warnings` passed. Full launcher,
native desktop/UI, live model and independent review remain unverified. This is
backend-slice evidence, not full product acceptance.

## 2026-10-02 — M1 independent-review repair

Independent review found that call/result persistence errors could replace the
original executor/validation error before final run recording, including
misclassifying cancellation as failure. The repair keeps typed execution errors
inside terminal-persistence errors through call, result and run writes. The
finalizer recognizes cancellation through these wrappers without discarding the
persistence diagnostic. Rejected calls remain non-executing.

Added three regression tests exercising six injected failures: terminal call
UPDATE and result INSERT for tool failure/cancellation, and rejected-call INSERT
and result INSERT for validation failure. These join the existing final-run
UPDATE failure test. The harness now has 11 tests total (three baseline and eight
added by M1), not 11 newly added tests. Final verification and narrow re-review
are recorded when complete.

Final repair verification: all 103 core and 240 server tests passed; core/server
all-target clippy with `-D warnings`, scoped rustfmt and staged whitespace checks
passed. Same isolated core/server setup and unchanged original Pumas pin as the
initial checkpoint. Native desktop and live-inference limits remain unchanged.

Independent narrow re-review accepted the repaired M1 source with no remaining
findings and independently reran all 11 harness tests. Reviewed full patch
SHA-256: `0dd5db0a2be738cc6202b031a5dbbf6dc8f345da3036c458d92e7dc93a0c0e27`.
This acceptance covers M1 only. Hosted checks and broader product acceptance
remain separate gates.

## 2026-10-02 — M2a explicit fictional-time context checkpoint

Local branch `feat/story-time-context` starts from reviewed M1 commit
`52a856048a06c5b036a17677b58901e497914446` (draft PR #1). Added explicit optional
fictional-time inputs to generation, child planning and preview transport;
resolved sparse snapshots per field before prompts. Future/overridden values are
excluded; equal-time conflicts fail closed; omitted time and cleared facts are
explicit unknowns. No persisted canonical data or timeline positions are changed.

Verification: all 103 core and 245 server tests passed; core/server all-target
clippy `-D warnings` passed. Frontend Svelte check reports zero errors/warnings;
all 284 frontend tests across 57 files passed (including 13 API tests); scoped
ESLint/Prettier, Rust formatting, whitespace and decision traceability passed.
This verifies service and client-helper behavior, not a new human time-input UI,
Tauri runtime launch, persistent world-time mapping or feature-length outcome.
Independent slice review remains pending.

M2a independent review found a future-only field could disappear rather than
remain explicitly unknown before its first assertion. The repair retains its
identity without including future values, removes the unknown marker when an
eligible assertion exists, and preserves an existing baseline. New mixed-field
resolver and database-to-prompt regressions cover the boundary. Prompt wording
now explicitly distinguishes resolved fields from untimed graph edges; provenance
remains label/time, not unique assertion identity. Final repaired verification
is tracked separately from the earlier passing checkpoint.

Hosted M1 checks at exact head `52a856048a06c5b036a17677b58901e497914446`:
Frontend succeeded. Linux stopped on renderer `derivable_impls`; Windows stopped
on `wgpu-hal` Direct3D12 type mismatches; traceability lacked `rg` and therefore
misreported missing headings. These gates remain failed, not waived. Baseline
comparison and focused remediation are separate from this temporal slice.

Repaired M2a final verification: 103 core and 248 server tests passed; all-target
core/server clippy with `-D warnings` and scoped formatting passed. Six temporal
unit tests also passed in an actual-source focused verifier. The full test binary
initially received SIGKILL under shared RAM pressure; moving only its generated
target from tmpfs to disk-backed storage and using one build job allowed the
final complete suite to run. A new integration fixture was corrected to use a
schema-valid character motivation field; production schema validation was not
relaxed. Independent re-review accepted the bounded repair on 2026-10-02 and
independently reran 103 core and 248 server tests, including six temporal resolver
and four context projection tests. Reviewed full patch SHA-256:
`9ed558ba73056e72b297b37648680a0c5fb060f9a6901fc9398c80337de289e4`.
This acceptance covers field resolution only; hosted qualification and the later
Pumas, graph relationship timing, and timeline workflow slices remain open.

M1 merged into main at `72b8485025701fc18220c6522562e874704b6286`.
Post-merge push CI run 37091158926 completed successfully on 2026-10-03.
Forward integration retains the independently accepted M2a source; only plan
status prose required reconciliation. Field-only temporal scope, label/time
provenance, and later Pumas/timeline integration limitations still apply.

## 2026-10-03 — M3a reference retrieval custody

Reverified main at `3757edf6e4050aa211cb6ac7d9f991167c2af74e`. M1/M2a are merged;
this slice implements the source-bound retrieval prerequisite described in the
plan. Added exact-source publication tickets, project index invalidation,
query-scope fencing, configured representation identity checks and validated
vector ranking. Persistent canon is unchanged. Tests exercise deletion after
inference starts by delivering a completed vector after the real deletion service.
Qualification and independent review are pending until recorded below.

Root review identified a late scope-capture hole: a queued generation could bind
to a reopened project's new index if its database path stayed the same. The
repair captures retrieval epoch before project snapshot I/O in single/batch
admission and carries it to attach. Project activation publishes path alongside
project/index under the project guard. Caller-level tests deliver delayed query
results after reopen/deletion and clear preexisting context on every rejection.
This fences optional retrieval only; the existing broader streaming-generation
persistence/cancellation lifecycle remains outside this slice.

Local qualification on Rust 1.92.0: core/server all-target clippy with
`-D warnings` passed against these actual source files in the bounded verification
workspace and unchanged Pumas `8444b50df28c3e2bd8db58fb3645fa4dd8664b27`.
Changed-source rustfmt, staged whitespace and decision traceability passed.
Local tests did **not** execute: the first attempt encountered unavailable ORT
binary download; a retry with the already-installed official ONNX 1.24.2 library
was SIGKILLed compiling Pumas under shared-memory pressure. No local test pass is
claimed. Exact-head hosted workspace/frontend tests and independent re-review
remain pending. Native desktop, actual model inference, saved-reference reindexing
and immutable model-revision identity are unverified/out of scope.

Independent root source re-review accepted the bounded retrieval correction on
2026-10-03, including admission epoch propagation, coherent activation and the
caller-level regression coverage. Publication is cleared; hosted execution is
still a separate pending gate. Broader generation persistence is not covered.


### 2026-10-04: M4a recovery and shared timeline write guard

- Recovered PR4 head `c04711ba61e55be3348a18347db4f70b9d547fa5` into an
  isolated checkout; no unpublished M4 implementation was found. Pumas stays at
  the repository's unchanged pin `8444b50df28c3e2bd8db58fb3645fa4dd8664b27`.
- Added transaction-local snapshot checks to all nine timeline history writers
  and focused regression tests. This is the state-custody prerequisite described
  above, not completed M4 or immutable long-lived proposal revision checking.
- Verification pending. Rust formatter executed successfully. The initial server
  test attempt reached dependency downloads, then its execution review was
  cancelled; no compile/test success is claimed. Publication remains on hold.

- Recovery validation resumed with unchanged dependency pins. The default ORT
  CDN download failed with `io: connection refused`; the matching official
  `onnxruntime==1.24.2` PyPI wheel supplied the native runtime through ort-sys's
  supported `ORT_LIB_LOCATION`/dynamic-link configuration. Wheel SHA256:
  `09aa6f8d766b4afc3cfba68dd10be39586b49f9462fbd1386c5d5644239461ca`.
- Final code passed `cargo test -p eidetic-core -p eidetic-server --all-targets`:
  **103 core + 268 server tests**. Includes nine new guard regressions and
  real project save/load, separate Y.Doc bytes, pre-initial-save rejection,
  stale writer rollback, refresh, replay and structural command acceptance.
  No native renderer, live inference or whole-workspace pass is inferred.
- Strict core/server all-target clippy passed with `-D warnings`; workspace
  rustfmt check, whitespace check and decision traceability passed. Publication
  and exact-head hosted/native qualification remain pending parent coordination.


### 2026-10-04: M4b descendant range history

- Separate local milestone above M4a; records the target and all actually changed
  descendants atomically, leaving unchanged/unrelated clips without new revisions.
- Corrected per-object history ordering across events. Regression reproduces
  parent-resize then direct-child-edit with reversed caller timestamps.
- Actual source all-target tests passed: **103 core + 273 server**, including
  five new range-history regressions. Strict core/server all-target clippy passed
  with `-D warnings`; unchanged Pumas and official ORT1.24.2 as above.
- Source re-review, publication and exact-head hosted qualification pending.
  No native interaction, full undo or agent editing acceptance is claimed.

- Follow-on source audit clarified existing node locks prevent AI regeneration,
  not manual geometry edits. No structural locking change was made or inferred
  from the earlier open M4 lock-policy note.

- Audited persistence, project database ownership, PDF export and history write
  sites: supported paths append history and do not reinsert or reorder events.
  Added real broad-save/reopen ordering regression; final tests and clippy passed.
  Documented that SQLite row order is local append-only storage, not a portable
  revision clock; future history rebuild/compaction needs an explicit contract.

### 2026-10-04: PR6 WAL interleaving and interrupted-write qualification

- Selected isolated task branch `test/pr6-wal-rollback` from exact PR6 head
  `03b47adc95b673daef5ccf3a1e17316bcf28143a`; verified qualified PR5
  `a2a2ff47a2a4d90da2498b0a16b371125fd147d2` is its ancestor. Environment
  initially selected main `3757edf6e4050aa211cb6ac7d9f991167c2af74e`.
  Pumas remains `8444b50df28c3e2bd8db58fb3645fa4dd8664b27`; no dependency,
  credentials, permission or network configuration changes.
- Added three real-project SQLite regressions: an overlapping WAL reader and
  writer with exact `SQLITE_BUSY_SNAPSHOT` promotion rejection; two-connection
  stale range rejection followed by reviewed reload, explicit resubmission and
  identity-preserving replay; and descendant-write interruption using both
  SQLite `ABORT` and `ROLLBACK`. Faults occur after all pending history and an
  ancestor update exist. Assertions cover current-state rollback, retention of
  a previously committed event, no leaked command/revision/field rows, reopen,
  separate Y.Doc bytes and reuse/replay of the failed command identity.
- The held-reader test exercises the guard/store transaction boundary directly;
  the stale and interruption tests call the shared range history writer. These
  deterministic interleavings do not claim service scheduling, process-kill,
  power-loss, native desktop or model inference qualification.
- Passed: `cargo test --locked -p eidetic-core` (103 tests); UI `npm run test`
  (284 tests / 57 files), `npm run check` (zero errors/warnings), `npm run lint`
  and `npm run format:check`; workspace rustfmt, whitespace and decision
  traceability checks.
- Passed compile-only qualification with the upstream-supported download skip:
  `ORT_SKIP_DOWNLOAD=1 cargo check --locked -p eidetic-server --tests` and
  `ORT_SKIP_DOWNLOAD=1 cargo clippy --locked -p eidetic-core -p eidetic-server
  --all-targets -- -D warnings`. This deliberately does not link or execute
  native ONNX Runtime, and is not a server test pass.
- Normal `cargo test --locked -p eidetic-server --lib
  timeline_command_history::range_tests` was blocked at ort-sys's ONNX 1.24.2
  download: `cdn.pyke.io` CONNECT proxy returned 403. New server tests remain
  unexecuted in this environment pending the parent's approved network update.
  Rerun that command, then the core/server all-target tests on the final head.
- No production defect was demonstrated; changes are tests and documentation
  only. Locks retain AI content-regeneration semantics. Split-note duplication,
  crossing-child cut policy, and long-lived proposal identity/revision
  preconditions remain separate design work. No merge, PR metadata change,
  external review request or CodeRabbit invocation was made.

### 2026-10-04: Core gap projection after overlapping edits

- Isolated branch `fix/timeline-overlap-gap-projection` starts at exact PR6
  `03b47adc95b673daef5ccf3a1e17316bcf28143a`, preserving PR5 ancestry.
  The test-only WAL successor `48dd8408923c6667591e918333c6ccd2380dbd86`
  is not included; its runtime qualification is delegated separately.
- Fixed core gap discovery so contained clips cannot move the occupied endpoint
  backwards. Equal-time boundaries use stable node identity ordering. No server,
  frontend, dependency, geometry mutation or product-policy change.
- All four new regressions fail with the original gap implementation (103 pass,
  four fail), then pass with the fix: **107 core tests**. Includes a public resize
  followed by renderer projection and checks for thresholds, level isolation,
  nested/overlapping/touching clips and reordered storage.
- Passed strict core all-target Clippy, **284 frontend tests**, Svelte check
  (zero errors/warnings), frontend lint, formatting and production build,
  workspace Rust formatting, decision traceability and whitespace checks.
- Server/native tests were not run for this slice. Dot qualification should
  run the server/native gates and visually confirm the gap overlay after an
  overlapping resize. No full-workspace, native UI or completed M4 claim.

### 2026-10-04: Timeline projection cache session custody

- Isolated branch `fix/timeline-projection-session-custody` starts at
  `e208f4ce74521dbdfd30759118c2edc181f9ef29`; the separate gap-projection
  branch remains unchanged for dot review. No WAL successor changes are included.
- Reproduced an old project refresh replacing a new project's cache using the
  actual frontend module with deferred transport responses. A cache generation
  now owns all refresh/command state updates; clear invalidates old completions.
  Within a generation, pending counts outstanding work, versions order cache
  replacement, and the latest-started request owns shared error state.
- All **26** deferred lifecycle tests fail against the original store and pass
  with the fix. They cover all nine command bridges, refreshes, clear/new loads,
  obsolete errors/finalizers and same-session command/refresh concurrency.
- Final **310 frontend tests** pass. Svelte check reports zero errors/warnings;
  lint, formatting, production build, decision traceability and whitespace
  checks pass. An initial full-suite documentation audit failure was corrected
  by keeping test-file documentation outside the store-module inventory table.
- The required pre-push gate runs native workspace tests; publication is handed
  off as a verified bundle without retrying blocked downloads or bypassing hooks.
  Backend session custody, caller continuations, other projection stores and
  native interaction validation remain outside this frontend cache milestone.
- Completion contract: finish this cache repair, prove deferred behavior and
  frontend gates, preserve the gap branch, and provide verified source transfer.
  No callable native Goal-setting tool is available; durable Goal activation is
  not claimed. Runtime model/reasoning selection is not exposed to this task.

### 2026-10-04: Keyboard timeline caller continuation custody

- Isolated branch `fix/timeline-keyboard-continuation-custody` follows
  `abae7ca83f41470f392ec8b81cf6f61513a415b7`; both independently qualified
  gap/cache branches remain unchanged.
- Four deferred delete/split regressions fail against the original keyboard
  adapter (four existing tests still pass). They reproduce clearing an intervening
  selection and clearing the same node ID after an editor-session reset.
- Editor reset now advances a lifetime generation. Successful delete/split clear
  only their still-selected target in that generation. The shell uses the keyboard
  adapter's guarded failure notifier, preserving current-session failure messages
  while suppressing old-session notifications.
- **318 frontend tests** pass, including eight added continuation tests covering
  both destructive actions, rejection preserving selection and notification
  ownership. Typecheck (zero errors/warnings), lint, formatting, build, decision
  traceability and whitespace checks pass. Native qualification remains delegated.
- This handles keyboard caller effects only. Backend admission, other manual
  callers, generation lifecycle and renderer custody remain separately scoped;
  no cut/child-content semantics are changed.

### 2026-10-04: Backend timeline command admission custody

- Isolated branch `fix/timeline-command-admission-custody` follows
  `abae7ca83f41470f392ec8b81cf6f61513a415b7`, separate from the keyboard
  continuation change. Frozen gap/cache branches are unchanged.
- An offline dependency-free harness executing the extracted production admission
  function with deferred/failing persistence reproduced returning B's mirror for
  captured path A after replacement. With the fix it returns path A and mirror A.
  This uses real core project types but mocked I/O/state: it is not server runtime
  evidence. Create-child now forwards its captured admission to create-node work.
- Added two deterministic native regressions using a one-thread blocking pool:
  failed-load fallback remains bound to A, and queued child creation with copied
  node IDs writes A rather than active B. These tests are **not executed here**.
- `ORT_SKIP_DOWNLOAD=1 cargo check --offline --locked -p eidetic-server --tests`
  and strict server/test Clippy passed. These prove compilation/lint only; they
  do not supply or exercise ONNX. No denied download was repeated.
- Dot handoff: run `cargo test -p eidetic-server admission_tests`, then applicable
  full server/native gates on this source. Verify A/B current-state and history
  effects. Post-commit event/Y.Doc/autosave and save-path publication custody are
  explicitly not repaired by this admission-only change.

- Independent review clarified the admission gate wording: its tests establish
  B's persisted database nodes and active mirror/path identity only. Shared
  Y.Doc, events and autosave are not isolated by that repair. This documentation
  correction is a successor; `898cb310` stays frozen for independent review.

### 2026-10-04: Timeline post-commit and document custody successor

- Branch `fix/timeline-postcommit-session-custody` follows frozen `898cb310`.
  Documentation-only commit `2991846` narrows the admission claim before this
  implementation. PR6 and the separately qualified gap/frontend/keyboard/resize
  branches are untouched.
- A harness executing the actual original post-commit notes body with the real
  Y.Doc actor demonstrated late A notes entering B, a timeline event and a save
  signal. Separate actor tests reproduced merge/empty-load retention: two new
  custody tests fail on baseline, while invalid-load/subscription tests pass.
- Added the participating session gate and queued-request identity check; project
  replacement/reopen and save-as invalidate queued admissions. Document, event and
  save publication stay under the admitted gate. Autosave holds the same gate
  from mirror/path snapshot through document serialization and persistence.
- Y.Doc replacement now restores fresh state, preserving invalid-load errors and
  update observation. Fallback resets first. Timeline document sends await capacity
  and report closed-channel errors, identifying failures after a durable SQL commit.
- **10 actual isolated Y.Doc tests pass**, importing production Y.Doc/supervisor
  modules with Tokio/Yrs/core dependencies and no ONNX/Pumas dependency. This is
  component runtime evidence, not a full server test pass.
- Skipped-ORT offline server/test compilation and strict Clippy pass. Seven new
  native scheduling/lifecycle tests are **unrun here**: six timeline tests plus one
  autosave test. They seed A/B histories and compare concrete rows; they do not
  claim arbitrary imported-history preservation or end-to-end client isolation.
- Dot's admission history patch `libfile_dd23e13015a8819185780824faaa4f5a`
  could not be materialized: supported Library transfer reported `download failed`
  on `oaisdmntpreastus2.blob.core.windows.net`, without a status. No retry/bypass;
  raw patch transfer requested. Its reported 275-server-test qualification applies
  to the earlier admission source, not this successor. Patch incorporation pending.
- Native handoff: run timeline post-commit, autosave, admission and Y.Doc custody
  tests, then full server/native gates. Check seeded history, explicit conflicts,
  document-channel errors and publication ordering. Other producers and wire-level
  pre-admission intent tokens remain out of scope. No merge readiness is claimed.

### 2026-10-04: Transition persistence repair after independent native review

- Frozen `de4c186b` remains the comparison source. The parent reports 107 core,
  286 server and 11 focused custody tests passing there, but two deterministic
  transition regressions failing: an earlier real A document blob becomes stale
  after committed notes and immediate A→B→A reopening; Save As uses stale mirror
  notes instead of committed source timeline notes. These are parent/native
  observations, not cloud server execution.
- Source comparison against `898cb310` confirms missing outgoing flush and mirror
  Save As predate the custody repair. Fresh replacement in `de4c186b` exposes the
  stale stored-document boundary; passing gate tests did not qualify persistence.
- Repair branch `fix/timeline-transition-persistence` follows `de4c186b`: create/load
  explicitly serialize and persist the outgoing session before document replacement.
  Save As flushes and reloads that source database, then saves its authoritative
  timeline/arcs with the current blob to a new destination. Active mirror refresh
  touches SQLite-owned timeline/arcs only. Existing different Save As destinations
  return a conflict to avoid retaining unrelated destination timeline/history.
- Required document serialization errors propagate instead of saving without a
  blob; autosave logs and skips that write, preserving the earlier saved document.
  Outgoing read/write errors prevent publication of a new active session. Direct
  flush completion is awaited; no debounce/save-signal completion is inferred.
- Admitted timeline/lifecycle work moves its gate into a supervised task through
  blocking persistence and publication. Caller cancellation preserves that task;
  admission waits are still cancellable. Backend shutdown aborts tasks. Other
  producers, process crashes and external collaboration remain unqualified.
- Eight additional native tests compile but are **unrun in cloud**: seven transition
  tests cover earlier-blob reopen, authoritative Save As, serialization/write errors,
  existing destination conflicts and cancelled transition/notes callers; one autosave
  test checks that serialization failure preserves its stored blob. The current
  focused suite has 15 timeline/autosave tests plus four document custody tests.
- An isolated harness copies the exact new ownership helper and imports production
  Y.Doc, backend supervisor and error modules with a minimal state wrapper and no
  Pumas/ONNX dependency. Its two cancellation cases fail with the old caller-owned
  gate pattern; all three cases pass with the repair, including error forwarding.
  This is component evidence and does not qualify actual AppState/server runtime.
- `ORT_SKIP_DOWNLOAD=1` offline server/test compilation and strict Clippy pass;
  native transition and full server retesting remain with the parent.
- Supported one-shot materialization of the combined native regressions
  `libfile_ab530ad64488819185e6f6bda07380a7` (SHA-256
  `5e7696dc0da6567d5a5d9f303f71081211870e25a2f214dd409dfe7b8d0a892b`)
  and rebased admission history patch `libfile_5d551b86f78481919f1d977a3a7a1c53`
  (SHA-256 `c75d31d18b18466e88c72b0d05b411b06afe041ba254d52a01ce86a695a256e1`)
  failed with `download failed` on `oaisdmntpreastus2.blob.core.windows.net`, with
  no exposed HTTP status. No retry/bypass; exact raw patch text requested. Neither
  patch is incorporated or claimed qualified in this successor. Initially-empty
  admission history qualification does not establish arbitrary imported history.
- Separate resize documentation successor `608e22bb` follows frozen `c81d6e877`.
  Its core README records `TimeRangeOverflow` and atomic validation; traceability
  passes across the complete `e208f4ce..608e22bb` resize change. The parent reports
  113 core tests/Clippy passing for frozen resize source; no implementation changed.

### 2026-10-04: Bounded supervisor records during normal editing

- Frozen persistence source `393786a93d1c973c40ce3d0148a8f770d6cc9c9b` remains
  unchanged. Parent/native qualification reports both exact earlier transition
  failure proofs passing unchanged, 107 core / 294 server tests, all 19 focused
  custody cases and strict Clippy/format/traceability. A separate test-only worktree
  adds unchanged transition proofs and enhanced initially-empty A/B history checks:
  296 server tests/Clippy pass. These are parent reports, not cloud execution, and
  the test-only patches are not silently imported into this source.
- Native source review identified completed `BackendTaskSupervisor` handles growing
  with each command unless desktop smoke calls `active_task_count`. This successor
  changes normal `spawn` to reap finished handles and poll their join results;
  successful/cancelled completions are observed and named panics are logged.
  Smoke counting uses the same observer rather than silently dropping results.
- Registration and reaping share the registry lock. Every unfinished handle remains
  owned for admitted completion, caller cancellation retention and shutdown joining.
  Existing `shutdown_all` abort/join and explicit `abort_all` behavior are unchanged.
  The last completed tail can remain until the next spawn/count/shutdown; cleanup
  does not require smoke introspection and total edit history no longer accumulates.
- Three new production-module supervisor tests all fail against `393786a` with only
  a passive test accessor added: repeated spawn records accumulate, finished records
  obscure the running-work bound, and normal admission does not report reaped panic
  results. With the repair, all three pass, including joined teardown of retained
  running work and captured named panic diagnostics.
- **18 isolated component tests pass** with production supervisor/Y.Doc/error
  modules plus the unchanged ownership helper and a minimal state wrapper. They
  include the five supervisor tests, ten Y.Doc tests and three cancellation/error
  helper cases. No ONNX/Pumas dependency is present. This is component runtime
  evidence, not actual AppState/server qualification.
- Added a native normal-runtime regression: 64 actual notes commands, passive
  registry observation, correct final SQLite/document notes and 65 seeded+new
  command records. It never calls `active_task_count` or any pruning observer.
  This regression compiles but remains **unrun in cloud**.
- `ORT_SKIP_DOWNLOAD=1` offline server/test compilation and strict Clippy pass.
  Source/format/traceability and repository hooks are checked for this successor.
  Parent owns native supervisor/focused/full server retesting and integration;
  no native runtime, merge readiness, merge or external review claim is made.

### 2026-10-04: Atomic proportional resize validation

- Isolated sequential branch `fix/timeline-resize-validation` follows tested gap
  head `e208f4ce74521dbdfd30759118c2edc181f9ef29`. Parent cleared the shared
  file for the resize method; frozen gap/custody/admission branches are unchanged.
- Real core execution reproduced accepted zero-duration children and arithmetic
  panic on a valid large range. New resize tests against the original method:
  four fail, two pass. They also expose large no-op millisecond rounding.
- Resize now validates source/proposed ranges before publishing any mutation,
  using checked integer scaling. Collapse or arithmetic failure returns a typed
  error; valid ranges preserve exact integer endpoints and existing clamp rules.
- **113 core tests** and strict core all-target Clippy pass. No native server test
  evidence is claimed. Dot should run timeline range-history/rollback regressions
  and native gates with the new core behavior, especially rejection of collapse.
- Crossing-child/split semantics remain unresolved and are not selected here.

### 2026-10-04: Accepted follow-up integration candidate on merged PR6

- Branch `fix/integrate-accepted-timeline-repairs` starts at exact main
  `a6bd4c1830dd1a500a9ededbcc8056bd358ccda7`, the merged PR6 head. Local candidate
  merge parents retain accepted keyboard `01befb0505714fce94ec1b1c1b0b86ce238945f1`,
  server custody through `ea404b9623320db7020abb1b376143a4063ff186` (tree
  `a61d64a634460717dd85100c802a51dbfc81666c`) and resize documentation/code through
  `608e22bb10aab9affa385aa5f784e3d41b8df75f`. Frozen source refs and main are not
  rewritten. This is local candidate composition, not a merge to main.
- Main already contains rebased gap/cache implementations; their production blobs
  match the accepted originals. They were not reapplied. Main's WAL/interleaving/
  rollback test file is retained byte-for-byte; its original qualification records
  remain a verbatim ledger prefix. Actual merge conflicts were documentation only.
  Follow-up records/plan sections are included once; runtime Rust and frontend
  source required no conflict edits or new feature decisions.
- Whole-repository Rust audit covers **248 files**: the candidate is exactly main
  plus accepted server/resize blobs. There is no Rust beyond accepted composition.
  Rust manifest SHA-256:
  `7552b3e97961de12a6cf87d6b4754ff2fd5f661133cdd19f9306894b75785283`.
  Keyboard production/test files match `01befb0` exactly. Dependency manifests,
  lockfiles and main projection-cache source/tests are unchanged. Evidence records
  the final candidate commit/tree and per-file source/blob identities externally.
- Combined cloud gates pass: **113 core tests**, strict core all-target Clippy,
  **318 frontend tests** (58 files), zero-error/warning typecheck, lint, formatting
  and production build. Server/test compilation and strict Clippy pass offline
  with `ORT_SKIP_DOWNLOAD=1`. No server/native tests executed here; compilation
  remains distinct from ORT runtime evidence.
- Parent reports native acceptance of exact `ea404b9`: 107 core / 298 server tests,
  strict Clippy/fmt/traceability; test-only unchanged reopen/Save As/history proofs
  pass 300 server tests/Clippy, including normal 64-command retention. Resize
  `608e22bb` retains previously native-tested code and accepted traceability.
  These accepted slices do not substitute for parent qualification of this combined
  tree, including resized descendants through server history/rollback paths.
- Source publication needs preserved Git objects. GitHub read-only lookup returns
  422 (`No commit found for SHA`) for `ea404b9`; the remote candidate branch is
  absent. The checkout pre-push hook runs native `./launcher.sh --test`, which is
  assigned to the parent. No hook/network bypass or repeated CDN attempt is made.
  Provide the verified candidate bundle, ancestry/blob audit, gate logs and a
  repository-template draft description for parent publication/native retesting.
  No external PR, review/bot request, main merge or merged PR6 update is claimed.

### 2026-10-04: Manual screenplay editing and canonical agent context candidate

- Recovered the original accepted integration from the two available exact
  parents and the owner-supplied commit bytes. Both tree
  `ea1c975c59c33535304cf226df99f008ce1900bc` and commit
  `9f44a974bc7830e721237b3ccb295095c1bed5f6` matched before feature work.
- Isolated candidate branch `feat/manual-screenplay-memory`: focused block
  edit/save UI, text-only canonical command with expected write identity and
  transaction-local lock checks, sparse old/new text history, source-bound
  screenplay context shared by prompt preview and generation, prompt invalidation.
- Executed 22 tests in an isolated Rust harness importing the actual repository
  SQLite/history/script-command/context/prompt modules, including four new
  manual-authoring regressions. It links no Pumas/ONNX runtime and is not the
  full server test suite. The saved/reopened exact Unicode/whitespace text,
  distinct segment/block write identities, old/new history, stale/ABA refusal,
  replay preservation, late-added locks and bounded/deleted context cases pass.
- Core suite: 113 passed. Frontend suite: 338 passed. Svelte check, lint,
  formatting and production build pass after correcting the new event test's
  mock interface. Server test compilation and Clippy pass offline with
  `ORT_SKIP_DOWNLOAD=1`; these are compilation evidence only.
- Native AppState regression calls the real manual edit service, preview and
  shared generation-admission helper with a stale mirror and no model/provider
  call. It compiles but remains unexecuted here because no ONNX library is
  available. The desktop crate offline check stopped at uncached
  `ab_glyph v0.2.32`; no dependency/network retry was attempted. Native desktop
  command registration, runtime tests and user-flow qualification remain pending.
- Recovery proposal and accepted async repairs remain separate/preserved. No
  inferred-world acceptance, timeline semantics, remote upload, PR merge,
  external review request or network-policy change was performed.

### 2026-10-04: Captured screenplay lineage and Needs review candidate

- Parent independently qualified frozen first feature `9fe7a4a5`: 113 core,
  314 server and 338 frontend tests, desktop compile/smoke, concurrent edit winner,
  project reload, preview and late-lock service fixtures passed. Exact whitespace,
  newlines, Unicode and separate revisions were retained. Parent accepts its
  source/native-service behavior; graphical typing/save/reopen remains unqualified
  due the extracted runtime's hardcoded WebKitNetworkProcess path. No workaround
  was attempted. This report does not qualify the descendant below.
- Isolated descendant branch `feat/script-input-impact` retains that frozen
  parent. Successful output now carries the exact screenplay inputs supplied to
  generation into the same SQLite transaction as output/history and existing
  DerivesFrom dependencies. Additive generation and revision-binding tables retain
  old unbound dependencies. Historical text, document, placement, source identity
  and separate block/segment write events are validated; conflicting input
  revisions are refused. Model-time edits are never silently rebound to latest.
- Derived script impact compares the latest successful generation's bound
  inputs with current source writes. Changed/deleted causes retain consumed
  event identities and historical excerpts; old lineage remains auditable after
  explicit refreshed generation. Own previous drafts retain audit bindings and
  their intentional replacement does not self-invalidate. Unknown input history
  differs from a known empty input set. Authored text, locks, canonical segment
  status and explicit proposal acceptance remain unchanged by review projection.
- Executed **34 isolated source-module tests**, including eight lineage fixtures:
  A-to-B/unrelated-C; source edits during generation; replay/signature conflict;
  deletion and explicit refreshed binding; SQL failure after output/history rows
  with complete rollback; fabricated evidence; unavailable versus empty lineage;
  conflicting revisions. This harness imports the actual SQLite/history/script/
  dependency/context/projection modules; it links no Pumas/ONNX runtime and is
  explicitly not the full server suite. Contract round trips exercise bound
  dependency and deleted-source impact shapes while preserving absent fields.
- Core suite: **113 passed**, strict core Clippy passed. Frontend suite:
  **339 passed**; typecheck (zero errors/warnings), lint, formatting and production
  build passed. New frontend fixture preserves deleted-source impact and exact
  authored output through cache refresh while refusing older clean responses.
- Server tests compile and strict all-target Clippy pass offline with
  `ORT_SKIP_DOWNLOAD=1`. New native persistence fixture and three failed/empty
  stream fixtures (now carrying real captured inputs and checking unchanged
  lineage counts) compile but were **not executed here**. Parent must qualify
  the full native server and graphical flow for this exact descendant. Earlier
  CDN `cdn.pyke.io` HTTP 403 remains preserved; no artifact retry or bypass.
- Both authored output and generation lineage roll back on failed atomic writes.
  This is SQLite transaction evidence, not process-kill/power-loss qualification.
  New review UI is read-only; broader propagation, timeline semantics, project
  switching/recovery and inferred world updates remain outside this slice.
- No GitHub publication, merge, external review request, network-policy change
  or model-provider call was performed. Publication hold remains unchanged.
  Commit milestone and verified source/evidence package are delivered separately
  through Library for parent review and native qualification.

### 2026-10-04: Targeted screenplay propagation review checkpoint

- Preserve frozen `0b687a8168c3499cfbcc356f814958d7ee11859b`. Parent reports its
  independent acceptance with 113 core / 323 server / 339 frontend, checks/build
  and real deletion-after-capture and placement-only probes. Source restored
  clean. Graphical Needs review remains unqualified under the WebKit blocker.
- Inspected existing proposal contracts/store/review/accept commands and desktop
  transport before implementation. Missing feature: connect a proven Needs review
  cause to a targeted pending propagation proposal, with stale acceptance refusal.
  Existing screenplay acceptance validated locks but carried no preview revisions.
  Continue on isolated descendant `feat/script-impact-review`, without changing
  frozen acceptance, publication hold or parked project-switch/recovery work.
- Preview captures a proven cause, generated output block, current target text/
  placement writes, canonical screenplay inputs and resolved graph evidence in
  one snapshot. A moved source is explicitly included outside normal continuity
  bounds. Deleted-source evidence remains explainable. Optional output identity
  makes the preview target the actual affected generated block, protecting other
  blocks in the same segment from falsely resolving that output's impact.
- Reuse the configured provider, canonical screenplay formatter and current
  graph/fictional-time resolver. The targeted prompt consumes only captured
  screenplay and resolved graph evidence. No unbound timeline prose, recaps,
  world inference or presentation-derived fictional time is supplied. UI query
  time is absent; typed requests also support explicit fictional time. No live
  model/provider call was made during qualification; quality remains unqualified.
- Strict provider stream completion refuses partial/error/empty drafts. Successful
  preview commits only an existing pending propagation proposal and its additive
  typed evidence binding. Original request replay skips provider invocation.
  Generic retarget/amend commands refuse bound previews; request a fresh preview.
- Explicit acceptance revalidates source/target/placement writes, current cause,
  graph evidence and pending proposal contents inside the SQLite writer transaction.
  Recheck protected spans and the canonical source node's content-regeneration
  lock. Target block/span, proposal status, sparse history and refreshed actual
  input lineage commit together. Preserve source edits, unrelated C, other blocks
  and document/segment metadata. Rejection records status without discarding
  authored text or clearing the cause; stale refusal keeps the proposal pending.
- Executed **137 isolated actual source-module tests**, including **12 new**
  targeted review/provider fixtures. Linked-scene accept/reject, source/target ABA,
  edits during provider I/O, invented cause/wrong block, full acceptance rollback,
  replay/retarget refusal, graph-time/stale world context, late span/node locks,
  deleted-source explanation and failed/empty provider output all pass. The harness
  now includes existing bible/temporal/propagation regressions and still links no
  Pumas/ORT. It is explicitly not the full server suite.
- Core: **114 passed**, strict all-target core Clippy passed. Frontend:
  **344 passed**; zero-error/warning Svelte check, lint, formatting and production
  build passed. New transport/store fixtures preserve review state on failed
  preview and do not send acceptance. SSR fixtures render exact whitespace/newline/
  Unicode proposal text, safely escaped markup, explicit controls and refusal to
  preview an unidentified output. SSR does not qualify graphical clicks.
- Server tests compile and strict all-target server Clippy pass offline with
  `ORT_SKIP_DOWNLOAD=1`; compilation remains distinct from native execution.
  Tauri registration/adapter and the AppState preview service require parent native
  qualification on the exact checkpoint. Known uncached desktop dependency and
  graphical runtime limitations were not bypassed. No ONNX retry or policy change.
- Commit hooks and decision traceability use the frozen lineage parent as the
  feature comparison base. Deliver the preserved-ancestry bundle, patch, exact
  identities and verification evidence through Library. No GitHub publication,
  cancelled-upload retry, external review request or merge is performed.

## Provider stream integrity repair after native targeted-preview review

- Independent native review of frozen `a372d4f` demonstrated two real transport
  failures: a truncated declared HTTP body persisted an unfinished proposal, and
  a split SSE event lost its first text while reporting success. Keep that
  checkpoint frozen; repair both existing adapters on a separate descendant.
- Replace stateless per-chunk parsing with shared byte-preserving SSE framing.
  Propagate HTTP body, provider error-frame, UTF-8 and JSON errors; require the
  completion marker and clean HTTP EOF. Keep first-choice event order and fixed
  production provider configuration. Full collection also propagates errors.
- Executed **143 isolated actual-source-module tests**, including **five new
  loopback HTTP transport tests** covering both adapters. Split JSON/Unicode,
  ordered tokens, clean termination, truncated bodies before/after `[DONE]`,
  missing completion, provider/malformed/invalid-UTF-8 events, and full collection
  pass. Actual adapter-to-preview-to-SQLite fixtures preserve authored text and
  history on failure, and record the complete clean text as a pending proposal.
  The harness uses a configuration-only shim, no Pumas/ORT or AppState runtime.
- Added two AppState public-service regressions, selected by
  `independent_real_service`: incomplete HTTP creates no proposal/history/event;
  complete split SSE persists full pending text and preserves authored canon.
  These compile here, but require native execution. Full server test compilation
  and strict all-target Clippy pass offline with `ORT_SKIP_DOWNLOAD=1`; neither
  constitutes server runtime evidence. No ONNX retry or policy workaround.
- The supplied native repro Library package could not be downloaded by the
  supported consumer helper in this environment. Its Library identity remains
  `libfile_ee025b5c35048191a43d70fe0ba8121e`; native review retains the original
  failures. The repair includes equivalent public-service probes for the parent
  to execute with real local ORT and loopback proxy exclusion.
- Ordinary timeline placement remains inspection-only and separate. No model
  call, provider quality claim, project recovery, GitHub publication or merge.

## Timeline range-to-screenplay placement slice

- Parent independently accepted frozen transport repair `11c437f`: both original
  unchanged real-service probes passed, along with **114 core / 341 server**
  native tests. Keep that milestone frozen; this feature is a separate descendant.
- Confirmed ordinary UI move/resize/keyboard range commands reach the timeline
  range service, whose old writer changed timeline nodes only. Canonical segment
  ranges/context and consumed placement bindings retained the earlier placement,
  and the old publication emitted no script event.
- Synchronize every live segment bound to the edited node or changed descendant
  to its new source range, under the same acquired writer transaction. Add only
  changed placement fields with exact old/new values and the same event identity.
  Preserve authored text, block revisions, spans/locks, status and other metadata.
  Deleted segments/documents and unbound segments do not participate; unchanged
  ranges add no segment revision. Existing timeline snapshot/replay guards remain.
- Extract the shared transaction-local revision inserter so derived segment
  revisions can be planned under the writer lock. Historical input validation
  replays sparse segment fields through the exact captured event using the
  existing append-only ordering boundary; later writes are never consumed
  accidentally. No schema version or new graph backend is introduced.
- After recorded commit, publish existing `ScriptChanged` beside `TimelineChanged`.
  Existing frontend handling invalidates prompt context and refreshes screenplay
  and history/review. Failure and identical replay publish neither event.
- Executed **203 isolated actual-source-module tests**, including **six new**
  deterministic two-scene fixtures: moving A after B reorders canonical context
  and gives B only a consumed-placement cause; explicit reject preserves all
  canon; fresh accept updates B alone and refreshes lineage. Retime/placement ABA
  refuses old preview; replay preserves later authoring; parent resize updates
  both scenes; injected segment failure rolls back nodes/script/history; sparse
  end-only placement validates captured historical input after a later move and
  rejects forged evidence. Optional fictional time remains explicit (`42`) and
  never comes from presentation placement.
- The harness includes actual persistence and existing range/guard tests, including
  real SQLite WAL promotion, stale writer/reload/replay, interrupted descendant
  rollback/reopen and broad-save event-order regressions. It links no Pumas/ORT
  and does not execute AppState; this is not the full server suite.
- **344 frontend tests passed**. Full server test compilation and strict all-target
  Clippy pass offline with `ORT_SKIP_DOWNLOAD=1`, which is compile-only evidence.
  Added native public range-service success/replay/rollback publication regression
  `native_range_edit_publishes_script_change_only_after_atomic_success_and_not_replay`;
  its execution and complete server qualification are delegated to native review.
- Preserve milestones and publication hold. No graphical/model-quality claim,
  ONNX retry, policy workaround, project recovery, hierarchy reparenting, external
  review request, GitHub publication or merge.

## Native timeline service fixture metadata correction

- Native review of frozen `dfd9406` passed **114 core / 347 server** tests; the
  remaining range-service test failed during fixture setup because its copied
  node-bearing database lacked the required `episode_structure` row. No
  production placement defect was established; six placement fixtures passed.
- Initialize the canonical episode table and row 1 from the fixture project's
  template name and serialized structure segments before `VACUUM INTO`. Keep
  production persistence unchanged and every original service/event/replay/
  rollback assertion byte-identical. The native reviewer reported this exact
  correction passes with the original assertions.
- Locally executed the corrected source setup prefix through actual SQLite,
  project save and reopen; canonical episode metadata and two-scene screenplay
  lineage are retained. Server tests compile and strict Clippy pass offline with
  `ORT_SKIP_DOWNLOAD=1` (compile only); exact descendant native execution remains
  with the same reviewer. No AppState substitute, HTTP bridge, graphical claim,
  model call, publication or merge.


## Manual first-block creation and screenplay appends

- Started a fresh `feat/screenplay-story-memory` worktree from verified merged main
  `27680cb52c2b8bdfcdc478bb7f35801aa2d8e072`; the parent reports its postmerge CI
  passed. Preserved prior path/custody qualification artifacts and branches.
- Confirmed Script could edit existing blocks but offered no writing action for
  an empty document. Added Write screenplay for the selected timeline context,
  kind selection, exact-text save, cancel and explicit placement refresh. Draft
  document/context/request identity is captured before typing and retained on
  refusal; selection changes cannot silently retarget text.
- Added a host-neutral creation command, thin registered Tauri adapter and cache
  action. SQLite creates main document/source segment only when absent, assigns
  command-derived block/span identities and appends after existing blocks. User
  provenance, sparse history and current projection are committed together.
  Existing blocks, protected spans, segment status and placement revision remain
  intact. Writer-local source/order revalidation rolls back partial history/text.
  Source-session admission precedes queueing, and the session gate is supervised
  through commit/event completion. Identical replay publishes no extra event.
- Executed **115 core tests**, **357 frontend tests**, and **219 actual-source
  SQLite/module tests**, including **seven new creation regressions** for exact
  save/reopen and neighboring memory; locks/metadata; existing generated text;
  replay after later edits; stale/deleted/empty requests; writer-trigger rollback;
  and retiming without authored revision changes. The standalone module harness
  uses a configuration-only shim for provider fixture tests, no AppState or ORT.
  Frontend draft, invoke, cache and SSR tests use fixtures; they are not graphical
  typing or native transport execution. No live-model calls were made.
- Frontend checks/build and strict core/server all-target/all-feature Clippy pass.
  Server compilation uses `ORT_SKIP_DOWNLOAD=1`, strictly compile-only evidence.
  Two new real AppState tests cover create/edit/save/reopen/preview/retime and
  queued source replacement; they compile but require exact-head native execution.
- Normal Chromium aborted because its installed SUID sandbox helper is incorrectly
  configured. GTK3/WebKit2GTK4.1 development packages are absent, so graphical and
  desktop build/runtime qualification remain pending. No sandbox settings were
  changed. The environment's earlier normal exact ORT download was denied by
  `cdn.pyke.io` with HTTP 403; no retry, alternate fetch or network-policy change.
- Automatic bible extraction, model quality and project switching are deferred.
  This milestone closes manual screenplay creation and feeds existing canonical
  memory/placement contracts. Parent owns independent review, PR and merge.


## Manual append propagation correction

- Independent review held `bab69018a0b8c5346a98e95bb09c9372e73361db` acceptance:
  A1 was consumed by generated B, then appending A2 changed canonical prompt
  context without changing B's dependency revisions or exposing Needs review.
  Executed a new actual-source regression on that frozen production source and
  reproduced the failure at the downstream review assertion. The earlier 219
  source tests passed but did not qualify consumed-source append propagation.
- Created isolated `fix/screenplay-append-impact` directly from that published
  source. A segment dependency now changes when manual creation adds a member:
  each new block gets an exact sparse `block.<block-id>` reference delta on its
  segment, and an existing segment's write identity advances in the same writer
  transaction. New-segment creation includes that delta in its create revision.
  Exact text, prior block revisions/provenance/locks, segment status and placement
  fields remain intact. No schema, lineage algorithm or UI change is introduced.
- Executed **223 actual-source module tests**, including four new regressions:
  consumed A1 + appended A2 exposes B review; preview retains exact A1/A2 and
  explicit acceptance updates B alone, clears review and binds A2 so its later
  edit retriggers review; unrelated C remains unaffected. An intervening A3
  refuses old acceptance with no partial history, and A2 replay preserves current
  membership identity. Injected block insertion failure rolls back the segment
  identity, text and all history. Captured A1/A2 membership evidence remains valid
  when its source retimes during generation, while B reports changed placement.
- Existing exact save/reopen, protected text, writer admission and retiming tests
  remain active; expected append revision counts now include the new segment
  revision. Strict server all-target/all-feature Clippy and normal commit hooks
  pass. Clippy uses `ORT_SKIP_DOWNLOAD=1`, compile-only. The harness executes real
  SQLite/source modules with a configuration-only provider fixture shim; no native
  AppState, graphical or live-model execution claim is made. Prior ORT CDN HTTP403
  and native/graphical dependency limitations remain; no retries or workarounds.
- Parent owns exact-head native qualification, independent review, PR and merge.


## PR8 uncertain-save retry and queued fixture schema repairs

- Read official review `5408583844`, comment `4179636372`, on exact head
  `35ad9d595a9b77b77b0678971fb2fceb382b9aac`. Added executable frontend regressions
  and reproduced both failures before repair: changed text/kind reused the old
  command ID after a simulated committed-but-lost acknowledgement, and uncertain
  cancel/restart replaced the original ID. The tests drive actual controller,
  store, command and desktop-invoke helpers; native receipts are fixture data.
- On isolated `fix/pr8-authoring-retry`, capture the submitted payload/ID once and
  retry that exact snapshot until its outcome is reconciled. While uncertain,
  text/kind/discard/restart/placement controls remain disabled, and mutable draft
  fields cannot change the retried command. Exact retry reconciles a recorded
  receipt without appending another block. The existing native error shape is
  preserved in `Error.cause`; only `bad_request` plus the exact known pre-recording
  placement message unlocks editing/current-placement recovery. Lookalike text,
  internal errors and untyped failures remain uncertain. A later exact retry
  receiving the definite placement refusal safely restores recovery.
- Parent-local handoff/log paths are absent here. Retrieved official Linux job
  `111546267404` and Windows job `111546267415` logs from run `37239871656`: both
  check out PR merge `0758062` for head `35ad9d5`, report **378 server passed / one
  failed**, and fail the queued creation fixture's diagnostic read at line 165
  with `no such table: script_documents`. The refused queued command correctly
  returns before schema creation. Initialize the empty canonical script schema
  in fixture setup before AppState/queueing; preserve every original assertion
  and all production Rust service/storage code.
- Executed **363 frontend tests**, including four new retry/command-path and two
  rendered-control fixtures. Executed **223 existing actual-source module tests
  plus one verification of the exact copied fixture setup prefix** against real
  saved/reopened SQLite (224 total). This prefix verifies canonical tables are
  empty and the selected source node remains stored; it does not invoke AppState.
  Frontend build/checks, strict server all-target/all-feature Clippy and normal
  commit hooks pass. Server Clippy uses `ORT_SKIP_DOWNLOAD=1`, compilation only.
- New-head native AppState/Linux/Windows CI remains pending hosted execution.
  No graphical or live-model claim, normal ORT download retry, policy/sandbox
  change, external review request or merge. Parent owns PR/review/qualification.


## PR8 creation draft lifetime correction

- Independent source review held published retry repair
  `23fc2379feca5467cc7d70fb0e7b4b8a0be359f9`: Script/Graph/Split navigation
  removes the Script panel, so its component-owned immutable pending submission
  could disappear despite correct local retry logic. Executed the fresh Script
  consumer regression against that exact frozen production head; returning Script
  failed to render Retry same save or the original uncertain draft. This is an
  executed state-lifetime/SSR reproduction, not a graphical observation.
- Isolated `fix/pr8-draft-lifetime` moves the existing creation controller into
  a small active-project-session owner. The Script composer consumes it and
  reacts to project-session replacement. Mode changes remain unrestricted;
  text, captured source and exact submitted payload/ID survive view removal,
  including a delayed acknowledgement failure while Script is absent. Existing
  exact retry and definite-placement-refusal behavior remain unchanged.
- Existing project activation resets this owner alongside transient editor
  state. An old retained consumer cannot submit into the new project; old
  completion only updates its retired draft and existing projection lifetime
  guards reject its publication. No application-restart persistence, backend
  write cancellation, generic draft framework or Rust change is introduced.
- Executed **366 frontend tests** across **66 files**, including three new
  actual-controller/command-path lifetime regressions: fresh Script consumers
  restore the exact uncertain submission after Graph/Split; an in-flight save
  loses acknowledgement while absent and later retries the identical command;
  real project activation replaces the owner and isolates late response/retry.
  Native receipts and invoke are fixtures. These qualify equivalent consumer
  state lifetime through real workspace SSR rendering, not DOM unmount events,
  graphical navigation, native transport, live models or full server runtime.
- Frontend typecheck (zero errors/warnings), static build, and normal commit
  hooks pass. The previously denied normal ORT download (`cdn.pyke.io`, HTTP403)
  was not retried; no policy or sandbox change. Parent retains exact-head hosted
  native/graphical qualification, independent review, PR and merge ownership.
