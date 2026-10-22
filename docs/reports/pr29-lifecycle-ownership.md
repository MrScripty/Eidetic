# PR29 lifecycle observation repair and remaining decision

## Scope

This follow-up starts at `b4af7e94a0eb7896e2118fb9cff564f39f66ec03`.
It changes only the existing private Pumas runtime owner and its regressions;
the public AppState/desktop/model contracts, producer pin, model selection and
inference behavior are unchanged. It is a partial lifecycle repair, not product
acceptance. The PR discussion owns the published repair SHA and exact-head CI.

## Repaired within the existing owner

- The child and stdout-drain task are retained in one runtime owner. Explicit
  shutdown waits for child exit and joins stdout; output I/O errors and task
  panics become terminal errors that repeated shutdown cannot erase.
- Cancelling a shutdown waiter leaves the stdout completion handle in the
  owner. A subsequent explicit shutdown can observe it. A stdout-drain timeout
  also retains its handle instead of reporting clean completion.
- A bootstrap that acknowledges borrowed ownership but fails to exit within
  the existing ten-second budget has only its spawned helper terminated and
  reaped. No unload/shutdown RPC or signal is sent to the borrowed service.
  The output observer is joined before reporting the normal timeout outcome.

The borrowed-helper regression first failed against the original source with
the helper still present in `/proc`. The test cleaned up that deliberate hang.
After repair it requires helper disappearance, an empty service RPC log and
continued external-service connectivity. Controlled stdout regressions cover
I/O failure, task panic, sticky terminal errors and cancelled-waiter handoff.
These are process/I/O fixture claims, not trained-model or native UI evidence.

## Required composition-owner decision

The admitted startup task and last-Drop fallback still lack an external owner
that can observe their completion, errors and panics after their caller is
gone. This repair does not relabel either path as complete. Exceptional helper
termination/wait errors or stdout timeouts during failed startup can still
reach that detached fallback and remain part of this gate.

An owner stored only inside the cancellable `attach`/`AppState::new` future
disappears when construction is cancelled. An owner stored only in the last
`PumasRuntime` disappears during Drop. A self-retaining task, logging,
`catch_unwind`, `kill_on_drop` or another detached observer cannot establish
external terminal observation. The existing generic backend supervisor also
aborts and drops its handles on last Drop, so merely registering there is not
a complete solution.

The smallest sound next design is a Pumas lifecycle owner retained by the
application composition root **before subprocess admission**. It must register
startup before spawn, retain startup/output/cleanup outcomes after caller
cancellation, receive last-Drop cleanup requests without spawning unowned
work, close admission during application exit and be explicitly drained by
the root. Incomplete cleanup retains custody and reports failure. The generic
backend supervisor need not be redesigned.

This requires an explicit decision about the AppState construction boundary
and the desktop/smoke callers that retain and finally drain that owner. The
alternative is to narrow the cancellation/last-Drop promise, which changes the
existing contract. Neither choice is silently implemented here. No global
owner, alternate runtime, cleanup thread or public API change is introduced.

## Synchronous shutdown execution contract

Source inspection uses the locked Tauri 2.11.2 implementation:
`async_runtime.rs` creates a Tokio runtime by default and `block_on` delegates
to it; `Builder::on_window_event` registers a synchronous window-event handler.
Eidetic calls `block_on(AppState::shutdown_tasks_async())` from that handler on
Destroyed, after stopping its event bridges and renderer owners. This keeps the
explicit drain receipt before the callback returns, and blocks further event
dispatch while waiting. The owned Pumas drain uses Tokio HTTP/process I/O and
does not intentionally dispatch UI work back to that callback.

This source inspection does not demonstrate deadlock, establish every possible
runtime override/callback context, or constitute native execution acceptance.
Moving final drain to a root retained outside the event loop (for example an
explicit post-event-loop exit phase) belongs to the composition decision above.
No event-loop/exit API is changed by this partial repair.

## Retained acceptance gates

- Decide and implement the retained composition owner, then directly test
  cancelled startup success/error/panic and last-Drop fallback completion and
  failure, including incomplete cleanup custody.
- Qualify the accepted desktop exit/callback execution contract in a supported
  native environment. There is no demonstrated deadlock claim to waive.
- Run actual trained chat/embedding and native lifecycle acceptance from
  [the operator procedure](../pumas-local-inference.md#local-acceptance-with-real-models).
  Model packages and native development libraries remain absent locally. No
  models, credentials or denied-access retries are part of this follow-up.

Keep PR29 draft until these gates are independently dispositioned. Source
tests and hosted CI prove their named source/fixture claims only.

## Local verification

- Six targeted Pumas runtime tests pass, including the original cancellation
  success fixture and the four new regressions.
- Locked core/server all-target tests pass: 129 core and 642 server tests,
  with all example targets; strict all-target/all-feature Clippy passes.
- Rust formatting, staged whitespace and decision traceability pass.
- The canonical `./launcher.sh --test` stops at missing `glib-2.0` development
  metadata during native compilation. The original output is retained; no
  assertion, discovery filter or threshold is weakened. Hosted exact-head
  native-workspace CI remains a separate gate recorded on the PR.
