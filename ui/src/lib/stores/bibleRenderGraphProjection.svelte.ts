import { getBibleRenderGraphProjection } from '$lib/projectionApi.js';
import type {
  BibleRenderGraphProjection,
  BibleRenderGraphProjectionRequest,
} from '$lib/bibleGraphTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

const DEFAULT_RENDER_GRAPH_MAX_NODES = 200;
const DEFAULT_RENDER_GRAPH_NEIGHBORHOOD_DEPTH = 1;

export const bibleRenderGraphProjectionState = $state<{
  projection: ProjectionEnvelope<BibleRenderGraphProjection> | null;
  pending: boolean;
  error?: string;
}>({
  projection: null,
  pending: false,
  error: undefined,
});

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedBibleRenderGraphProjection(): ProjectionEnvelope<BibleRenderGraphProjection> | null {
  return bibleRenderGraphProjectionState.projection;
}

export function bibleRenderGraphRequestForTimelineSelection(
  selectedTimelineNodeId: string | null | undefined,
): BibleRenderGraphProjectionRequest {
  return bibleRenderGraphRequestForWorkspaceSelection({
    selectedTimelineNodeId,
  });
}

export function bibleRenderGraphRequestForWorkspaceSelection({
  selectedTimelineNodeId,
  selectedGraphNodeId,
}: {
  selectedTimelineNodeId?: string | null;
  selectedGraphNodeId?: string | null;
}): BibleRenderGraphProjectionRequest {
  return {
    ...(selectedTimelineNodeId ? { selected_timeline_node_id: selectedTimelineNodeId } : {}),
    ...(selectedGraphNodeId ? { selected_node_id: selectedGraphNodeId } : {}),
    neighborhood_depth: DEFAULT_RENDER_GRAPH_NEIGHBORHOOD_DEPTH,
    max_nodes: DEFAULT_RENDER_GRAPH_MAX_NODES,
  };
}

function replaceBibleRenderGraphProjectionIfFresh(
  projection: ProjectionEnvelope<BibleRenderGraphProjection>,
): void {
  if (shouldReplaceProjection(bibleRenderGraphProjectionState.projection, projection)) {
    bibleRenderGraphProjectionState.projection = projection;
  }
}

export async function refreshBibleRenderGraphProjection(
  query?: BibleRenderGraphProjectionRequest,
): Promise<ProjectionEnvelope<BibleRenderGraphProjection>> {
  bibleRenderGraphProjectionState.pending = true;
  bibleRenderGraphProjectionState.error = undefined;

  try {
    const projection = await getBibleRenderGraphProjection(query);
    replaceBibleRenderGraphProjectionIfFresh(projection);
    return projection;
  } catch (error) {
    bibleRenderGraphProjectionState.error = errorMessage(
      error,
      'Failed to load bible render graph projection',
    );
    throw error;
  } finally {
    bibleRenderGraphProjectionState.pending = false;
  }
}

export function clearBibleRenderGraphProjection(): void {
  bibleRenderGraphProjectionState.projection = null;
  bibleRenderGraphProjectionState.pending = false;
  bibleRenderGraphProjectionState.error = undefined;
}
