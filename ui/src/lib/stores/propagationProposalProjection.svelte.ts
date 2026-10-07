import {
  acceptPropagationProposal,
  requestScriptImpactProposal,
  requestScriptFactProposal,
  createPropagationProposal,
  rejectPropagationProposal,
  updatePropagationProposal,
} from '$lib/commandApi.js';
import { getPropagationProposalListProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';
import type {
  AcceptPropagationProposalCommand,
  CreatePropagationProposalCommand,
  PropagationProposalCommandResponse,
  PropagationProposalListProjection,
  RejectPropagationProposalCommand,
  UpdatePropagationProposalCommand,
  RequestScriptImpactProposalCommand,
} from '$lib/propagationProposalTypes.js';

let sessionEpoch = $state(0);
export function getPropagationProposalSessionEpoch(): number {
  return sessionEpoch;
}

export const propagationProposalProjectionState = $state<{
  projection: ProjectionEnvelope<PropagationProposalListProjection> | null;
  pending: boolean;
  error?: string;
}>({
  projection: null,
  pending: false,
  error: undefined,
});

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function cacheProjection(projection: ProjectionEnvelope<PropagationProposalListProjection>): void {
  if (shouldReplaceProjection(propagationProposalProjectionState.projection, projection)) {
    propagationProposalProjectionState.projection = projection;
  }
}

export function getCachedPropagationProposalListProjection(): ProjectionEnvelope<PropagationProposalListProjection> | null {
  return propagationProposalProjectionState.projection;
}

export async function refreshPropagationProposalListProjection(): Promise<
  ProjectionEnvelope<PropagationProposalListProjection>
> {
  const admitted = sessionEpoch;
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const projection = await getPropagationProposalListProjection();
    if (admitted !== sessionEpoch) throw new Error('The project changed during proposal refresh.');
    cacheProjection(projection);
    return projection;
  } catch (error) {
    if (admitted === sessionEpoch)
      propagationProposalProjectionState.error = errorMessage(
        error,
        'Failed to load propagation proposals',
      );
    throw error;
  } finally {
    if (admitted === sessionEpoch) propagationProposalProjectionState.pending = false;
  }
}

export async function applyCreatePropagationProposalCommand(
  payload: CreatePropagationProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const response = await createPropagationProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to create propagation proposal',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
  }
}

export async function applyRejectPropagationProposalCommand(
  payload: RejectPropagationProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const response = await rejectPropagationProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to reject propagation proposal',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
  }
}

export async function applyUpdatePropagationProposalCommand(
  payload: UpdatePropagationProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const response = await updatePropagationProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to update propagation proposal',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
  }
}

export async function applyAcceptPropagationProposalCommand(
  payload: AcceptPropagationProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const response = await acceptPropagationProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to accept propagation proposal',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
  }
}

export function clearPropagationProposalListProjection(): void {
  sessionEpoch += 1;
  propagationProposalProjectionState.projection = null;
  propagationProposalProjectionState.pending = false;
  propagationProposalProjectionState.error = undefined;
}

export async function applyRequestScriptImpactProposalCommand(
  payload: RequestScriptImpactProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;
  try {
    const response = await requestScriptImpactProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to preview screenplay update',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
  }
}

async function applyScopedFactCommand(
  operation: () => Promise<PropagationProposalCommandResponse>,
): Promise<PropagationProposalCommandResponse> {
  const admitted = sessionEpoch;
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;
  try {
    const response = await operation();
    if (admitted !== sessionEpoch) throw new Error('The project changed during fact review.');
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    if (admitted === sessionEpoch)
      propagationProposalProjectionState.error = errorMessage(error, 'Fact review failed');
    throw error;
  } finally {
    if (admitted === sessionEpoch) propagationProposalProjectionState.pending = false;
  }
}
export function applyRequestScriptFactProposalCommand(
  payload: import('$lib/scriptFactTypes.js').RequestScriptFactProposalCommand,
  commandId?: CommandId,
): Promise<PropagationProposalCommandResponse> {
  return applyScopedFactCommand(() => requestScriptFactProposal(payload, commandId));
}
export function applyScriptFactDecisionCommand(
  proposalId: string,
  accept: boolean,
  commandId: string,
): Promise<PropagationProposalCommandResponse> {
  return applyScopedFactCommand(() =>
    accept
      ? acceptPropagationProposal({ proposal_id: proposalId }, commandId)
      : rejectPropagationProposal({ proposal_id: proposalId }, commandId),
  );
}
