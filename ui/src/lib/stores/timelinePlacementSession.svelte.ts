import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import { createCommandId } from '$lib/commandTransport.js';
import { createTimelinePlacementDraft } from '$lib/components/editor/timelinePlacementDraft.svelte.js';
import { getEditorSessionGeneration } from './editor.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { applyTimelineNodeRangeCommand } from './timelineRenderProjection.svelte.js';

let session = -1;
type PlacementDrafts = Record<string, ReturnType<typeof createTimelinePlacementDraft> | undefined>;
let drafts = Object.create(null) as PlacementDrafts;

/** Transient placement intents survive panel/selection navigation in this session. */
export function getSessionTimelinePlacementDraft(node: SelectedNodeEditorNode) {
  const admitted = getEditorSessionGeneration();
  if (session !== admitted) {
    session = admitted;
    drafts = Object.create(null) as PlacementDrafts;
  }
  let draft = drafts[node.node_id];
  if (!draft) {
    const assertCurrent = () => {
      if (admitted !== getEditorSessionGeneration())
        throw new Error('The project changed before this placement edit.');
    };
    draft = createTimelinePlacementDraft({
      nodeId: node.node_id,
      commandId: createCommandId,
      async read() {
        assertCurrent();
        const projection = await refreshSelectedNodeEditorProjection(node.node_id);
        assertCurrent();
        const read = projection.payload.node;
        return read?.node_id === node.node_id ? (read.range_read ?? null) : null;
      },
      async apply(payload, commandId) {
        assertCurrent();
        return applyTimelineNodeRangeCommand(payload, commandId);
      },
    });
    drafts[node.node_id] = draft;
  }
  return draft;
}
