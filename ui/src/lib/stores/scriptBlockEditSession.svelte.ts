import { untrack } from 'svelte';
import { createScriptBlockEditDraft } from '$lib/components/editor/scriptBlockEditDraft.svelte.js';
import {
  applyScriptBlockEditCommand,
  applyScriptBlockRemovalCommand,
  refreshScriptDocumentProjection,
} from './scriptDocumentProjection.svelte.js';

let generation = $state(0);
type EditDrafts = Record<string, ReturnType<typeof createScriptBlockEditDraft> | undefined>;
let drafts = $state.raw<EditDrafts>(Object.create(null));

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
      remove: async (payload, commandId) => {
        assertCurrent();
        return applyScriptBlockRemovalCommand(payload, commandId);
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
    // Consumers obtain this persistent owner in a derived read. Registration
    // must not become a forbidden mutation of that consuming reaction.
    untrack(() => {
      drafts = { ...drafts, [key]: draft };
    });
  }
  return draft;
}

// Keep an uncertain removal reachable if ScriptChanged refresh has already
// removed its block. These are the same per-block authoring owners, not canon.
export function getOrphanedScriptBlockRemovals(documentId: string, visibleBlockIds: string[]) {
  return Object.entries(drafts).flatMap(([key, draft]) => {
    const [document, block] = JSON.parse(key) as [string, string];
    return document === documentId &&
      draft?.state.removal.active &&
      !visibleBlockIds.includes(block)
      ? [draft]
      : [];
  });
}

// Canonical disappearance must not hide a still-owned edit or exact retry.
export function getMissingScriptBlockDrafts(documentId: string, visibleBlockIds: string[]) {
  return Object.entries(drafts).flatMap(([key, draft]) => {
    const [document, block] = JSON.parse(key) as [string, string];
    return document === documentId && draft?.state.editing && !visibleBlockIds.includes(block)
      ? [draft]
      : [];
  });
}

export function resetSessionScriptBlockEditDrafts(): void {
  generation += 1;
  drafts = Object.create(null) as EditDrafts;
}
