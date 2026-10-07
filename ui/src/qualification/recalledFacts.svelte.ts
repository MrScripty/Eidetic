import * as production from '../lib/commandApi.js';
import type { RequestScriptImpactProposalCommand } from '$lib/propagationProposalTypes.js';
import { bibleRecallState } from '../lib/stores/bibleRecallProjection.svelte.js';

export * from '../lib/commandApi.js';

/** Explicit qualifier controls; canonical writes and replay use unchanged public commands. */
export const selectedQA = $state({
  capturedRequest: '',
  busy: false,
  mutations: 0,
  replay: '',
  error: '',
});

export function requestScriptImpactProposal(
  payload: RequestScriptImpactProposalCommand,
  commandId?: string,
) {
  if (payload.recall_selection?.facts.length) selectedQA.capturedRequest = JSON.stringify(payload);
  return production.requestScriptImpactProposal(payload, commandId);
}

function message(error: unknown): string {
  return error instanceof Error
    ? error.message
    : typeof error === 'string'
      ? error
      : JSON.stringify(error);
}

export async function mutateSelectedFact(): Promise<void> {
  selectedQA.busy = true;
  selectedQA.error = '';
  try {
    await production.setBibleGraphField({
      node_id: 'qualification.keeper',
      part_id: 'part.default.qualification.keeper.profile',
      part_key: 'profile',
      part_name: 'Profile',
      part_sort_order: 10,
      field_id: 'qualification.keeper.tagline',
      field_key: 'tagline',
      value: {
        type: 'text',
        value:
          selectedQA.mutations === 0
            ? 'SELECTED copper roof key — 雨.'
            : 'SELECTED final roof key — 雨.',
      },
      field_sort_order: 20,
    });
    selectedQA.mutations += 1;
  } catch (error) {
    selectedQA.error = message(error);
  } finally {
    selectedQA.busy = false;
  }
}

export async function replayStaleSelection(): Promise<void> {
  selectedQA.busy = true;
  selectedQA.error = '';
  selectedQA.replay = '';
  try {
    const request = JSON.parse(selectedQA.capturedRequest) as RequestScriptImpactProposalCommand;
    request.proposal_id = `script.review.qa-stale.${crypto.randomUUID()}`;
    await production.requestScriptImpactProposal(request);
    selectedQA.replay = 'UNEXPECTED acceptance of stale selection';
  } catch (error) {
    selectedQA.replay = `Actual backend refusal: ${message(error)}`;
  } finally {
    selectedQA.busy = false;
  }
}

export function selectedReceipt(): string {
  return JSON.stringify({
    fixture:
      'qualifier-only public-command mutation and exact stale-selector replay; synthetic provider replies',
    ...selectedQA,
    recallVersion: bibleRecallState.projection?.version ?? null,
    recallInvalidated: bibleRecallState.invalidated,
  });
}
