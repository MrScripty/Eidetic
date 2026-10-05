# Exact-source story-memory service qualification

The full server suite now executes on frozen application source
`8d2719d2f80f3529864a138720aeb02463fc390f`, tree
`bf8fc445602b0a8b52cbd5434c668f36cb4a004c`, with a separately audited dependency
candidate. Application files under crates, ui and src-tauri remain unchanged.
Both original review branches remain frozen, including membership repair
`8ea7153b2c0f236c85ec47c0422e4f8e58a078c3`, tree
`be8155a48c6b7718b80fd080f8b3fdca10fdbac3`.

## Dependency admission

The maintained CI tests the workspace/all targets after preparing Pumas. Its
previous pin `8444b50df28c3e2bd8db58fb3645fa4dd8664b27` resolves ort/ort-sys
2.0.0-rc.12 with `download-binaries` and `copy-dylibs`. It cannot qualify this
source under the owner's no-native-download requirement and was not built.

Pumas successor `a94fd92021f27fdeedb6e2de6e01c41c250ef576`, tree
`4a6977b88ed532089807183e218a959ac02b724d`, was fetched and checked out in a
separate worktree; the original checkout stays at its old pin. This is Pumas
0.7.0 and a substantial dependency change, not a one-line loader patch. Before
repinning, the actual used ModelLibrary, ProviderRegistry, RuntimeProfileId,
RuntimeProviderId, RuntimeProfileService and adapter APIs were inspected. The
actual server compiled and all its tests executed without API adaptations.

The candidate lock retains ort/ort-sys rc.12, rusqlite 0.32.1 and tokenizers
0.23.1. It removes native downloader dependencies and adds Pumas's capability
filesystem and typed loader dependencies. Dependency milestone `1e4afc03427054d2ec40798ba08581f1cee08ee0`
(tree `3de785bb26280d49103b0e602d06a168f9bb7a8d`) is a separate integration
for review; old-pin execution is not claimed.

Both normal and all-features resolved graphs enable `ort/load-dynamic` and
`ort-sys/disable-linking`, with no download/copy-dylib feature. The CI admission
script checks Cargo metadata before build scripts execute and rejects additive
download features even if disabled linking is also present. Pumas owns typed
runtime loading of a separately provisioned SDK. No ONNX SDK was provisioned,
no ONNX download was attempted, and no ORT_SKIP_DOWNLOAD, DOCS_RS,
ORT_LIB_LOCATION or ORT_DYLIB_PATH was used in successful execution.

## Executed scope

Rust 1.92.0; existing maintained test commands, locked/offline after resolving
crate metadata. The launcher-supported writable XDG state isolation was used:

```sh
cargo test --offline --locked -p eidetic-server --lib
cargo test --offline --locked -p eidetic-core
cargo clippy --offline --locked -p eidetic-server --all-targets -- -D warnings
npm --prefix ui test -- --run
python3 -m unittest discover -s scripts -p 'test_qualify_*.py' -v
python3 -m unittest discover -s scripts -p 'test_check_onnx_no_download.py' -v
```

Results: **425 full server / 119 core / 411 frontend / 12 HTTP fixture / 4
download-admission tests pass**. Strict server all-target Clippy passes normally.
This includes the actual AppState/public-service manual creation/edit/save/reopen,
fresh screenplay context projection, bounded context membership repair, current
agent tool reads and stale proposal acceptance guards. It closes the previous
module-harness-only execution gap for these paths.

The first full run is preserved: 408 passed and 17 failed at fixture directory
creation because the host default data directory is read-only in this environment.
No source or permission changes were needed; rerunning with the maintained
launcher's writable XDG isolation made all 425 pass. This was an actual execution
failure, distinct from historical GitHub zero-step allocation failures.

Raw source-bound logs and JSON receipt are retained at
`/workspace/scratch/no-download-qualification/`. The receipt records the exact
source, dependency tree, candidate lock hash and SHA-256 of each relevant log.
Receipt SHA-256: `331f62f3203645077393a91fab78de3a4db7cba6b75e82ec82d1da48b6074e7e`.

## Fresh native qualification

The historical canonical-scene workflow is guarded to old source 9e8bd1c9 and
cannot qualify new memory changes. The ordinary CI does not run on feature-branch
pushes. Neither historical failed job was rerun.

Separate `test/story-memory-native` qualification binds the actual application
files to 8d2719d2 and permits only the reviewed dependency/qualification/report
changes. It builds with the audited no-download pin, runs actual core/server
regressions, then reuses the maintained real Tauri Bible driver: exact manual
red-to-blue fact edit, draft and saved screenplay preservation, targeted pending
preview, and explicit acceptance with refreshed consumed-fact provenance.
Its provider is explicitly a synthetic localhost HTTP/SSE fixture. This existing
GUI walkthrough qualifies consumed-field propagation, not the 201-node membership
stress case or context-stack inspector visibility; those remain separately scoped.

Native execution, new screenshots, SDK inference and real-model quality are
pending until the new run returns evidence. Full workspace/native local execution
requires GTK/WebKit/display packages absent here; no permissions, credentials or
security settings were changed. Parent retains PR/review/merge/Library ownership.
