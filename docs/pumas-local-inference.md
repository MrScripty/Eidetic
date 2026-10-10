# Local inference through Pumas

Eidetic uses Pumas main `ad31e391dcd94c204b3fcd748cc98d27b3281e65`
(0.8.0-rc.1). Chat and reference embeddings are independent selections. Production
requests use real Pumas providers through its serving lifecycle and typed HTTP
model-operation contract; no fake embeddings, cloud substitution or model
download is used as a fallback.

## Prepare

1. Run `scripts/prepare-pumas-dependency.sh`. On a new machine use
   `EIDETIC_FETCH_PUMAS=1`; an existing mismatched checkout is refused, never reset.
   For active producer development set `EIDETIC_PUMAS_LIBRARY_REF` to the exact
   tested checkout, update/review the lockfile and repeat the acceptance below.
2. Build the same Pumas checkout's `pumas-rpc` with default features:
   `cargo build --manifest-path ../../ai-systems/Pumas-Library/rust/Cargo.toml -p pumas-rpc`.
   `inference-plugins` is required. Supply the native ONNX Runtime library as
   documented by Pumas (including `ORT_DYLIB_PATH` where required). Eidetic never
   enables ONNX Runtime's download-binaries feature.
3. Use Pumas to install/configure the runtime profiles and the local models you
   already have. Chat requires a chat-capable provider/model, typically llama.cpp
   or Ollama. Embeddings require a text-embedding-capable model/provider, including
   a complete real ONNX package/tokenizer/config supported by Pumas. The provider
   rejects incompatible model/profile combinations. No gated model or credentials
   are needed for the consumer tests.

## Connect and select

Run Pumas separately, for example:

```bash
/path/to/pumas-rpc --launcher-root /absolute/pumas/root --port 8080
```

Enter its numeric loopback root URL (`http://127.0.0.1:8080`, without `/v1`) in
Eidetic's AI panel. Select **Pumas**, list installed models/profiles, choose the
chat model and its profile, then **Load**. Choose **Pumas local inference** in
Reference embeddings, enter its independent endpoint, list and select the
embedding model and appropriate profile, and **Load**. Save configuration.
Capability checks require the exact loaded model/profile; merely listing a
model or reaching the HTTP service is insufficient. Load errors, unsupported
capabilities and inference-disabled builds are displayed.

Alternatively set `EIDETIC_PUMAS_ROOT=/absolute/pumas/root` before launching
Eidetic to borrow an authenticated advertised local service. On Linux, also set
`EIDETIC_PUMAS_RPC_BIN=/absolute/path/to/pumas-rpc` to allow the selected-root
`--attach-or-start-local-http` bootstrap. This starts only the Pumas control plane;
model/runtime selection stays explicit. The bootstrap's ownership acknowledgement
and exact instance/service generations govern cleanup. Cancelled startup finishes
its admitted construction and drains an owned service. On application shutdown an
owned service receives a generation-fenced shutdown and its child exit is observed.
A borrowed Pumas service receives no implicit unload/shutdown. Explicit bootstrap
failures appear in AI status and keep Pumas selected; they do not revert to a
direct llama.cpp endpoint. **Unload** is an
explicit operator action on the selected model/profile, including borrowed models.
Provider unload errors remain failures even if Pumas reports RPC `success: true`;
an error-free `unloaded: false` is displayed as already not served.
For non-Linux systems start Pumas separately and borrow its endpoint.

Existing direct llama.cpp and OpenRouter chat remain supported. An independently
configured OpenAI-compatible embedding provider uses its own `/v1` base URL,
model, operator revision and bearer API key. Its revision is operator supplied,
not a producer weight attestation. Reference embeddings default to **Disabled**;
this cannot accidentally send reference text to a chat provider.

Configuration remains application-session state. Reopening the AI panel restores
that state's selections; after a full restart select/save providers again. Saved
reference text is SQLite project content and is never discarded by index failure.

## Local acceptance with real models

1. Load both selected models. Confirm chat status is connected; choose a notes
   node and generate to exercise the actual Pumas chat provider. Also try a
   structured child-plan call; typed v1 has no JSON-mode option, so invalid JSON
   becomes a visible provider error rather than a successful structured result.
2. Upload a short reference containing a unique detail (for example a location
   name absent from the project). Wait for **ready**, nonzero chunk count and
   the model/revision/dimension display. If it fails, inspect the displayed error
   and Pumas's runtime diagnostics; uploading is not proof of embedding success.
3. Put related notes on a generation target and generate. Confirm the notification
   names the reference source. Open **Last Generation Prompt** in the Notes panel and
   confirm the retrieved detail appears in the exact prompt sent to Pumas. Inspect
   the actual generated text independently; prompt inclusion demonstrates retrieval
   contribution even if the model chooses not to repeat that detail.
4. Save and reopen the SQLite project. References remain listed and are automatically
   reindexed with the current embedding selection. **Reindex references** also
   explicitly retries saved text after startup/configuration or a failed load.
5. Change the embedding model/profile and save. The previous index is invalidated
   before new publication. Reload the same Pumas model at a new revision: retrieval
   refuses the old index and shows **stale**; reindex it before expecting retrieval.
   A changed vector dimension also excludes old candidates. Delete a reference
   while indexing and reopen during indexing; obsolete results cannot publish.
6. Unload embeddings and generate: reference retrieval must report unavailable;
   ordinary chat generation may continue, and it must not claim references were
   consumed. Unload chat and generate: generation must fail visibly. Try an
   unsupported model/profile and an inference-disabled Pumas binary.
7. Close Eidetic while using a borrowed Pumas service: it and its loaded models
   remain available. With Linux owning bootstrap, close Eidetic and confirm the
   exact owned Pumas service and its managed providers/native sessions drain.
   A borrowed/external provider's process ownership remains governed by Pumas.

## Index identity and limits

The disposable index binds exact document content/name/type/id and source ticket,
project epoch, embedding provider/endpoint/model/profile, Pumas instance/service
generations, selected load timestamp and actual library metadata (including
upstream revision if available), plus matching vector dimensions. All chunks of a
document publish together after successful embedding. Deleted sources, project
replacement and embedding-selection changes invalidate outstanding tickets.
Generation rechecks scope/config and current source at attachment.
The captured scope is checked before changing index compatibility/status, so a
late query from an earlier index cannot mark a rebuilt reference stale.

Pumas typed v1 returns vectors but no immutable package-weight revision in each
operation response. Eidetic observes provenance/load identity before and after
embedding. This prevents mixing observed reloads/revisions; it cannot prove that
weights were not mutated out of band without metadata/reload, or detect an
unobserved producer ABA within a single inference call. Do not edit loaded model
files in place; reload and reindex. A stronger producer-issued immutable revision
would require a coordinated Pumas contract change; none is introduced here.

## Automated evidence

`cargo test -p eidetic-server --lib pumas` exercises the actual consumer HTTP/RPC
and SSE paths with explicitly controlled producer fixtures. It checks lifecycle,
errors, revision changes, upload/reopen/reindex, retrieved prompt inclusion,
ownership cleanup and cancellation. Existing vector/index/session and nonlocal
provider regressions remain active. Fixture vectors and screenplay text are
synthetic **test data**, never production fallbacks or real-model evidence.
Real ONNX/llama.cpp model acceptance was not executed in the cloud environment,
which contains no chat or embedding weights/native model runtime fixture.
