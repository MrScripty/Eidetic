import type { createScriptBlockEditDraft } from './scriptBlockEditDraft.svelte.js';
import type { createScriptBlockCreationDraft } from './scriptBlockCreationDraft.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';

// Copying starts an ordinary new draft, never restores canon or retires the source.
export function copyMissingScriptDraft(
  editor: ReturnType<typeof createScriptBlockEditDraft>,
  composer: ReturnType<typeof createScriptBlockCreationDraft>,
  source: SelectedNodeEditorNode | null,
): boolean {
  const draft = editor.state;
  if (
    !source ||
    !draft.editing ||
    !draft.baseRevision ||
    draft.saving ||
    draft.comparing ||
    draft.uncertain ||
    draft.removal.active ||
    composer.state.writing ||
    composer.state.saving ||
    composer.state.uncertain
  )
    return false;
  return composer.begin(source, { text: draft.text, kind: editor.originalKind });
}
