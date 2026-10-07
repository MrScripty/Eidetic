import * as production from '../lib/commandApi.js';
import { getScriptDocumentProjection } from '../lib/projectionApi.js';
import type { RequestScriptFactProposalCommand } from '../lib/scriptFactTypes.js';
import type {
  AcceptPropagationProposalCommand,
  RejectPropagationProposalCommand,
} from '../lib/propagationProposalTypes.js';
export * from '../lib/commandApi.js';
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
});
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
          (c) => c.input.kind === 'bible_field' || c.input.kind === 'bible_edge',
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
