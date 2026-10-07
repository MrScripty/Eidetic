# Authored timeline Notes and targeted screenplay review

Application source: `a31f2a650fdf5f87cc27985b63f47310e2b34758`, tree
`55e63d0e16e21373bf6ba597c48227590d12ecec`, successor of PR21
`1bf7873579705a824ad45a1c95435e700cb09eda`. Separate feature branch:
`feat/timeline-notes-screenplay-review`. PR19/20/21 remain open drafts, unmerged
and unchanged. This report and its small custody manifest add no runtime change.

## Verified gap and result

The baseline public SQLite regression ran exactly once and failed: generation
captured selected-clip Notes, but an authored Notes edit produced no screenplay
review cause. Placement and Bible/manual-screenplay review already worked.
The source adds Notes consumption to existing target/accepted-proposal receipts
and graph dependencies, using the sparse owned Notes field revision. Range,
status, recap and lock changes do not advance that field clock. Missing/unowned
legacy history remains unknown; actual older target receipts can derive impact
without rewriting old graph/history rows.

An exact Notes edit identifies its consuming generated scene. The existing
Preview update / Reject / Accept update controls display original/current Notes
and a targeted screenplay proposal. Edit and preview retain saved material;
explicit acceptance alone replaces the selected block and refreshes its lineage.
Changed/restored Notes, manual target edits, locks and delayed results retain
stale guards. No parallel draft/proposal/revision store or model dependency exists.
Decision and scope: [timeline-notes-review plan](../plans/agent-story-workflows/timeline-notes-review.md).

## Maintained gates

Local and hosted all-target checks passed: **123 core, 542 server, 551 UI tests in
96 files**. Ten Notes-focused backend regressions and five new UI tests cover
exact provenance, unrelated work, clearing, ABA, delayed results, refusal/replay,
locks, legacy receipts and explicit review. Strict all-target/all-feature
core/server Clippy, rustfmt, UI typecheck (zero errors/warnings), lint, format,
build, decision traceability and normal commit hooks passed. Qualification
driver suite: 48 Python tests; ONNX no-download checker: five tests.
Native builds used the established hosted locked dependency route; local ONNX403
was not bypassed. Existing Bible owner/refresh repairs were not duplicated.

## Hosted native execution and capture-inspection blocker

[Run 37688763677](https://github.com/MrScripty/Eidetic/actions/runs/37688763677)
passed, first attempt, job `113023267024`, QA
`9501a1fa8d60ede96cedbedd235d0ed07a6d9de8`, tree
`dfd9873e7937df4dd9b97d4881ed1c216b405e5e`. Source admission checked the exact
nine-file qualification-only delta from unchanged application source `a31f2a6`.
The actual native application used ordinary X11/AT-SPI actions; the public seed,
read-only QA receipt and HTTP replies were visibly labelled synthetic.

The passed driver exercised exact debounced Notes writing, Notes-cause selection,
pending targeted preview, an ordinary Notes edit/restore ABA, stale acceptance
refusal, rejection, fresh preview and explicit acceptance. It asserted unchanged
saved material before acceptance, preservation of unrelated saved blocks/draft,
Bible fields and placement after acceptance, and clearing of the selected Notes
cause. It asserted exactly six accepted synthetic HTTP requests: four seed
generation/recap calls and two explicitly requested targeted previews. The fixture
already had other review causes; the added Notes cause is the qualified transition.

Exact authored Notes are `"  Mara reveals the witness — 雨.\n\n  "`; accepted
synthetic text is `"  Synthetic Notes update: Mara identifies the witness — 雨.\n\n  "`.
The unrelated draft is `"Unrelated draft: Eli pockets a brass whistle.\n\n"`.

[Artifact 11513301158](https://github.com/MrScripty/Eidetic/actions/runs/37688763677/artifacts/11513301158)
contains the driver's four declared capture states, their JPEG quality-85 display
derivatives, two sanitized logs and `capture-evidence.json`; upload recorded 11
files, 1,697,365 ZIP bytes. Hosted ZIP SHA256:
`b70165deb35dbf1bf44c2a90d4db28237c4aa6cb1224cb4381865f1812ad7751`.
Declared PNG originals:

- `timeline-notes-01-needs-review.png`
- `timeline-notes-02-pending.png`
- `timeline-notes-03-stale.png`
- `timeline-notes-04-accepted.png`

**The four passed captures are not yet downloaded, independently hash-verified or
visually inspected in this executor.** After environment reconnect, `download_file`
became unavailable; the proxy rejected authorized cloud-file and GitHub CLI reads
with HTTP403, including the approved network execution route. Parent-side artifact
materialization, member/hash checks and visual inspection remain required. This is
an artifact-access blocker, not an automatic approval rejection. Hosted execution
is a confirmed pass; this report makes no claim that the new four images were seen.

## Earlier attempts and inspection evidence

Run 37684720147 passed UI/build/server gates, saved the exact Notes and found the
canonical Notes cause, then failed to observe a label inside closed disclosures.
QA `ed1b748` opened the disclosures ordinarily. Run 37687254794 then reached the
visible Notes cause but failed to enumerate a closed chooser's options. Forward
QA `9501a1f` anchored the existing chooser to its ordinary source clip label,
selected with native keys and required the exact Notes dependency in the proposal.
Application source remained unchanged through all three runs.

Both failed archives and lossless originals are preserved locally outside Git.
The second original, SHA256
`55c44ec6c42119d8cf5d6c38869d937ea6452393a3b281dce947b213ba9a15dc`,
was inspected: exact Notes and a downstream Notes cause are visible with Bible,
timeline, screenplay and the retained draft. Its JPEG derivative was verified at
quality 85. That image precedes preview/acceptance and is labelled a failed-driver
capture; it is not substituted for the passed run's four captures.

## Scope and custody

Only selected clip owned Notes are scoped. Ancestor/sibling Notes, other timeline
fields, arc membership and project switching remain separate. Real-model quality,
Windows desktop behavior and raw AI prompt-cache refresh are not newly qualified.
Backend refusal/replay tests snapshot every SQLite user table; native refusal
assertions cover the existing recorded-table snapshot. Runtime binary/module
hashes are driver receipts, not an offline archive of those bytes.

[Custody manifest](timeline-notes-review-native-provenance/custody.json) binds exact
source/QA, test counts, hosted artifact metadata, prior failures and inspection
limits. Raw originals, JPEGs, logs and archives stay outside source Git. Parent
owns durable external/Library delivery. The new archive expires
**2026-10-10T21:36:54Z**; earlier artifact `11503180647` expires
**2026-10-10T18:16:31Z**, and its ZIP/four original captures and local preserve ref
remain intact. No history rewrite, new raw collection commit, external reviewer
request, credential change or merge was performed.
