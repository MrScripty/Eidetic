import { untrack } from 'svelte';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import { createCommandId } from '$lib/commandTransport.js';
import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import { createTimelineReorderDraft } from '$lib/components/editor/timelineReorderDraft.svelte.js';
import { editorState, getEditorSessionGeneration } from './editor.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import { applyTimelineSiblingReorderCommand } from './timelineRenderProjection.svelte.js';

let session = -1;
type ReorderDrafts = Record<string, ReturnType<typeof createTimelineReorderDraft> | undefined>;
let drafts = Object.create(null) as ReorderDrafts;

/** Navigation retains each clip's intent; closing/reopening a project ends custody. */
export function getSessionTimelineReorderDraft(node: SelectedNodeEditorNode) {
  return untrack(() => {
    const admitted = getEditorSessionGeneration();
    if (session !== admitted) {
      session = admitted;
      drafts = Object.create(null) as ReorderDrafts;
    }
    let draft = drafts[node.node_id];
    if (!draft) {
      const assertCurrent = () => {
        if (admitted !== getEditorSessionGeneration())
          throw new Error('The project changed before this sibling reorder.');
      };
      draft = createTimelineReorderDraft({
        nodeId: node.node_id,
        commandId: createCommandId,
        async read() {
          assertCurrent();
          // Independent read never retargets the selected projection cache.
          const projection = await getSelectedNodeEditorProjection({ node_id: node.node_id });
          assertCurrent();
          const read = projection.payload.node;
          return read?.node_id === node.node_id ? (read.order_read ?? null) : null;
        },
        async apply(payload, commandId) {
          assertCurrent();
          const response = await applyTimelineSiblingReorderCommand(payload, commandId);
          assertCurrent();
          invalidateScriptContext();
          if (editorState.selectedNodeId === node.node_id) {
            // Projection refresh failure cannot turn an owned save receipt into
            // an uncertain acknowledgement. This cache reports its own errors.
            void refreshSelectedNodeEditorProjection(
              node.node_id,
              () =>
                admitted === getEditorSessionGeneration() &&
                editorState.selectedNodeId === node.node_id,
            ).catch(() => {});
          }
          return response.order_read ?? null;
        },
      });
      drafts[node.node_id] = draft;
    }
    return draft;
  });
}
