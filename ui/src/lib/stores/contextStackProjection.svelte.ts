import { getContextStackProjection } from '$lib/projectionApi.js';
import type { ContextStackProjection } from '$lib/contextInfluenceTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { NodeId } from '$lib/timelineTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export const contextStackProjectionState = $state<{
  targetNodeId: NodeId | null;
  projection: ProjectionEnvelope<ContextStackProjection> | null;
  pending: boolean;
  error?: string;
}>({
  targetNodeId: null,
  projection: null,
  pending: false,
  error: undefined,
});

let latestRequestId = 0;

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedContextStackProjection(): ProjectionEnvelope<ContextStackProjection> | null {
  return contextStackProjectionState.projection;
}

function cacheContextStackProjection(projection: ProjectionEnvelope<ContextStackProjection>): void {
  if (shouldReplaceProjection(contextStackProjectionState.projection, projection)) {
    contextStackProjectionState.projection = projection;
  }
}

export async function refreshContextStackProjection(
  targetNodeId: NodeId,
): Promise<ProjectionEnvelope<ContextStackProjection>> {
  const requestId = latestRequestId + 1;
  latestRequestId = requestId;
  contextStackProjectionState.targetNodeId = targetNodeId;
  contextStackProjectionState.pending = true;
  contextStackProjectionState.error = undefined;

  try {
    const projection = await getContextStackProjection({ target_node_id: targetNodeId });
    if (requestId === latestRequestId) {
      cacheContextStackProjection(projection);
    }
    return projection;
  } catch (error) {
    if (requestId === latestRequestId) {
      contextStackProjectionState.error = errorMessage(
        error,
        'Failed to load context stack projection',
      );
    }
    throw error;
  } finally {
    if (requestId === latestRequestId) {
      contextStackProjectionState.pending = false;
    }
  }
}

export function clearContextStackProjection(): void {
  latestRequestId += 1;
  contextStackProjectionState.targetNodeId = null;
  contextStackProjectionState.projection = null;
  contextStackProjectionState.pending = false;
  contextStackProjectionState.error = undefined;
}
