import { getBibleRenderGraphProjection } from '$lib/projectionApi.js';
import type {
  BibleRenderGraphProjection,
  BibleRenderGraphProjectionRequest,
} from '$lib/bibleGraphTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

const DEFAULT_RENDER_GRAPH_MAX_NODES = 200;
const DEFAULT_RENDER_GRAPH_MAX_EDGES = 500;
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

let activeBibleRenderGraphRequest: BibleRenderGraphProjectionRequest =
  defaultBibleRenderGraphRequest();

function defaultBibleRenderGraphRequest(): BibleRenderGraphProjectionRequest {
  return {
    neighborhood_depth: DEFAULT_RENDER_GRAPH_NEIGHBORHOOD_DEPTH,
    max_nodes: DEFAULT_RENDER_GRAPH_MAX_NODES,
    max_edges: DEFAULT_RENDER_GRAPH_MAX_EDGES,
  };
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedBibleRenderGraphProjection(): ProjectionEnvelope<BibleRenderGraphProjection> | null {
  return bibleRenderGraphProjectionState.projection;
}

export function getActiveBibleRenderGraphProjectionRequest(): BibleRenderGraphProjectionRequest {
  return { ...activeBibleRenderGraphRequest };
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
  activeTimelineMs,
  focusedRootId,
  search,
}: {
  selectedTimelineNodeId?: string | null;
  selectedGraphNodeId?: string | null;
  activeTimelineMs?: number | null;
  focusedRootId?: string | null;
  search?: string | null;
}): BibleRenderGraphProjectionRequest {
  const normalizedSearch = search?.trim();
  return {
    ...(focusedRootId ? { focused_root_id: focusedRootId } : {}),
    ...(selectedTimelineNodeId ? { selected_timeline_node_id: selectedTimelineNodeId } : {}),
    ...(selectedGraphNodeId ? { selected_node_id: selectedGraphNodeId } : {}),
    ...(activeTimelineMs !== null && activeTimelineMs !== undefined
      ? { active_timeline_ms: Math.max(0, Math.trunc(activeTimelineMs)) }
      : {}),
    ...(normalizedSearch ? { search: normalizedSearch } : {}),
    neighborhood_depth: DEFAULT_RENDER_GRAPH_NEIGHBORHOOD_DEPTH,
    max_nodes: DEFAULT_RENDER_GRAPH_MAX_NODES,
    max_edges: DEFAULT_RENDER_GRAPH_MAX_EDGES,
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
  activeBibleRenderGraphRequest = query ?? defaultBibleRenderGraphRequest();
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
  activeBibleRenderGraphRequest = defaultBibleRenderGraphRequest();
}
