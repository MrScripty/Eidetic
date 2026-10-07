import {
  bibleGraphNodeProjectionState,
  cacheKey,
  cacheNodeProjection,
} from './bibleGraphNodeDetailProjection.svelte.js';
export {
  bibleGraphNodeProjectionState,
  getCachedBibleGraphNodeProjection,
  isBibleGraphNodeProjectionPending,
  getBibleGraphNodeProjectionError,
  refreshBibleGraphNodeProjection,
  retainBibleGraphNodeDetail,
  refreshOwnedBibleGraphNodeProjections,
  clearBibleGraphNodeProjection,
  clearBibleGraphNodeDetailProjections,
} from './bibleGraphNodeDetailProjection.svelte.js';
export type { BibleGraphNodeProjectionKey } from './bibleGraphNodeDetailProjection.svelte.js';
import {
  createConnectedBibleGraphNode,
  createBibleGraphNode,
  deleteBibleGraphEdge,
  deleteBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphNodeName,
  setBibleGraphEdge,
  setBibleGraphEdgeLabel,
  setBibleGraphField,
  setBibleGraphSnapshotField,
} from '$lib/commandApi.js';
import { getBibleGraphNodeListProjection } from '$lib/projectionApi.js';
import type {
  BibleGraphEdgeLabelCommandResponse,
  BibleGraphNodeCommandResponse,
  BibleGraphRootsCommandResponse,
  BibleGraphEdge,
  BibleGraphNodeId,
  BibleGraphNodeListProjection,
  BibleNodeDetailProjection,
  CreateBibleGraphNodeCommand,
  SetBibleGraphEdgeCommand,
  SetBibleGraphEdgeLabelCommand,
  SetBibleGraphFieldCommand,
  SetBibleGraphNodeNameCommand,
  SetBibleGraphSnapshotFieldCommand,
} from '../bibleGraphTypes.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';
import { invalidateBibleRecall } from './bibleRecallProjection.svelte.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
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

export function getCachedBibleGraphNodeListProjection(): ProjectionEnvelope<BibleGraphNodeListProjection> | null {
  return bibleGraphNodeProjectionState.nodeList;
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
    invalidateBibleRecall();
    const response = await ensureCanonicalBibleRoots(commandId);
    invalidateBibleRecall(response.projection?.version);
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
    invalidateBibleRecall();
    const response = await createBibleGraphNode(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
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

export async function createConnectedBibleGraphNodeProjection(
  parentId: BibleGraphNodeId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: parentId };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    invalidateBibleRecall();
    const response = await createConnectedBibleGraphNode(parentId);
    invalidateBibleRecall(response.projection?.version);
    const confirmedKeyString = cacheKey({ node_id: response.projection.payload.node.id });
    const accepted = cacheNodeProjection(confirmedKeyString, response.projection);
    if (accepted && shouldInvalidateNodeListForNodeProjection(response.projection)) {
      bibleGraphNodeProjectionState.nodeList = null;
    }
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to create connected bible graph node',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[keyString] = false;
  }
}

export async function deleteBibleGraphNodeProjection(
  nodeId: BibleGraphNodeId,
  commandId?: CommandId,
): Promise<BibleGraphRootsCommandResponse> {
  const key = { node_id: nodeId };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;
  bibleGraphNodeProjectionState.nodeListPending = true;
  bibleGraphNodeProjectionState.nodeListError = undefined;

  try {
    invalidateBibleRecall();
    const response = await deleteBibleGraphNode({ node_id: nodeId }, commandId);
    invalidateBibleRecall(response.projection?.version);
    cacheNodeListProjection(response.projection);
    delete bibleGraphNodeProjectionState.projections[keyString];
    delete bibleGraphNodeProjectionState.errors[keyString];
    return response;
  } catch (error) {
    const message = errorMessage(error, 'Failed to delete bible graph node');
    bibleGraphNodeProjectionState.errors[keyString] = message;
    bibleGraphNodeProjectionState.nodeListError = message;
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[keyString] = false;
    bibleGraphNodeProjectionState.nodeListPending = false;
  }
}

