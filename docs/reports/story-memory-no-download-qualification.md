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

Fresh hosted run [37379727832](https://github.com/MrScripty/Eidetic/actions/runs/37379727832)
**passes** at qualification `a0b46fedfdef03901ecd031f43c4bf7636627ce1`, tree
`2a689f3b23506254655c6164a0c91ff9177f4657`; job `111998278006`.
Its source guard and all-features no-download gate passed before compilation.
Actual desktop and fixture builds passed, followed by **119 core / 425 server**
tests and the actual native walkthrough. No historical job was rerun.

The GUI saved exactly `Mara's umbrella is blue.` over the red fact. The capture
receipt proves that the saved manual source and generated target remained
unchanged, an exact manual draft survived the Bible refresh, and pending preview
consumed the new value plus exact saved screenplay. Only explicit Accept update
replaced the target and refreshed its consumed-fact dependency. The Bible value,
timeline and screenplay review state are visible together in the unmodified
native window screenshots. Some screenplay columns overflow the horizontal
viewport; exact saved/proposed values are additionally checked by the read-only
SQL and native accessibility observations. No screenshot was retouched.

| Screenshot | SHA-256 |
| --- | --- |
| `eidetic-bible-fact-review.png` | `a9c3312d93dce5ce478733698c6e32228e268d037d7659f95d30834039947c9b` |
| `eidetic-bible-fact-preview.png` | `5f22e64859cd1fd821f6df6a88074a6218298fd674446bbf07dbd675f45dfec4` |
| `eidetic-bible-fact-accepted.png` | `ad9ae78ab45c82de2ccd8c8b5079ae8a0606496a9ffd83a58ff6aa26b7203dac` |

Artifact `11372654692`, `eidetic-story-memory-native-8d2719d`, was downloaded
and verified against GitHub's ZIP digest
`756f27afe7b96d529c3947a0cc74e1e3224ff5e61b1795f8062a1cd0ff64de04`.
The three PNGs, sanitized app log and capture receipt are preserved under
`/workspace/scratch/no-download-qualification/native-37379727832/`.
Capture receipt hash: `35d959a37e3316c4faefd1aaf2b995db587851ca982811e0549c8fdb24408bb9`.
Actual native binary hash: `59f7b56b079f37ea709499db6ce6188df9c21973c2ff1e36699c6cb2cc580ab9`.
Full job log hash: `a047333ad0d88daed84176fdbde37af7c9c7704291f9f0467ebe8e7265e94f47`.

The native walkthrough qualifies consumed-field propagation on this exact
application source. The 201-node membership case executes in the full server
suite; its GUI stress presentation and the context-stack inspector GUI remain
unqualified. SDK inference, live Pumas runtime operation and real-model quality
also remain unqualified. Full local workspace/native execution requires absent
GTK/WebKit/display packages; hosted execution supplied the scoped native evidence.
No credentials, permissions or security settings were changed. The local
full-workspace pre-push hook was excluded only for those absent native packages;
ordinary pre-commit gates and the explicit service checks passed. Parent retains
PR/review/merge/Library ownership.
