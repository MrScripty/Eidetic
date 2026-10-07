import type { BibleRecallProjection, BibleRecallRequest } from '$lib/bibleRecallTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { getBibleRecallProjection as productionRecall } from '../lib/projectionApi.js';
import { bibleRecallState } from '../lib/stores/bibleRecallProjection.svelte.js';

/** Qualifier-only completion control; every held success is an actual domain read. */
export const lifecycleQA = $state({
  inspectorsVisible: true,
  hosts: 0,
  holdNext: false,
  held: false,
  reads: 0,
  completions: 0,
  lastCompletion: 'none',
  actualReadVersion: 0,
  actualReadQuery: '',
});
let heldCompletion:
  | {
      evidence: ProjectionEnvelope<BibleRecallProjection>;
      resolve: (value: ProjectionEnvelope<BibleRecallProjection>) => void;
      reject: (error: Error) => void;
    }
  | undefined;

export async function getBibleRecallProjection(
  query: BibleRecallRequest,
): Promise<ProjectionEnvelope<BibleRecallProjection>> {
  const held = lifecycleQA.holdNext;
  lifecycleQA.holdNext = false;
  lifecycleQA.reads += 1;
  const evidence = await productionRecall(query);
  if (!held) return evidence;
  if (heldCompletion) throw new Error('QA completion already held');
  return new Promise((resolve, reject) => {
    heldCompletion = { evidence, resolve, reject };
    lifecycleQA.actualReadVersion = evidence.version;
    lifecycleQA.actualReadQuery = JSON.stringify(evidence.payload.request);
    lifecycleQA.held = true;
  });
}

export function releaseHeldCompletion(failure: boolean): void {
  const completion = heldCompletion;
  if (!completion) throw new Error('No actual recall completion is held');
  heldCompletion = undefined;
  lifecycleQA.held = false;
  lifecycleQA.completions += 1;
  lifecycleQA.lastCompletion = failure
    ? 'labelled synthetic completion error'
    : 'actual read success';
  if (failure) completion.reject(new Error('QA labelled synthetic delayed recall error'));
  else completion.resolve(completion.evidence);
}

export function lifecycleReceipt(): string {
  return JSON.stringify({
    fixture: 'qualifier-only native lifecycle controls; actual domain reads; synthetic error',
    ...lifecycleQA,
    productionState: {
      anchor: bibleRecallState.anchor,
      pending: bibleRecallState.pending,
      invalidated: bibleRecallState.invalidated,
      error: bibleRecallState.error,
      projectionVersion: bibleRecallState.projection?.version ?? null,
    },
  });
}
