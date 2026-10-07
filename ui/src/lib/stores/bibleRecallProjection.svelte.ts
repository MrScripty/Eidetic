import type { BibleRecallProjection, BibleRecallRequest } from '$lib/bibleRecallTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { getBibleRecallProjection } from '$lib/projectionApi.js';
import { getEditorSessionGeneration } from './editor.svelte.js';

export const bibleRecallState = $state<{
  anchor: string | null;
  projection: ProjectionEnvelope<BibleRecallProjection> | null;
  pending: boolean;
  error: string | null;
  invalidated: boolean;
}>({ anchor: null, projection: null, pending: false, error: null, invalidated: false });

let requestGeneration = 0;
let versionFloor = 0;
const inspectors: { anchor: string; session: number }[] = [];

/** Revoke inspection work without claiming that canonical facts changed. */
export function revokeBibleRecall(): void {
  requestGeneration += 1;
  versionFloor = Math.max(versionFloor, bibleRecallState.projection?.version ?? 0);
  bibleRecallState.projection = null;
  bibleRecallState.pending = false;
  bibleRecallState.error = null;
}

/** Shared inspectors retain evidence until the last current owner leaves. */
export function retainBibleRecallInspector(anchor: string): () => void {
  selectBibleRecallAnchor(anchor);
  const owner = { anchor, session: getEditorSessionGeneration() };
  inspectors.push(owner);
  return () => {
    const index = inspectors.indexOf(owner);
    if (index < 0) return;
    inspectors.splice(index, 1);
    if (
      bibleRecallState.anchor === owner.anchor &&
      getEditorSessionGeneration() === owner.session &&
      !inspectors.some((other) => other.anchor === owner.anchor && other.session === owner.session)
    ) {
      revokeBibleRecall();
    }
  };
}

export function clearBibleRecall(): void {
  requestGeneration += 1;
  versionFloor = 0;
  bibleRecallState.anchor = null;
  bibleRecallState.projection = null;
  bibleRecallState.pending = false;
  bibleRecallState.error = null;
  bibleRecallState.invalidated = false;
}

export function selectBibleRecallAnchor(anchor: string | null): void {
  if (bibleRecallState.anchor === anchor) return;
  clearBibleRecall();
  bibleRecallState.anchor = anchor;
}

/** Bible writes/events revoke outstanding reads as well as displayed evidence. */
export function invalidateBibleRecall(version?: number): void {
  versionFloor = Math.max(versionFloor, version ?? 0, bibleRecallState.projection?.version ?? 0);
  bibleRecallState.invalidated ||= bibleRecallState.projection !== null || bibleRecallState.pending;
  revokeBibleRecall();
}

export async function recallBibleFacts(query: BibleRecallRequest): Promise<void> {
  selectBibleRecallAnchor(query.anchor_node_id);
  const generation = ++requestGeneration;
  const session = getEditorSessionGeneration();
  const active = () =>
    generation === requestGeneration &&
    session === getEditorSessionGeneration() &&
    bibleRecallState.anchor === query.anchor_node_id;
  bibleRecallState.projection = null;
  bibleRecallState.pending = true;
  bibleRecallState.error = null;
  try {
    const projection = await getBibleRecallProjection(query);
    if (!active()) return;
    if (
      projection.version < versionFloor ||
      projection.payload.request.anchor_node_id !== query.anchor_node_id ||
      projection.payload.request.story_time_ms !== query.story_time_ms ||
      projection.payload.request.direction !== query.direction ||
      projection.payload.request.neighbor_limit !== query.neighbor_limit ||
      JSON.stringify(projection.payload.request.edge_kinds) !== JSON.stringify(query.edge_kinds)
    ) {
      bibleRecallState.error = 'Recall evidence is stale. Recall again for current facts.';
      return;
    }
    versionFloor = projection.version;
    bibleRecallState.projection = projection;
    bibleRecallState.invalidated = false;
  } catch (error) {
    if (active()) bibleRecallState.error = error instanceof Error ? error.message : 'Recall failed';
  } finally {
    if (active()) bibleRecallState.pending = false;
  }
}
