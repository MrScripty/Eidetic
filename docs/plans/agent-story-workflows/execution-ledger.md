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
