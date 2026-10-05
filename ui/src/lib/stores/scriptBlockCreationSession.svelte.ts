import { createScriptBlockCreationDraft } from '$lib/components/editor/scriptBlockCreationDraft.svelte.js';
import {
  applyScriptBlockCreationCommand,
  MAIN_SCRIPT_DOCUMENT_ID,
} from './scriptDocumentProjection.svelte.js';

// View mode changes may destroy every Script panel. The project session owns
// its transient draft and immutable pending submission until reconciliation.
let generation = $state(0);
let draft: ReturnType<typeof createScriptBlockCreationDraft> | null = null;

export function getSessionScriptBlockCreationDraft(): ReturnType<
  typeof createScriptBlockCreationDraft
> {
  const admittedGeneration = generation;
  if (!draft) {
    draft = createScriptBlockCreationDraft({
      documentId: () => MAIN_SCRIPT_DOCUMENT_ID,
      save: (payload, commandId) => {
        if (generation !== admittedGeneration) {
          return Promise.reject(new Error('The project changed before this screenplay save.'));
        }
        return applyScriptBlockCreationCommand(payload, commandId);
      },
    });
  }
  return draft;
}

export function resetSessionScriptBlockCreationDraft(): void {
  generation += 1;
  draft = null;
}
