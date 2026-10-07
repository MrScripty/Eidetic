import { getBibleGraphNodeProjection } from '$lib/projectionApi.js';
import type {
  BibleGraphNodeId,
  BibleNodeDetailProjection,
  BibleGraphNodeListProjection,
} from '$lib/bibleGraphTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { getEditorSessionGeneration } from './editor.svelte.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';
import { createBibleGraphNodeReadOwners } from './bibleGraphNodeReadOwners.js';

export interface BibleGraphNodeProjectionKey {
  node_id: BibleGraphNodeId;
}

// Shared with the existing command/list facade; this remains the sole detail cache.
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

const { requests, inspectors, createInspectors } = createBibleGraphNodeReadOwners();
export const cacheKey = ({ node_id }: BibleGraphNodeProjectionKey) => encodeURIComponent(node_id);

export function cacheNodeProjection(
  key: string,
  projection: ProjectionEnvelope<BibleNodeDetailProjection>,
): boolean {
  if (key !== cacheKey({ node_id: projection.payload.node.id }))
    throw new Error('Bible detail returned another node.');
  if (!shouldReplaceProjection(bibleGraphNodeProjectionState.projections[key] ?? null, projection))
    return false;
  bibleGraphNodeProjectionState.projections[key] = projection;
  return true;
}

export function getCachedBibleGraphNodeProjection(key: BibleGraphNodeProjectionKey) {
  return bibleGraphNodeProjectionState.projections[cacheKey(key)];
}
export function isBibleGraphNodeProjectionPending(key: BibleGraphNodeProjectionKey): boolean {
  return bibleGraphNodeProjectionState.pending[cacheKey(key)] === true;
}
export function getBibleGraphNodeProjectionError(key: BibleGraphNodeProjectionKey) {
  return bibleGraphNodeProjectionState.errors[cacheKey(key)];
}

export async function refreshBibleGraphNodeProjection(
  key: BibleGraphNodeProjectionKey,
  isOwned: () => boolean = () => true,
): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  const encoded = cacheKey(key),
    token = {},
    session = getEditorSessionGeneration();
  requests.set(encoded, token);
  const current = () => session === getEditorSessionGeneration() && requests.get(encoded) === token;
  bibleGraphNodeProjectionState.pending[encoded] = true;
  bibleGraphNodeProjectionState.errors[encoded] = undefined;
  try {
    const projection = await getBibleGraphNodeProjection(key);
    if (!current() || !isOwned()) throw new Error('The Bible detail owner changed during refresh.');
    cacheNodeProjection(encoded, projection);
    return projection;
  } catch (error) {
    if (current() && isOwned())
      bibleGraphNodeProjectionState.errors[encoded] =
        error instanceof Error ? error.message : 'Failed to load bible graph node';
    throw error;
  } finally {
    if (current()) bibleGraphNodeProjectionState.pending[encoded] = false;
  }
}

/** Refresh visible inspectors without removing projections and their dirty forms. */
export async function refreshOwnedBibleGraphNodeProjections(): Promise<void> {
  const session = getEditorSessionGeneration();
  await Promise.all(
    [...inspectors.entries()].map(async ([encoded, owners]) => {
      const active = () =>
        session === getEditorSessionGeneration() &&
        inspectors.get(encoded) === owners &&
        owners.size > 0;
      if (!active()) return;
      await refreshBibleGraphNodeProjection({ node_id: decodeURIComponent(encoded) }, active);
    }),
  );
}

/** Both visible Bible inspectors may own the same node. Releases are idempotent. */
export function retainBibleGraphNodeDetail(key: BibleGraphNodeProjectionKey): () => void {
  const encoded = cacheKey(key),
    owner = {},
    session = getEditorSessionGeneration();
  const owners = inspectors.get(encoded) ?? createInspectors();
  owners.add(owner);
  inspectors.set(encoded, owners);
  const active = () =>
    session === getEditorSessionGeneration() &&
    inspectors.get(encoded) === owners &&
    owners.size > 0;
  void refreshBibleGraphNodeProjection(key, active).catch(() => {});
  return () => {
    if (!owners.delete(owner) || inspectors.get(encoded) !== owners || owners.size) return;
    inspectors.delete(encoded);
    clearBibleGraphNodeProjection(key);
  };
}

export function clearBibleGraphNodeProjection(key: BibleGraphNodeProjectionKey): void {
  const encoded = cacheKey(key);
  requests.delete(encoded);
  delete bibleGraphNodeProjectionState.projections[encoded];
  delete bibleGraphNodeProjectionState.pending[encoded];
  delete bibleGraphNodeProjectionState.errors[encoded];
}

export function clearBibleGraphNodeDetailProjections(): void {
  requests.clear();
  inspectors.clear();
  bibleGraphNodeProjectionState.projections = {};
  bibleGraphNodeProjectionState.pending = {};
  bibleGraphNodeProjectionState.errors = {};
}
