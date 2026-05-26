import {
  acceptAffectProposal,
  createAffectProposal,
  rejectAffectProposal,
} from '$lib/commandApi.js';
import { getAffectProposalListProjection } from '$lib/projectionApi.js';
import type {
  AcceptAffectProposalCommand,
  AffectProposalCommandResponse,
  AffectProposalListProjection,
  CreateAffectProposalCommand,
  RejectAffectProposalCommand,
} from '$lib/affectTypes.js';
import type { CommandId, ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export const affectProposalProjectionState = $state<{
  projection: ProjectionEnvelope<AffectProposalListProjection> | null;
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

function cacheProjection(projection: ProjectionEnvelope<AffectProposalListProjection>): void {
  if (shouldReplaceProjection(affectProposalProjectionState.projection, projection)) {
    affectProposalProjectionState.projection = projection;
  }
}

export function getCachedAffectProposalListProjection(): ProjectionEnvelope<AffectProposalListProjection> | null {
  return affectProposalProjectionState.projection;
}

export async function refreshAffectProposalListProjection(): Promise<
  ProjectionEnvelope<AffectProposalListProjection>
> {
  affectProposalProjectionState.pending = true;
  affectProposalProjectionState.error = undefined;

  try {
    const projection = await getAffectProposalListProjection();
    cacheProjection(projection);
    return projection;
  } catch (error) {
    affectProposalProjectionState.error = errorMessage(error, 'Failed to load affect proposals');
    throw error;
  } finally {
    affectProposalProjectionState.pending = false;
  }
}

export async function applyCreateAffectProposalCommand(
  payload: CreateAffectProposalCommand,
  commandId?: CommandId,
): Promise<AffectProposalCommandResponse> {
  affectProposalProjectionState.pending = true;
  affectProposalProjectionState.error = undefined;

  try {
    const projection = await createAffectProposal(payload, commandId);
    cacheProjection(projection);
    return projection;
  } catch (error) {
    affectProposalProjectionState.error = errorMessage(error, 'Failed to create affect proposal');
    throw error;
  } finally {
    affectProposalProjectionState.pending = false;
  }
}

export async function applyRejectAffectProposalCommand(
  payload: RejectAffectProposalCommand,
  commandId?: CommandId,
): Promise<AffectProposalCommandResponse> {
  affectProposalProjectionState.pending = true;
  affectProposalProjectionState.error = undefined;

  try {
    const projection = await rejectAffectProposal(payload, commandId);
    cacheProjection(projection);
    return projection;
  } catch (error) {
    affectProposalProjectionState.error = errorMessage(error, 'Failed to reject affect proposal');
    throw error;
  } finally {
    affectProposalProjectionState.pending = false;
  }
}

export async function applyAcceptAffectProposalCommand(
  payload: AcceptAffectProposalCommand,
  commandId?: CommandId,
): Promise<AffectProposalCommandResponse> {
  affectProposalProjectionState.pending = true;
  affectProposalProjectionState.error = undefined;

  try {
    const projection = await acceptAffectProposal(payload, commandId);
    cacheProjection(projection);
    return projection;
  } catch (error) {
    affectProposalProjectionState.error = errorMessage(error, 'Failed to accept affect proposal');
    throw error;
  } finally {
    affectProposalProjectionState.pending = false;
  }
}

export function clearAffectProposalListProjection(): void {
  affectProposalProjectionState.projection = null;
  affectProposalProjectionState.pending = false;
  affectProposalProjectionState.error = undefined;
}
