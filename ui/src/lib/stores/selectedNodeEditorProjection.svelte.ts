import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { SelectedNodeEditorProjection } from '$lib/selectedNodeEditorTypes.js';
import type { NodeId } from '$lib/timelineTypes.js';

export const selectedNodeEditorProjectionState = $state<{
  selectedNodeId: NodeId | null;
  projection: ProjectionEnvelope<SelectedNodeEditorProjection> | null;
  pending: boolean;
  error?: string;
}>({
  selectedNodeId: null,
  projection: null,
  pending: false,
  error: undefined,
});

let latestRequestId = 0;

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedSelectedNodeEditorProjection(): ProjectionEnvelope<SelectedNodeEditorProjection> | null {
  return selectedNodeEditorProjectionState.projection;
}

export async function refreshSelectedNodeEditorProjection(
  nodeId: NodeId | null = selectedNodeEditorProjectionState.selectedNodeId,
): Promise<ProjectionEnvelope<SelectedNodeEditorProjection>> {
  const requestId = latestRequestId + 1;
  latestRequestId = requestId;
  selectedNodeEditorProjectionState.selectedNodeId = nodeId;
  selectedNodeEditorProjectionState.pending = true;
  selectedNodeEditorProjectionState.error = undefined;

  try {
    const projection = await getSelectedNodeEditorProjection({ node_id: nodeId });
    if (requestId === latestRequestId) {
      selectedNodeEditorProjectionState.projection = projection;
    }
    return projection;
  } catch (error) {
    if (requestId === latestRequestId) {
      selectedNodeEditorProjectionState.error = errorMessage(
        error,
        'Failed to load selected node editor projection',
      );
    }
    throw error;
  } finally {
    if (requestId === latestRequestId) {
      selectedNodeEditorProjectionState.pending = false;
    }
  }
}

export function clearSelectedNodeEditorProjection() {
  latestRequestId += 1;
  selectedNodeEditorProjectionState.selectedNodeId = null;
  selectedNodeEditorProjectionState.projection = null;
  selectedNodeEditorProjectionState.pending = false;
  selectedNodeEditorProjectionState.error = undefined;
}
