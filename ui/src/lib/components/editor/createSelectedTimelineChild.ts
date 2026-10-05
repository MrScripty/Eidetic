import { editorState, getEditorSessionGeneration } from '$lib/stores/editor.svelte.js';
import { applyCreateTimelineChildFromParentCommand } from '$lib/stores/timelineRenderProjection.svelte.js';

/** Reuse canonical parent-derived creation; late acknowledgements never retarget
 * a different selection, project lifetime or unmounted editor. */
export async function createSelectedTimelineChild(
  parentId: string,
  isMounted: () => boolean,
): Promise<string | null> {
  const session = getEditorSessionGeneration();
  const childId = crypto.randomUUID();
  const result = await applyCreateTimelineChildFromParentCommand({
    parent_id: parentId,
    node_id: childId,
  });
  if (
    !isMounted() ||
    session !== getEditorSessionGeneration() ||
    editorState.selectedNodeId !== parentId
  ) {
    return null;
  }
  const child = result.projection.payload.clips.find((clip) => clip.node_id === childId);
  if (!child || child.parent_id !== parentId) {
    throw new Error('Created child is missing from its canonical timeline projection');
  }
  editorState.selectedNodeId = childId;
  editorState.selectedLevel = child.level;
  return childId;
}
