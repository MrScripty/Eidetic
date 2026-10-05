import { createScriptBlockEditDraft } from '$lib/components/editor/scriptBlockEditDraft.svelte.js';
import {
  applyScriptBlockEditCommand,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';

let generation = $state(0);
type EditDrafts = Record<string, ReturnType<typeof createScriptBlockEditDraft> | undefined>;
let drafts = Object.create(null) as EditDrafts;

export function getSessionScriptBlockEditDraft(documentId: string, blockId: string) {
  const admittedGeneration = generation;
  const key = JSON.stringify([documentId, blockId]);
  let draft = drafts[key];
  if (!draft) {
    const assertCurrent = () => {
      if (generation !== admittedGeneration) {
        throw new Error('The project changed before this screenplay edit.');
      }
    };
    draft = createScriptBlockEditDraft({
      documentId,
      blockId,
      save: async (payload, commandId) => {
        assertCurrent();
        return applyScriptBlockEditCommand(payload, commandId);
      },
      readCurrent: async () => {
        assertCurrent();
        const projection = await refreshScriptDocumentProjection({ document_id: documentId });
        if (projection.payload.document.id !== documentId) return null;
        return (
          projection.payload.segments
            .flatMap((segment) => segment.blocks)
            .find((block) => block.block.id === blockId) ?? null
        );
      },
      reload: async () => {
        assertCurrent();
        return refreshScriptDocumentProjection({ document_id: documentId });
      },
    });
    drafts[key] = draft;
  }
  return draft;
}

export function resetSessionScriptBlockEditDrafts(): void {
  generation += 1;
  drafts = Object.create(null) as EditDrafts;
}
