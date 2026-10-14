# Native workflow acceptance — 2026-10-11

Source: `18d6c5e0c51c2c6b75f161939ce4b32f0bfb6338`, based on main
`7e963d58036c4a576a6750d7fc0b7916262f03e4`. This branch is separate from
[Pumas integration PR #29](https://github.com/MrScripty/Eidetic/pull/29).
The screenshots are actual native Tauri/WebKit and Bevy windows. The frontend
is the production Vite build, served locally; no transport or model adapter is
stubbed. All story content is original authored fiction, not generated output.
No model weights, personal data, credentials or raw runtime logs are included.

## Results

- Opened a saved SQLite project through the native project list. Modification
  dates now parse the backend's Unix seconds, use the database's modification
  time, and have an unknown-date fallback.
- Selected the first scene on the main timeline and read its Notes, screenplay
  and inherited structure. Renamed that clip, verified SQLite, then restored its
  title through the UI after reloading the title draft.
- Edited the first screenplay heading to `EXT. NORTHBREAK HARBOR - BLUE HOUR`;
  saved it, reopened the project with the rebuilt application and verified it.
  A subsequent unsaved screenplay edit was cancelled without replacing that text.
- Edited Mara Vale's tagline to `She can repair anything except a goodbye.
  Until tonight.` Saved it through the field's Save control, verified SQLite and
  read the same value after project reopen. The initial automation selected
  Discard draft instead of Save; only the corrected, persisted edit qualifies.
- Opened, focused, hid and reopened the same native graph owner. Its OS window
  became unmapped on hide and its readiness status returned after reopening.
  Disabling graph-camera tonemapping removes the missing-LUT startup error.
- Opened the separate native timeline as the first Bevy runner in a fresh
  application process. Its geometry rendered; closing destroyed its OS window.
- Verified graph → timeline startup rejection and timeline runner recreation
  rejection through visible UI error status, with an actionable restart message
  and no `RecreationAttempt` panic in the guarded run. The main timeline remains
  usable after rejection.

## Captures and proposed captions

| Capture | Honest caption |
| --- | --- |
| [screenplay-timeline.png](screenplay-timeline.png) | Eidetic's screenplay editor and six-scene timeline, using an original authored demo. Saved text was verified after reopening the project. |
| [story-bible-timeline.png](story-bible-timeline.png) | Character details beside the selected scene and episode timeline. The authored story-bible edit was saved and verified after reopen. |
| [project-open.png](project-open.png) | Reopening a saved local project with a readable modification date. |
| [graph-controls.png](graph-controls.png) | Graph controls and the authored project's relationships; the native diagram is displayed in a separate window. |
| [native-graph.png](native-graph.png) | The current native Bevy graph window rendering authored story-bible nodes. This is minimal renderer evidence, with a linear layout and an empty lower panel. |
| [native-timeline.png](native-timeline.png) | The separate native timeline rendering geometry from the saved project. This minimal renderer omits the labels available in the main editor. |
| [second-renderer-error.png](second-renderer-error.png) | A visible, actionable error when a second native renderer would recreate winit's event loop. |

Use the first two views for application presentation. The minimal native views
are supporting evidence of current implementation limits, not redesigned mockups.
Pixels were captured directly from native windows using ImageMagick `import`;
they have not been edited. See [provenance.json](provenance.json) for dimensions,
source identities, runtime details and file hashes.

## Local reproduction

On Linux with the normal documented GTK/WebKit and graphics dependencies:

```bash
EIDETIC_FETCH_PUMAS=1 scripts/prepare-pumas-dependency.sh
cd ui
npm ci
npm run build
cd ..
export XDG_DATA_HOME="$(mktemp -d /tmp/eidetic-authored-demo.XXXXXX)"
cargo run --locked -p eidetic-server --example portfolio_authored_demo
# Terminal 1, production frontend:
npm --prefix ui run preview -- --host 127.0.0.1 --port 5173 --strictPort
# Terminal 2, with the same XDG_DATA_HOME:
cargo run --locked -p eidetic-desktop --bin eidetic-desktop
```

The fixture refuses to overwrite an existing `Signal at Low Tide` database.
Open Project → `Signal_at_Low_Tide`; select the first scene, edit/save a
screenplay block or a Bible → Mara Vale field, then reopen and compare. Try a
timeline title draft and cancellation separately. Use the main timeline normally.
Open the floating graph first for its renderer checks; restart the app before
opening the separate floating timeline first.

Install dependencies inside each worktree. The earlier capture's node_modules
symlink caused Vite to deny the Svelte hydration entry outside its workspace.
This acceptance used an ordinary worktree-local `npm ci`, not an access-rule
bypass. The cloud environment used writable staged Debian dependencies and a
dependency-only WebKit helper/bundle path relocation; no application-source
transport shim or fabricated UI state was used.

## Native renderer limit

Graph and timeline currently own separate Bevy/winit runners. Winit permits one
event loop per process, including after a runner exits. The new guard reserves
that ownership atomically before native startup and returns existing runner-error
status for a second reservation. Showing a still-running hidden graph reuses its
owner. A terminated runner requires an application restart. Unified multi-window
rendering needs a separate architecture change; this branch does not implement it.

## Validation

- Frontend: 603 tests passed; Svelte check found zero errors/warnings; lint,
  formatting and production build passed.
- Desktop: 95 tests passed, including concurrent native event-loop reservation.
- Native graph crate: 61 tests passed, one existing display test ignored. The
  actual native graph and timeline checks above supplement controlled fixtures.
- Server persistence: six tests passed, including database-vs-directory mtime.
- Authored fixture: seeded and saved in a fresh profile; repeat refused before
  overwriting the database.
- Workspace clippy with all targets and warnings denied passed. Decision
  traceability passed for the committed source change.
- Exact-head CI status is reported with the branch/PR outcome separately.

## Deferred Pumas inference acceptance

There are no chat or embedding weights in this executor. The user's unpublished
Pumas changes are not available here. No generation, embedding, cloud inference
or paid inference was executed. Reference embedding/reindex/retrieval, local
model load/cancel/unload, and owned-vs-borrowed inference cleanup remain deferred
to the coordinated producer and PR #29 integration; this branch retains main's
older Pumas pin and does not modify the producer repository.

After the parent publishes the producer fixes and coherently updates PR #29's
pin/lockfile, use that branch's
[operator instructions](https://github.com/MrScripty/Eidetic/blob/codex/pumas-local-inference/docs/pumas-local-inference.md).
From its Eidetic checkout:

```bash
EIDETIC_FETCH_PUMAS=1 scripts/prepare-pumas-dependency.sh
cargo test --locked -p eidetic-server --lib pumas
cargo build --manifest-path ../../ai-systems/Pumas-Library/rust/Cargo.toml -p pumas-rpc
```

That test command uses explicitly controlled producer fixtures; it does not
qualify real-model inference. With already installed compatible local models and
native runtime libraries, run the built producer:

```bash
../../ai-systems/Pumas-Library/rust/target/debug/pumas-rpc \
  --launcher-root "$PUMAS_ROOT" --port 8080
```

Run Eidetic in another terminal:

```bash
EIDETIC_PUMAS_ROOT="$PUMAS_ROOT" \
  cargo run --locked -p eidetic-desktop --bin eidetic-desktop
```

Use the AI panel's separate chat and embedding provider/model/profile selections:
load each, cancel an admitted load, retry, generate with an authored reference
containing a distinctive fact, and inspect the exact prompt for retrieval
contribution. Save/reopen and reindex; change the embedding model/revision and
verify stale-index rejection. Unload embeddings and then chat, checking visible
failures. Try unsupported profiles and an inference-disabled producer. Closing
Eidetic must preserve a borrowed service and its loaded models.

For the owning-bootstrap check, stop the separately launched test producer first
and additionally set `EIDETIC_PUMAS_RPC_BIN` to that exact built binary before
starting Eidetic. Verify its owned service/provider cleanup on close using the
producer's local lifecycle evidence. Keep authentication and runtime logs local.
Do not download gated models or provide credentials in chat to perform these checks.
