import {
  createBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphEdge,
  setBibleGraphField,
  setBibleGraphSnapshotField,
} from '$lib/commandApi.js';
import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
} from '$lib/projectionApi.js';
import type {
  BibleGraphNodeCommandResponse,
  BibleGraphRootsCommandResponse,
  BibleGraphNodeId,
  BibleGraphNodeListProjection,
  BibleNodeDetailProjection,
  CreateBibleGraphNodeCommand,
  SetBibleGraphEdgeCommand,
  SetBibleGraphFieldCommand,
  SetBibleGraphSnapshotFieldCommand,
} from '../bibleGraphTypes.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export interface BibleGraphNodeProjectionKey {
  node_id: BibleGraphNodeId;
}

export const bibleGraphNodeProjectionState = $state<{
  projections: Record<string, ProjectionEnvelope<BibleNodeDetailProjection>>;
  pending: Record<string, boolean>;
  errors: Record<string, string | undefined>;
  nodeList: ProjectionEnvelope<BibleGraphNodeListProjection> | null;
  nodeListPending: boolean;
  nodeListError?: string;
}>({
  projections: {},
  pending: {},
  errors: {},
  nodeList: null,
  nodeListPending: false,
  nodeListError: undefined,
});

function cacheKey({ node_id }: BibleGraphNodeProjectionKey): string {
  return encodeURIComponent(node_id);
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function cacheNodeProjection(
  keyString: string,
  projection: ProjectionEnvelope<BibleNodeDetailProjection>,
): boolean {
  if (
    !shouldReplaceProjection(
      bibleGraphNodeProjectionState.projections[keyString] ?? null,
      projection,
    )
  ) {
    return false;
  }

  bibleGraphNodeProjectionState.projections[keyString] = projection;
  return true;
}

function cacheNodeListProjection(
  projection: ProjectionEnvelope<BibleGraphNodeListProjection>,
): void {
  if (shouldReplaceProjection(bibleGraphNodeProjectionState.nodeList, projection)) {
    bibleGraphNodeProjectionState.nodeList = projection;
  }
}

function shouldInvalidateNodeListForNodeProjection(
  projection: ProjectionEnvelope<BibleNodeDetailProjection>,
): boolean {
  return (
    bibleGraphNodeProjectionState.nodeList === null ||
    projection.version >= bibleGraphNodeProjectionState.nodeList.version
  );
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

export function getCachedBibleGraphNodeListProjection(): ProjectionEnvelope<BibleGraphNodeListProjection> | null {
  return bibleGraphNodeProjectionState.nodeList;
}

export async function refreshBibleGraphNodeProjection(
  key: BibleGraphNodeProjectionKey,
): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const projection = await getBibleGraphNodeProjection(key);
    cacheNodeProjection(keyString, projection);
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

export async function refreshBibleGraphNodeListProjection(): Promise<
  ProjectionEnvelope<BibleGraphNodeListProjection>
> {
  bibleGraphNodeProjectionState.nodeListPending = true;
  bibleGraphNodeProjectionState.nodeListError = undefined;

  try {
    const projection = await getBibleGraphNodeListProjection();
    cacheNodeListProjection(projection);
    return projection;
  } catch (error) {
    bibleGraphNodeProjectionState.nodeListError = errorMessage(
      error,
      'Failed to load bible graph nodes',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.nodeListPending = false;
  }
}

export async function ensureCanonicalBibleRootProjections(
  commandId?: CommandId,
): Promise<BibleGraphRootsCommandResponse> {
  bibleGraphNodeProjectionState.nodeListPending = true;
  bibleGraphNodeProjectionState.nodeListError = undefined;

  try {
    const response = await ensureCanonicalBibleRoots(commandId);
    cacheNodeListProjection(response.projection);
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.nodeListError = errorMessage(
      error,
      'Failed to ensure canonical bible roots',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.nodeListPending = false;
  }
}

export async function createBibleGraphNodeProjection(
  payload: CreateBibleGraphNodeCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: payload.node_id ?? `pending-create:${commandId ?? 'new'}` };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const response = await createBibleGraphNode(payload, commandId);
    const confirmedKeyString = cacheKey({ node_id: response.projection.payload.node.id });
    const accepted = cacheNodeProjection(confirmedKeyString, response.projection);
    if (accepted && shouldInvalidateNodeListForNodeProjection(response.projection)) {
      bibleGraphNodeProjectionState.nodeList = null;
    }
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

export async function setBibleGraphFieldProjection(
  payload: SetBibleGraphFieldCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: payload.node_id };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const response = await setBibleGraphField(payload, commandId);
    cacheNodeProjection(keyString, response.projection);
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to set bible graph field',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[keyString] = false;
  }
}

export async function setBibleGraphEdgeProjection(
  payload: SetBibleGraphEdgeCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const sourceKey = { node_id: payload.from_node_id };
  const targetKey = { node_id: payload.to_node_id };
  const sourceKeyString = cacheKey(sourceKey);
  const targetKeyString = cacheKey(targetKey);
  bibleGraphNodeProjectionState.pending[sourceKeyString] = true;
  bibleGraphNodeProjectionState.errors[sourceKeyString] = undefined;

  try {
    const response = await setBibleGraphEdge(payload, commandId);
    const accepted = cacheNodeProjection(sourceKeyString, response.projection);
    if (accepted && targetKeyString !== sourceKeyString) {
      delete bibleGraphNodeProjectionState.projections[targetKeyString];
      delete bibleGraphNodeProjectionState.errors[targetKeyString];
    }
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[sourceKeyString] = errorMessage(
      error,
      'Failed to set bible graph edge',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[sourceKeyString] = false;
  }
}

export async function setBibleGraphSnapshotFieldProjection(
  payload: SetBibleGraphSnapshotFieldCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: payload.node_id };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    const response = await setBibleGraphSnapshotField(payload, commandId);
    cacheNodeProjection(keyString, response.projection);
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to set bible graph snapshot field',
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

export function clearBibleGraphNodeListProjection(): void {
  bibleGraphNodeProjectionState.nodeList = null;
  bibleGraphNodeProjectionState.nodeListPending = false;
  bibleGraphNodeProjectionState.nodeListError = undefined;
}
