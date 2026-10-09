import { untrack } from 'svelte';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import { createCommandId } from '$lib/commandTransport.js';
import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import { createTimelineNotesDraft } from '$lib/components/editor/timelineNotesDraft.svelte.js';
import { editorState, getEditorSessionGeneration } from './editor.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import { applyTimelineNodeNotesCommand } from './timelineRenderProjection.svelte.js';

let session = -1;
type NotesDrafts = Record<string, ReturnType<typeof createTimelineNotesDraft> | undefined>;
let drafts = Object.create(null) as NotesDrafts;

/** Navigation retains each clip's intent; closing/reopening a project ends custody. */
export function getSessionTimelineNotesDraft(node: SelectedNodeEditorNode) {
  return untrack(() => {
    const admitted = getEditorSessionGeneration();
    if (session !== admitted) {
      session = admitted;
      drafts = Object.create(null) as NotesDrafts;
    }
    let draft = drafts[node.node_id];
    if (!draft) {
      const assertCurrent = () => {
        if (admitted !== getEditorSessionGeneration())
          throw new Error('The project changed before this Notes edit.');
      };
      draft = createTimelineNotesDraft({
        nodeId: node.node_id,
        commandId: createCommandId,
        async read() {
          assertCurrent();
          // Independent read never retargets the selected projection cache.
          const projection = await getSelectedNodeEditorProjection({ node_id: node.node_id });
          assertCurrent();
          const read = projection.payload.node;
          return read?.node_id === node.node_id ? (read.notes_read ?? null) : null;
        },
        async apply(payload, commandId) {
          assertCurrent();
          const response = await applyTimelineNodeNotesCommand(payload, commandId);
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
          return response.notes_read ?? null;
        },
      });
      drafts[node.node_id] = draft;
    }
    return draft;
  });
}
