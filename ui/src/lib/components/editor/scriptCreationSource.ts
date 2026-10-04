import { untrack } from 'svelte';
import { editorState } from '$lib/stores/editor.svelte.js';
import { timelineRenderProjectionState } from '$lib/stores/timelineRenderProjection.svelte.js';
import {
  refreshSelectedNodeEditorProjection,
  selectedNodeEditorProjectionState,
} from '$lib/stores/selectedNodeEditorProjection.svelte.js';

export function getScriptCreationSource() {
  const node = selectedNodeEditorProjectionState.projection?.payload.node;
  if (!node || node.node_id !== editorState.selectedNodeId) return null;
  const timeline = timelineRenderProjectionState.projection;
  if (timeline) {
    const clip = timeline.payload.clips.find((item) => item.node_id === node.node_id);
    if (!clip || clip.start_ms !== node.start_ms || clip.end_ms !== node.end_ms) return null;
  }
  return node;
}

// The Script panel effect tracks selection and the canonical timeline. Editor
// read completion is untracked: it must not create a self-retrying read loop.
// A retime can supersede an in-flight pre-move read; the store's existing
// request IDs and projection versions own admission of all responses.
export function refreshRetimedScriptSource(): Promise<void> {
  const nodeId = editorState.selectedNodeId;
  const clip = timelineRenderProjectionState.projection?.payload.clips.find(
    (item) => item.node_id === nodeId,
  );
  if (!nodeId || !clip) return Promise.resolve();
  const start = clip.start_ms;
  const end = clip.end_ms;
  return untrack(async () => {
    const node = selectedNodeEditorProjectionState.projection?.payload.node;
    if (node?.node_id === nodeId && node.start_ms === start && node.end_ms === end) return;
    await refreshSelectedNodeEditorProjection(nodeId);
  });
}
