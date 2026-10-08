# Exact committed Notes prompt custody qualification

[Hosted run 37713651568](https://github.com/MrScripty/Eidetic/actions/runs/37713651568)
**passed**, including ordinary exact Notes editing, held older real-context return
refusal, ordinary clear and exact restoration. Saved screenplay text, spans,
segments, locks, timeline placement, Bible facts and an unrelated F draft are
preserved. This is native application assertion/capture evidence; personal visual
inspection remains blocked by executor artifact-transfer HTTP403.

Application source: `c5fad5cbca6f1d3ba42bcdb66c833ac1d513dbf4`, tree
`ade8c69ff296285c3b60bc4bdd3f0dfcbde2ec6a`, separate
`fix/notes-prompt-preview-custody`, based on PR22 Notes/report head
`5a8670d95e44915426986faa76b5d1739fc0e036`. Frozen QA:
`11b60a92c6fed0040f34c3a153329b3f8c7af826`, tree
`35f3f4983680e1c255090fe6fc7921770c4aa326`, separate
`test/notes-prompt-preview-native`. The exact QA delta is ten enumerated files;
the hosted admission check confirms every application file is unchanged.
PR19/20/21/22 and main are untouched and unmerged by this work.

## Concrete gap, implementation and independent review

The public Notes command emits `TimelineChanged` and `NodeUpdated`. Their UI
handlers refresh canonical selected Notes and screenplay impacts without advancing
`scriptDocumentProjectionState.contextRevision`. `BeatEditor` supplies exact Notes
to the existing context request owner, which previously deduplicated on selected
node and script revision only. Six new regression cases fail on exact `5a8670d`
while seven maintained cases pass. The existing `ai_context_preview` public read
already loads canonical SQLite; no new endpoint, model or dependency is needed.

The request owner now includes exact nonempty committed Notes in its identity.
A change clears displayed old context and starts the existing public read.
Invalidation clears that identity. Request-ID and selected-node admission still
protect success, failure and loading finalization. Manual Refresh and unchanged
identity deduplication are retained. No saved text, draft, proposal or acceptance
owner changes.

Independent review of exact `c5fad5c`, its thirteen lifecycle cases and the
`BeatEditor` / public-read / command-event integration found no issue within this
bounded scope. Tests exercise whitespace-only differences, Unicode, same-node
Notes ABA success/failure, overlapping Refresh, failed fresh reads, clearing,
selection changes and stale finalizers. Broader metadata/context membership,
project switching and real-model quality remain separate.

## Verified gates

- Local focused lifecycle plus selected-inspector events: **34 tests / 2 files**.
- Full UI, repeated hosted: **557 tests / 96 files**; typecheck **0 errors / 0 warnings**,
  lint, format, production build and labelled QA host build pass.
- Hosted locked core **123** / server **542** tests and strict all-target/all-feature
  Clippy pass. Local ONNX403 was not bypassed; runtime/service execution uses the
  established hosted dependency route.
- Native-driver **50** and ONNX no-download-policy **5** tests pass.
- Normal commit hooks pass; logs and machine receipts stay outside source Git.

| Frozen application file | SHA-256 |
| --- | --- |
| `ui/src/lib/components/editor/contextRequestLifecycle.ts` | `bead79a6cb21b0ab519a19f142601485356ca6bf90ad209e91f23e5b46dac9e6` |
| `ui/src/lib/components/editor/contextRequestLifecycle.test.ts` | `77892744bdcf52e8ebb7701da45bf0cdf5e4fe374d5d441eb9fdc82211e3e0ab` |

## Native route and exact outputs

Ordinary native controls select B, open Raw AI Prompt, request Refresh, edit Notes,
clear and restore them. The exact public saved value is
`  Mara reveals the witness — 雨.\n\n  `. The clearly labelled QA seam holds the
unmodified result of the real public context read; it fabricates no prompt.
`untrack` prevents qualifier state from becoming a production editor dependency.
Native [AT-SPI text-range scrolling](https://github.com/GNOME/pyatspi2/blob/master/pyatspi/text.py)
reveals the actual Notes before glyph bounds are checked. Exact text is read from
the real User Prompt, excluding the QA receipt textarea. No DOM/IPC injection or
direct database write is used. Read-only canonical material/history comparisons
and actual native draft controls establish preservation and late-return refusal.

Four explicitly synthetic HTTP/SSE fixture responses create two saved consumers;
all are accepted by the fixture's exact-prompt guards. Context preview makes no
additional inference call. Real-model narrative quality is unqualified.

[Artifact 11523381856](https://github.com/MrScripty/Eidetic/actions/runs/37713651568/artifacts/11523381856):
**2154758 bytes**, ZIP SHA-256
`b4f95daaf9f11d3bcd5043c85249384079f884dcefc4d19c40cd1560bc07e6ff`,
expires **2026-10-11T01:44:58Z**. Connector ZIP reference:
`file_00000000e8308230a57a6c1fba103549`. It preserves five lossless native PNGs,
five JPEG quality85 display derivatives, capture JSON and two sanitized logs.
Native window PID24767 / X11 window2097155 belongs to the actual application;
its executable SHA-256 is `dfc065ef2c1147dff16ef6ba4b55afe6e24759796087d75251bc3f0bcbd488b5`.

| State | Original PNG | Original SHA-256 | JPEG85 SHA-256 |
| --- | --- | --- | --- |
| Original context | `notes-prompt-01-original.png` | `a87a7a36ace6dba6e63604d90a59f7523965c794b2417669a210d0c85f4aee76` | `00b858b37d639da0ecc925b26cda125f2ee7fc532563877be0a1fc66b06ed814` |
| Fresh while old return held | `notes-prompt-02-fresh-old-held.png` | `2fe7b9cd6e9ae659a4a8566a5c85c92f33612d565bef016e4d3bc6f99961442d` | `2b0ef7a6c8d4097f5a0d2548dfa69d560d9a6e9e2530d38431d256486067c241` |
| Late return refused | `notes-prompt-03-late-return-refused.png` | `63012c5f6c4301f5efb0f5613dbdb99491cd0cc2c9d3c14491b7d5747a4f2daa` | `a49ba85c473f9bdac7bf62711f3ddf79da5088bc0c2049d3f3d1ed5f15630345` |
| Ordinary clear | `notes-prompt-03b-cleared.png` | `d31ed9bd7e9ce77d6ca1c2cb2fd1d052b75a96f2db398b969353127496dcc6c7` | `0fb1b94f36f66af1cf70998d68d859ed26c4cef01be8511c54ceaf4df517777e` |
| Exact restoration | `notes-prompt-04-restored.png` | `55663e6c14ce7f743f03c820c1560538cfa87aa74e26763c86b92c15c6291f11` | `a68c13b7ec031be40633d8cbcedefcd2cd433272c50fae3ec54fb8e5c598d604` |

The receipt records actual fresh/restored Notes glyph bounds inside the real User
Prompt and editor viewport, raw-panel absence after clear, a new restored public
read, unchanged canonical history after released read, and preservation of saved
material and the unrelated F draft. The workflow result is terminal success;
no passing run was restarted.

## Preserved attempts and remaining inspection gap

| Attempt | Frozen QA | Result | Artifact / ZIP SHA-256 |
| --- | --- | --- | --- |
| [37709357998](https://github.com/MrScripty/Eidetic/actions/runs/37709357998) | `75f04f28c5c6256053eca6396550a209cb5d8fe2` / tree `2a3263b2fd316636414df6ef379052f27e07b385` | Hosted gates pass; original glyph check times out below editor edge. QA adds native text scrolling. | `11521497436`, 432439 bytes / `5577d26de6d449cf8e9dd9c4ac6342119e79a48c5aa6bcf4d70b6a4e1ed46084` |
| [37711020635](https://github.com/MrScripty/Eidetic/actions/runs/37711020635) | `795e1d5c4716c04904d20f5a053582b583db748d` / tree `dbfb8f1ce28ebd6cf93abbf9619efa44313852fd` | Three native states pass; empty-string helper selects without deleting. QA adds BackSpace for empty input. | `11521319325`, 1752027 bytes / `9fc49c262c00300cf27df08ddf1433bd0a1a1b22e250f82e57ff5aff28456789` |
| [37712308609](https://github.com/MrScripty/Eidetic/actions/runs/37712308609) | `edf5b9910375b5f1f08bb14f3491ba25d72d27f6` / tree `61653d61622b5ee4381cf29f5d0d41555c4fbf8e` | Clear succeeds; broad empty-field locator types into unsaved Bible Summary draft, then exact Notes commit check refuses. QA uses observed NOTES label at x296. | `11522328081`, 1731180 bytes / `6c3c7e96a72a7db76a4e45e8bbe4362944d41e4cfc5399b2c0269d6ce5b6cc7a` |

All attempts preserve originals and decoded logs outside Git; no product change
was made to repair those qualifier defects. Executor transfer of the artifact
archive is refused by its network proxy with HTTP403, including a supported
explicit network-access request. The connector archive reference and verified
GitHub artifact metadata are available. No personal screenshot-inspection claim
is made. Parent owns independent visual inspection, durable Library delivery and
later PR/review/merge decisions. Pumas modality publication remains separate.
