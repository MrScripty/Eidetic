import { getChangeReviewProjection } from '$lib/projectionApi.js';
import type {
  ChangeReviewProjection,
  ChangeReviewProjectionEnvelope,
} from '$lib/changeReviewTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export const changeReviewProjectionState = $state<{
  projection: ProjectionEnvelope<ChangeReviewProjection> | null;
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

export function getCachedChangeReviewProjection(): ChangeReviewProjectionEnvelope | null {
  return changeReviewProjectionState.projection;
}

function replaceChangeReviewProjectionIfFresh(projection: ChangeReviewProjectionEnvelope): void {
  if (shouldReplaceProjection(changeReviewProjectionState.projection, projection)) {
    changeReviewProjectionState.projection = projection;
  }
}

export async function refreshChangeReviewProjection(): Promise<ChangeReviewProjectionEnvelope> {
  changeReviewProjectionState.pending = true;
  changeReviewProjectionState.error = undefined;

  try {
    const projection = await getChangeReviewProjection();
    replaceChangeReviewProjectionIfFresh(projection);
    return projection;
  } catch (error) {
    changeReviewProjectionState.error = errorMessage(
      error,
      'Failed to load change review projection',
    );
    throw error;
  } finally {
    changeReviewProjectionState.pending = false;
  }
}

export function clearChangeReviewProjection(): void {
  changeReviewProjectionState.projection = null;
  changeReviewProjectionState.pending = false;
  changeReviewProjectionState.error = undefined;
}
