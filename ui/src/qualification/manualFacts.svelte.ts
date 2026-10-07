import { untrack } from 'svelte';
import * as production from '../lib/commandApi.js';
import { getScriptDocumentProjection } from '../lib/projectionApi.js';
import * as projections from '../lib/projectionApi.js';
import { refreshBibleGraphNodeProjection } from '../lib/stores/bibleGraphNodeDetailProjection.svelte.js';
import type { SetBibleGraphFieldCommand } from '../lib/bibleGraphTypes.js';
import type { RequestScriptFactProposalCommand } from '../lib/scriptFactTypes.js';
import type {
  AcceptPropagationProposalCommand,
  RejectPropagationProposalCommand,
} from '../lib/propagationProposalTypes.js';
export * from '../lib/commandApi.js';
export * from '../lib/projectionApi.js';
export const factQA = $state({
  busy: false,
  error: '',
  request: '',
  requestId: '',
  decision: '',
  decisionId: '',
  accept: false,
  replay: '',
  edgeMutations: 0,
  impacts: '',
  savePauseArmed: false,
  saveAcknowledgementHeld: false,
  detailMode: 'normal',
  detailReadsHeld: 0,
  detailFailures: 0,
  newerFactWrites: 0,
});
let releaseSave: (() => void) | undefined;
const releaseReads: Array<() => void> = [];
export async function setBibleGraphField(payload: SetBibleGraphFieldCommand, commandId?: string) {
  const response = await production.setBibleGraphField(payload, commandId);
  if (factQA.savePauseArmed && payload.field_id === 'qualification.mara.tagline') {
    factQA.savePauseArmed = false;
    factQA.saveAcknowledgementHeld = true;
    await new Promise<void>((resolve) => {
      releaseSave = resolve;
    });
    factQA.saveAcknowledgementHeld = false;
  }
  return response;
}
export async function getBibleGraphNodeProjection(key: { node_id: string }) {
  // Fault instrumentation must not become an input of the production owner effect.
  const mode = untrack(() => factQA.detailMode);
  if (key.node_id === 'qualification.mara') {
    if (mode === 'fail') {
      untrack(() => factQA.detailFailures++);
      throw new Error('SYNTHETIC QA transport: Bible detail unavailable.');
    }
    if (mode === 'hold') {
      untrack(() => factQA.detailReadsHeld++);
      await new Promise<void>((resolve) => {
        releaseReads.push(resolve);
      });
    }
  }
  return projections.getBibleGraphNodeProjection(key);
}
export function armSavePause() {
  factQA.savePauseArmed = true;
}
export function releaseSaveAcknowledgement() {
  releaseSave?.();
  releaseSave = undefined;
}
export function newerFact() {
  return action(async () => {
    await production.setBibleGraphField({
      node_id: 'qualification.mara',
      part_id: 'part.default.qualification.mara.profile',
      part_key: 'profile',
      part_name: 'Profile',
      part_sort_order: 10,
      field_id: 'qualification.mara.tagline',
      field_key: 'tagline',
      field_sort_order: 20,
      value: { type: 'text', value: 'Green saved fact — 雨.' },
    });
    factQA.newerFactWrites++;
    return { publicNewerFact: true };
  });
}
export function failDetailRefresh() {
  return action(async () => {
    factQA.detailMode = 'fail';
    try {
      await refreshBibleGraphNodeProjection({ node_id: 'qualification.mara' });
    } catch {
      return { labelledTransportFailure: true };
    }
    throw new Error('Expected labelled detail failure was not exercised');
  });
}
export function armInitialDetailFailure() {
  // Arm only: the ordinary entity selection must cause the first failed read.
  factQA.detailMode = 'fail';
}
export function holdDetailRecovery() {
  factQA.detailMode = 'hold';
}
export function releaseDetailRecovery() {
  factQA.detailMode = 'normal';
  for (const resolve of releaseReads.splice(0)) resolve();
}
export function requestScriptFactProposal(
  payload: RequestScriptFactProposalCommand,
  commandId?: string,
) {
  factQA.request = JSON.stringify(payload);
  factQA.requestId = commandId ?? '';
  return production.requestScriptFactProposal(payload, commandId);
}
export function acceptPropagationProposal(
  payload: AcceptPropagationProposalCommand,
  commandId?: string,
) {
  factQA.decision = JSON.stringify(payload);
  factQA.decisionId = commandId ?? '';
  factQA.accept = true;
  return production.acceptPropagationProposal(payload, commandId);
}
export function rejectPropagationProposal(
  payload: RejectPropagationProposalCommand,
  commandId?: string,
) {
  factQA.decision = JSON.stringify(payload);
  factQA.decisionId = commandId ?? '';
  factQA.accept = false;
  return production.rejectPropagationProposal(payload, commandId);
}
async function action(operation: () => Promise<unknown>) {
  factQA.busy = true;
  factQA.error = '';
  factQA.replay = '';
  try {
    const response = await operation();
    factQA.replay = JSON.stringify(response);
  } catch (error) {
    factQA.error = String(error);
  } finally {
    factQA.busy = false;
  }
}
export function replayRequest() {
  return action(async () => {
    const response = await production.requestScriptFactProposal(
      JSON.parse(factQA.request),
      factQA.requestId,
    );
    return {
      outcome: response.outcome,
      proposalIds: response.projection.payload.proposals.map((p) => p.id),
    };
  });
}
export function replayDecision() {
  return action(async () => {
    const response = await (
      factQA.accept ? production.acceptPropagationProposal : production.rejectPropagationProposal
    )(JSON.parse(factQA.decision), factQA.decisionId);
    return { outcome: response.outcome };
  });
}
export function relationshipABA() {
  return action(async () => {
    for (const label of ['TEMPORARY changed association', 'Mara trusts Eli']) {
      await production.setBibleGraphEdge({
        edge_id: 'qualification.mara.eli',
        from_node_id: 'qualification.mara',
        to_node_id: 'qualification.eli',
        edge_kind: 'references',
        label,
        directed: true,
        sort_order: 0,
      });
    }
    factQA.edgeMutations += 2;
    return { publicRelationshipABA: true };
  });
}
export function readImpacts() {
  return action(async () => {
    const projection = await getScriptDocumentProjection({ document_id: 'script.document.main' });
    factQA.impacts = JSON.stringify(
      projection.payload.segments.map((s) => ({
        source: s.segment.source_node_id,
        segment: s.segment.id,
        needsReview: s.impact?.needs_review,
        causes: s.impact?.causes.filter(
          (c) =>
            c.input.kind === 'bible_field' ||
            c.input.kind === 'bible_edge' ||
            c.dependency_id.endsWith('.timeline_notes'),
        ),
      })),
    );
    return { readOnlyCanonicalImpacts: true };
  });
}
export function receipt() {
  return JSON.stringify({
    fixture: 'qualifier-only public commands and read-only impacts; synthetic replies',
    ...factQA,
  });
}
