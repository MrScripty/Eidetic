import {
  acceptPropagationProposal,
  createPropagationProposal,
  rejectPropagationProposal,
} from '$lib/commandApi.js';
import { getPropagationProposalListProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '$lib/projectionTypes.js';
import type {
  AcceptPropagationProposalCommand,
  CreatePropagationProposalCommand,
  PropagationProposalCommandResponse,
  PropagationProposalListProjection,
  RejectPropagationProposalCommand,
} from '$lib/propagationProposalTypes.js';

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
  propagationProposalProjectionState.projection = projection;
}

export function getCachedPropagationProposalListProjection(): ProjectionEnvelope<PropagationProposalListProjection> | null {
  return propagationProposalProjectionState.projection;
}

export async function refreshPropagationProposalListProjection(): Promise<
  ProjectionEnvelope<PropagationProposalListProjection>
> {
  propagationProposalProjectionState.pending = true;
  propagationProposalProjectionState.error = undefined;

  try {
    const projection = await getPropagationProposalListProjection();
    cacheProjection(projection);
    return projection;
  } catch (error) {
    propagationProposalProjectionState.error = errorMessage(
      error,
      'Failed to load propagation proposals',
    );
    throw error;
  } finally {
    propagationProposalProjectionState.pending = false;
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
  propagationProposalProjectionState.projection = null;
  propagationProposalProjectionState.pending = false;
  propagationProposalProjectionState.error = undefined;
}
