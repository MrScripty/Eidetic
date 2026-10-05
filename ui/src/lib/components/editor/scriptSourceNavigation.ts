import { editorState } from '$lib/stores/editor.svelte.js';
import { scrollToNode } from '$lib/stores/timeline.svelte.js';
import { getCachedTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';
import { refreshSelectedNodeEditorProjection } from '$lib/stores/selectedNodeEditorProjection.svelte.js';

export function getScriptSourceClip(nodeId: string | null | undefined) {
  if (!nodeId) return null;
  return (
    getCachedTimelineRenderProjection()?.payload.clips.find((clip) => clip.node_id === nodeId) ??
    null
  );
}

export async function showScriptSource(nodeId: string | null | undefined): Promise<boolean> {
  // Re-read the canonical clip at activation: a previously rendered link may
  // have been retimed or removed. Navigation never reconstructs source ranges
  // from screenplay placement or changes an in-progress writing draft.
  const clip = getScriptSourceClip(nodeId);
  if (!clip) return false;
  editorState.selectedNodeId = clip.node_id;
  editorState.selectedLevel = clip.level;
  scrollToNode(clip.start_ms, clip.end_ms);
  await refreshSelectedNodeEditorProjection(clip.node_id);
  return true;
}
