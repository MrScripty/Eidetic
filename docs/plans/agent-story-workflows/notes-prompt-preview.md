# Exact committed Notes in the selected agent prompt preview

Base: reviewed PR22 head `5a8670d95e44915426986faa76b5d1739fc0e036`.
Separate branch `fix/notes-prompt-preview-custody`; PR19/20/21/22 stay unmerged.
This completes a bounded part of the existing manual authoring Memory read /
Projection propagation criterion. Selected Notes already produce targeted
screenplay review with explicit acceptance; that feature remains intact.

## Evidence and ownership

`BeatEditor` passes selected canonical Notes into `contextRequestLifecycle.update`.
The lifecycle checks Notes only for emptiness, then deduplicates on node ID and
script-context revision. A same-node nonempty Notes edit advances neither key, so
the displayed raw AI prompt remains old even though selected Notes and screenplay
impact refresh. `getAiContext` already calls `ai_context_preview`, whose server
owner loads the canonical SQLite project and captures generation context. No
backend/model endpoint or new store is needed. On exact base, six new Notes-only
tests fail and seven maintained cases pass; the log stays outside source Git.

## Decision and gates

Extend the existing request owner's cache identity with exact committed Notes.
Do not trim/normalize nonempty Notes. A changed value clears the displayed old
context, loads canonical context and advances the existing request ID. Its
success/failure/finalizer cannot replace newer Notes/selection/revision requests.
Clearing Notes clears the identity; a restore requests fresh evidence. Retain
ordinary manual Refresh and unchanged-key deduplication. No saved story/script,
draft or proposal write is performed and no acceptance policy changes.

Tests cover Unicode/whitespace-only edits, Notes edit/restore ABA, overlapping
Refresh and late failure/success ordering. Native qualification must exercise
ordinary Notes and raw-prompt controls, with a visibly labelled held real context
response, using unchanged application files. Seed model replies stay explicitly
synthetic; the context read itself requires no inference. Show Bible, timeline,
screenplay and exact fresh Notes in prompt evidence. Original PNGs and sanitized
logs remain artifacts outside Git; display derivatives use JPEG quality 85.

## Alternatives and limits

Waiting for a screenplay edit or requiring manual Refresh leaves a known Notes
change stale. A second context cache or a guessed Pumas endpoint duplicates
ownership. Reuse the public read and request-ID guards. Other metadata/context
membership, unobserved history-only changes, project switching and modality
publication remain separate; this is exact committed Notes propagation, not
real-model quality qualification.
