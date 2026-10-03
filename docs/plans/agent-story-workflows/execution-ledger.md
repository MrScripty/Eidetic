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
