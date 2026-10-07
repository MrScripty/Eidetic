# Explicit recalled facts in screenplay previews — local evidence

Application source: `e140290ed4414585c0c507874fab5b0e04c9dc22`, tree `c1c682d2487c5f9901f55c4b89339b9eaf2188d2`, branch
`feat/recalled-facts-screenplay-preview`, directly based on qualified
`4c468d6386bcad44117fe7e4439202cdde52a0f7` (tree `6f546f9d75bffe6baef70912f7635996f6acb244`).
This milestone is local only. Source and evidence are separate; no push, PR19
change, reviewer request or merge was performed. Parent owns publication and
Library delivery.

## Result and limits

An existing Needs review block now offers explicit selection of up to eight
resolved baseline facts from the currently displayed unspecified-time Bible
recall. The selection supplies identities and expected revisions rather than
client values. Shared recall runs again inside the existing proposal capture
transaction. Canonical chosen fields plus necessary names and connecting untimed
paths enter that new preview and its existing consumed-input binding. No new
persistent state, context link, automatic expansion or generation workflow.

Snapshot-backed, unresolved, omitted and unknown-name selections are visibly
unavailable. Nonempty timed selection is refused. Empty selection has existing
behavior. A packet, target or editor-session change retires transient selection.
Existing proposal IDs remain immutable and selection never amends a pending
proposal. Existing explicit acceptance and atomic custody checks remain owners
of saved-text replacement and generation dependencies.

The existing scene context is retained. Only selected supplemental values are
added outside it. Entity membership custody records field identities separately
from consumed values. Existing global context epochs can conservatively stale a
pending preview after unrelated writes; this feature does not relax that guard.

## Local verification

| Check | Result |
| --- | --- |
| Core aggregate | 123 passed |
| Server aggregate | 520 passed |
| New server selection regressions | 9 passed |
| UI aggregate | 522 passed in 89 files |
| Independent affected UI verification | 16 passed |
| Strict core/server all-target Clippy | Passed, `-D warnings` |
| UI typecheck | 0 errors, 0 warnings |
| UI lint, format and production build | Passed |
| Rust format, decision traceability, source commit hooks | Passed |
| Native admission | Blocked: GTK3 and WebKit4.1 unavailable |

Toolchains and existing shared dependency setup were reused. The first server
aggregate had 503 passes and 17 `ReadOnlyFilesystem` fixture failures because
`default_project_dir()` uses the read-only default user app-data directory.
The final complete aggregate passed with `XDG_DATA_HOME` and `XDG_CACHE_HOME`
under writable `/workspace/scratch`; HOME and product/fixture logic were unchanged.
Earlier schema/fixture and formatting attempts are retained separately in the
manifest and are not presented as final gates.

## Six acceptance groups

1. **Selected prompt and receipts:** a linked peer outside the 200-node default
   context supplies exact `SELECTED dry roof — 雨` and its canonical revision to
   the actual preview provider callback. Its unselected sibling
   `UNSELECTED brass key in cellar` is absent. Saved manual B stays exact.
2. **Empty intent:** capture/context bindings match existing no-selection behavior
   after normalizing the optional request field.
3. **Source custody:** over-limit, duplicate, missing/forged, timed, unresolved or
   omitted selectors refuse. Field/name/path changes, ABA, clears, deletion,
   snapshot reclassification and budget omission refuse fresh capture or delayed
   provider recording. Stale acceptance preserves whole pending proposals and
   saved scene text. Core deserialization refuses a client-supplied fact value.
4. **Preservation and immutability:** rejection, target lock, manual target ABA and
   canonical placement ABA cannot overwrite saved text, placement or pending
   proposals. The existing UI command/store test preserves exact target and
   unrelated unsaved drafts through successful/failed synthetic preview calls.
5. **Acceptance and later impact:** only explicit acceptance changes the target;
   other scene text and placement remain exact. Selected fact/name/path inputs
   become existing dependencies and later edits derive typed Needs review.
   An unselected sibling-value edit does not become consumed-fact impact. Fresh
   explicit evidence supports a subsequent reviewed update.
6. **UI and native boundary:** SSR verifies visible selection limits and pending
   canonical receipts; an actual Svelte client-rune proxy test verifies plain
   cloneable request custody and packet/session retirement. These tests do not
   mount or qualify the new selector in a native application window. New native
   screenshots are unavailable.

## Independent review

`/root/review_recall_lifecycle_successor` reviewed the complete local diff and
reported no remaining blocking defect. Its custom-kind alias finding was fixed
by detaching both query-filter and path kind objects. Actual Svelte proxy payload
cloning and mutation isolation have regression coverage. Unknown anchor-name
custody disables related checkboxes, and refused selection restores the actual
checkbox state. The reviewer independently passed all 16 affected UI tests.

## Native and model evidence boundary

Local admission failed at `pkg-config --modversion gtk+-3.0 webkit2gtk-4.1`.
Hosted native execution would require publishing source/qualifier, which this
local-only instruction does not authorize. No ONNX403 bypass or synthetic native
screenshot was used. The earlier qualified `4c468d6` screenshots remain evidence
for that base only and are not relabelled for this feature.

Provider streams and UI command mocks in these tests are explicitly synthetic.
The real prompt construction path executes, but no real model or narrative
quality is claimed. A later authorized native qualifier should show Bible,
timeline and screenplay together, an exact ordinary fact edit, checked baseline
fact, pending targeted proposal, draft preservation and explicit acceptance.

## Sealed local custody

`recalled-facts-preview-local-manifest.json` contains SHA256/byte receipts for
all 23 changed source files, complete final gate logs, retained earlier attempt
logs, independent-review findings and the native blocker. Manifest SHA256:
`dd82acd6bce07bb982beea664fad0b9df2251c794f33df89b3bda16091c7eeac`.

Both local worktrees are intended to finish clean after temporary dependency
symlinks are removed. The evidence commit directly succeeds the frozen source
and changes only this report and manifest.
