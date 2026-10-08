import { untrack } from 'svelte';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import { createCommandId } from '$lib/commandTransport.js';
import { createTimelineTitleDraft } from '$lib/components/editor/timelineTitleDraft.svelte.js';
import { getEditorSessionGeneration } from './editor.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { applyTimelineNodeNameCommand } from './timelineRenderProjection.svelte.js';

let session = -1;
type TitleDrafts = Record<string, ReturnType<typeof createTimelineTitleDraft> | undefined>;
let drafts = Object.create(null) as TitleDrafts;

/** Transient title intents survive panel/selection navigation in this session. */
export function getSessionTimelineTitleDraft(node: SelectedNodeEditorNode) {
  return untrack(() => getDraft(node));
}

function getDraft(node: SelectedNodeEditorNode) {
  const admitted = getEditorSessionGeneration();
  if (session !== admitted) {
    session = admitted;
    drafts = Object.create(null) as TitleDrafts;
  }
  let draft = drafts[node.node_id];
  if (!draft) {
    const assertCurrent = () => {
      if (admitted !== getEditorSessionGeneration())
        throw new Error('The project changed before this title edit.');
    };
    draft = createTimelineTitleDraft({
      nodeId: node.node_id,
      commandId: createCommandId,
      async read() {
        assertCurrent();
        const projection = await refreshSelectedNodeEditorProjection(node.node_id);
        assertCurrent();
        const read = projection.payload.node;
        return read?.node_id === node.node_id ? (read.name_read ?? null) : null;
      },
      async apply(payload, commandId) {
        assertCurrent();
        return applyTimelineNodeNameCommand(payload, commandId);
      },
    });
    drafts[node.node_id] = draft;
  }
  return draft;
}
