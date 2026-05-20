import {
  acceptBibleReferenceProposal,
  createBibleReferenceProposal,
  rejectBibleReferenceProposal,
} from '$lib/commandApi.js';
import { getBibleReferenceProposalListProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';
import type {
  AcceptBibleReferenceProposalCommand,
  BibleReferenceProposalCommandResponse,
  BibleReferenceProposalListProjection,
  CreateBibleReferenceProposalCommand,
  RejectBibleReferenceProposalCommand,
} from '$lib/semanticProposalTypes.js';

export const semanticProposalProjectionState = $state<{
  projection: ProjectionEnvelope<BibleReferenceProposalListProjection> | null;
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

function cacheProjection(
  projection: ProjectionEnvelope<BibleReferenceProposalListProjection>,
): void {
  if (shouldReplaceProjection(semanticProposalProjectionState.projection, projection)) {
    semanticProposalProjectionState.projection = projection;
  }
}

export function getCachedBibleReferenceProposalListProjection(): ProjectionEnvelope<BibleReferenceProposalListProjection> | null {
  return semanticProposalProjectionState.projection;
}

export async function refreshBibleReferenceProposalListProjection(): Promise<
  ProjectionEnvelope<BibleReferenceProposalListProjection>
> {
  semanticProposalProjectionState.pending = true;
  semanticProposalProjectionState.error = undefined;

  try {
    const projection = await getBibleReferenceProposalListProjection();
    cacheProjection(projection);
    return projection;
  } catch (error) {
    semanticProposalProjectionState.error = errorMessage(
      error,
      'Failed to load bible reference proposals',
    );
    throw error;
  } finally {
    semanticProposalProjectionState.pending = false;
  }
}

export async function applyCreateBibleReferenceProposalCommand(
  payload: CreateBibleReferenceProposalCommand,
  commandId?: CommandId,
): Promise<BibleReferenceProposalCommandResponse> {
  semanticProposalProjectionState.pending = true;
  semanticProposalProjectionState.error = undefined;

  try {
    const response = await createBibleReferenceProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    semanticProposalProjectionState.error = errorMessage(
      error,
      'Failed to create bible reference proposal',
    );
    throw error;
  } finally {
    semanticProposalProjectionState.pending = false;
  }
}

export async function applyRejectBibleReferenceProposalCommand(
  payload: RejectBibleReferenceProposalCommand,
  commandId?: CommandId,
): Promise<BibleReferenceProposalCommandResponse> {
  semanticProposalProjectionState.pending = true;
  semanticProposalProjectionState.error = undefined;

  try {
    const response = await rejectBibleReferenceProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    semanticProposalProjectionState.error = errorMessage(
      error,
      'Failed to reject bible reference proposal',
    );
    throw error;
  } finally {
    semanticProposalProjectionState.pending = false;
  }
}

export async function applyAcceptBibleReferenceProposalCommand(
  payload: AcceptBibleReferenceProposalCommand,
  commandId?: CommandId,
): Promise<BibleReferenceProposalCommandResponse> {
  semanticProposalProjectionState.pending = true;
  semanticProposalProjectionState.error = undefined;

  try {
    const response = await acceptBibleReferenceProposal(payload, commandId);
    cacheProjection(response.projection);
    return response;
  } catch (error) {
    semanticProposalProjectionState.error = errorMessage(
      error,
      'Failed to accept bible reference proposal',
    );
    throw error;
  } finally {
    semanticProposalProjectionState.pending = false;
  }
}

export function clearBibleReferenceProposalListProjection(): void {
  semanticProposalProjectionState.projection = null;
  semanticProposalProjectionState.pending = false;
  semanticProposalProjectionState.error = undefined;
}
