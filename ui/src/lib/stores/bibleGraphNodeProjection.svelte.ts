import { createBibleGraphNode } from '$lib/commandApi.js';
import { getBibleGraphNodeProjection } from '$lib/projectionApi.js';
import type {
  BibleGraphNodeCommandResponse,
  BibleGraphNodeId,
  BibleNodeDetailProjection,
  CommandId,
  CreateBibleGraphNodeCommand,
  ProjectionEnvelope,
} from '../types.js';

export interface BibleGraphNodeProjectionKey {
  node_id: BibleGraphNodeId;
}

export const bibleGraphNodeProjectionState = $state<{
  projections: Record<string, ProjectionEnvelope<BibleNodeDetailProjection>>;
  pending: Record<string, boolean>;
  errors: Record<string, string | undefined>;
}>({
  projections: {},
  pending: {},
  errors: {},
});

function cacheKey({ node_id }: BibleGraphNodeProjectionKey): string {
  return encodeURIComponent(node_id);
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedBibleGraphNodeProjection(
  key: BibleGraphNodeProjectionKey,
): ProjectionEnvelope<BibleNodeDetailProjection> | undefined {
  return bibleGraphNodeProjectionState.projections[cacheKey(key)];
}

export function isBibleGraphNodeProjectionPending(key: BibleGraphNodeProjectionKey): boolean {
  return bibleGraphNodeProjectionState.pending[cacheKey(key)] === true;
}

export function getBibleGraphNodeProjectionError(
  key: BibleGraphNodeProjectionKey,
): string | undefined {
  return bibleGraphNodeProjectionState.errors[cacheKey(key)];
}

export async function refreshBibleGraphNodeProjection(
  key: BibleGraphNodeProjectionKey,
): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const projection = await getBibleGraphNodeProjection(key);
    bibleGraphNodeProjectionState.projections[keyString] = projection;
    return projection;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to load bible graph node',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[keyString] = false;
  }
}

export async function createBibleGraphNodeProjection(
  payload: CreateBibleGraphNodeCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: payload.node_id };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const response = await createBibleGraphNode(payload, commandId);
    bibleGraphNodeProjectionState.projections[keyString] = response.projection;
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to create bible graph node',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[keyString] = false;
  }
}

export function clearBibleGraphNodeProjection(key: BibleGraphNodeProjectionKey): void {
  const keyString = cacheKey(key);
  delete bibleGraphNodeProjectionState.projections[keyString];
  delete bibleGraphNodeProjectionState.pending[keyString];
  delete bibleGraphNodeProjectionState.errors[keyString];
}
