# Exact authored Bible freeform text

Separate bounded successor of accepted Notes commit
`c703ebb7a0016554b702124a794a4654968b42cd`, tree
`ab5d64837792dc9379370a927a32486a6cc92f39`. Notes/PR28 and PR27 stay unchanged.

The authoring roadmap and manual-edit Bible reconciliation require preservation
of authored material and canonical before/after fact provenance. The existing
Bible Save/preview/accept workflow and graph dependency memory already work.
This slice corrects an input loss rather than introducing another fact writer.

Before implementation, the actual client-compiled `createBibleGraphFieldDrafts`
received `  Mara reveals the witness — 雨.\n\n  ` but called its existing Save callback
with `Mara reveals the witness — 雨.`. A regression against the unchanged owner
failed with those exact actual/expected values; eleven existing client tests passed.
Its request construction called `submitted.trim()` even though
`BibleGraphPartFields` forwards exact text and the canonical field writer retains
`FieldValue::Text` verbatim. The UI retired the draft after acknowledgement of the
trimmed value, silently discarding author input. Baseline evidence is retained
outside Git in `bible-fact-text-custody-evidence/exact-fact-baseline.log`.

Keep the submitted freeform text exactly as entered. Existing empty input still
maps to `null` and clears the field. Whitespace-only authored input is a text value,
not an empty clear. Existing typed fields, canonical writer/provenance, validation,
conflict handling, pending-save custody and explicit screenplay acceptance are
unchanged. No schema, dependency, provider, recovery or new command is added.

Client qualification includes leading/trailing spaces, newline/Unicode text,
whitespace-only text and explicit empty clear. Qualification/independent review
and exact source are recorded in the separate external handoff. No hosted or
real-model quality claim is implied.
