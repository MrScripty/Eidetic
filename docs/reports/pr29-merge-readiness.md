# PR29 integration and remaining acceptance

## Scope and source identity

PR29's existing `codex/pumas-local-inference` branch starts at
`b09f9c288b8feeb5da3f1643b085b0338c106dd2`. This integration merges main
`59793c1950916a067f6429db59bbc639fe3c4acf`, preserving both histories and PR30's
native UI changes. It does not release or merge PR29. The parent owns that
decision after reviewing the source and the separate runtime acceptance gate.

Pumas remains pinned to `ad31e391dcd94c204b3fcd748cc98d27b3281e65`
(`0.8.0-rc.1`). Main's older `a94fd920` setup pin is `0.7.0` and lacks the
discovery and typed-operation APIs used here. The setup script, governance
document and lockfile agree. No producer source is changed. See
[dependency governance](../dependency-governance.md).

## Confirmed defect and repair

The pinned producer's ONNX, llama.cpp and Ollama serving adapters omit the
optional `loaded_at` timestamp. Eidetic previously required it, so a successfully
loaded real embedding model could never be used. The old fixture invented a
timestamp and concealed this incompatibility. A producer-shaped regression
first failed with `Pumas embedding load is missing revision timestamp`.

The consumer now binds embedding identity to the producer's monotonic serving
cursor, instance/service generations, selected provider/model/profile and actual
metadata. The optional timestamp is retained if supplied. Existing observations
before and after inference remain required. Same-model unload/reload with
unchanged metadata invalidates old vectors, and a serving change during inference
fails without publishing any document chunks. An unrelated serving mutation also
requires reindexing; this conservative behavior is explicit in the operator guide.
No fabricated model revision or fixture output enters production.

## Verification on 2026-10-14

The executor has Rust 1.92.0 and Node 24.12.0. Tests use the selected exact Pumas
checkout. Application data/config/cache were isolated under a temporary directory
because the executor's host home is read-only.

| Check | Result and boundary |
| --- | --- |
| `cargo test --locked -p eidetic-core -p eidetic-server --all-targets` | 123 core and 615 server tests pass; all example test targets pass. |
| `cargo test --locked -p eidetic-server --lib pumas` | 16 controlled consumer/process fixture tests pass. |
| `cargo clippy --locked -p eidetic-core -p eidetic-server --all-targets --all-features -- -D warnings` | Pass. |
| Frontend lint, format, typecheck and `npm run test` | Pass; 608 tests in 107 files. |
| Locked all-feature Cargo metadata and ONNX no-download guard | Pass; five guard tests pass. |
| Authoring fixture regressions | Seven tests pass. |
| `cargo fmt --all -- --check` and `git diff --check` | Pass. |
| Independent GPT-6.1 Sol High source review | No remaining confirmed P1/P2 in the candidate repair. |
| `./launcher.sh --test` | Environment-blocked at missing `glib-2.0`; GTK/WebKit development libraries are absent. Full native workspace checks must run in hosted Linux/Windows CI on the pushed head. |
| `scripts/check-dependencies.sh` | Fails npm high/critical advisory gate: 42 reported vulnerabilities (1 low, 24 moderate, 14 high, 3 critical). Rust audit/deny/machete tools are absent and reported as local skips. The frontend manifests/lockfile are unchanged from main; several high/critical findings have no available fix. Separate dependency maintenance or an explicit parent disposition is required. |

The PR record owns the exact pushed head and its hosted CI run. Passing source
checks do not resolve the dependency audit or trained-model acceptance below.

## Trained-model acceptance remains blocked

There are no local chat or embedding weights. Ordinary unauthenticated HEAD
requests for public trained chat/embedding artifacts at Hugging Face fail at the
executor proxy with `CONNECT tunnel failed: 403 Forbidden`. Access was not
retried through another host or bypass. No trained inference was executed.
Controlled vectors and screenplay text are synthetic fixture data only.
PR30's native UI captures do not qualify this new model/reference lifecycle.

Minimal operator step: on a machine with accessible local trained chat and
embedding packages, build this exact Pumas pin with default inference plugins,
load both selected models/profiles, and execute
[the local real-model acceptance procedure](../pumas-local-inference.md#local-acceptance-with-real-models).
Record actual generation, reference indexing and exact prompt contribution,
reopen/reindex, same-model reload refusal, and owned/borrowed cleanup. Keep PR29
draft until the parent reviews and dispositions the gates; no denial bypass,
credentials or release action is part of this repair.

## Deferred robustness observation

Independent review also noted that a nonconforming bootstrap that acknowledges
borrowed ownership but then hangs can outlive the ten-second bootstrap wait; that
failure path drops its child observer. The pinned producer normally exits
immediately after borrowed acknowledgement. This lower-priority subprocess
robustness issue is deferred to a separately scoped lifecycle repair. It does not
grant authority to unload or shut down the borrowed Pumas service.