export async function setBibleGraphNodeNameProjection(
  payload: SetBibleGraphNodeNameCommand,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const key = { node_id: payload.node_id };
  const keyString = cacheKey(key);
  bibleGraphNodeProjectionState.pending[keyString] = true;
  bibleGraphNodeProjectionState.errors[keyString] = undefined;

  try {
    invalidateBibleRecall();
    const response = await setBibleGraphNodeName(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
    const accepted = cacheNodeProjection(keyString, response.projection);
    if (accepted && shouldInvalidateNodeListForNodeProjection(response.projection)) {
      bibleGraphNodeProjectionState.nodeList = null;
    }
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[keyString] = errorMessage(
      error,
      'Failed to rename bible graph node',
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
    invalidateBibleRecall();
    const response = await setBibleGraphField(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
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
    invalidateBibleRecall();
    const response = await setBibleGraphEdge(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
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

export async function setBibleGraphEdgeLabelProjection(
  edge: BibleGraphEdge,
  payload: SetBibleGraphEdgeLabelCommand,
  commandId?: CommandId,
): Promise<BibleGraphEdgeLabelCommandResponse> {
  const sourceKeyString = cacheKey({ node_id: edge.from_node_id });
  const targetKeyString = cacheKey({ node_id: edge.to_node_id });
  bibleGraphNodeProjectionState.pending[sourceKeyString] = true;
  bibleGraphNodeProjectionState.errors[sourceKeyString] = undefined;
  try {
    invalidateBibleRecall();
    const response = await setBibleGraphEdgeLabel(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
    if (!response.projection) return response;
    const accepted = cacheNodeProjection(sourceKeyString, response.projection);
    if (accepted && targetKeyString !== sourceKeyString) {
      delete bibleGraphNodeProjectionState.projections[targetKeyString];
      delete bibleGraphNodeProjectionState.errors[targetKeyString];
    }
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[sourceKeyString] = errorMessage(
      error,
      'Failed to save relationship label',
    );
    throw error;
  } finally {
    bibleGraphNodeProjectionState.pending[sourceKeyString] = false;
  }
}

export async function deleteBibleGraphEdgeProjection(
  edge: BibleGraphEdge,
  commandId?: CommandId,
): Promise<BibleGraphNodeCommandResponse> {
  const sourceKey = { node_id: edge.from_node_id };
  const targetKey = { node_id: edge.to_node_id };
  const sourceKeyString = cacheKey(sourceKey);
  const targetKeyString = cacheKey(targetKey);
  bibleGraphNodeProjectionState.pending[sourceKeyString] = true;
  bibleGraphNodeProjectionState.errors[sourceKeyString] = undefined;

  try {
    invalidateBibleRecall();
    const response = await deleteBibleGraphEdge({ edge_id: edge.id }, commandId);
    invalidateBibleRecall(response.projection?.version);
    const accepted = cacheNodeProjection(sourceKeyString, response.projection);
    if (accepted && targetKeyString !== sourceKeyString) {
      delete bibleGraphNodeProjectionState.projections[targetKeyString];
      delete bibleGraphNodeProjectionState.errors[targetKeyString];
    }
    return response;
  } catch (error) {
    bibleGraphNodeProjectionState.errors[sourceKeyString] = errorMessage(
      error,
      'Failed to delete bible graph edge',
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
    invalidateBibleRecall();
    const response = await setBibleGraphSnapshotField(payload, commandId);
    invalidateBibleRecall(response.projection?.version);
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

export function clearBibleGraphNodeListProjection(): void {
  bibleGraphNodeProjectionState.nodeList = null;
  bibleGraphNodeProjectionState.nodeListPending = false;
  bibleGraphNodeProjectionState.nodeListError = undefined;
}
