import { getBibleRenderGraphProjection } from '$lib/projectionApi.js';
import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export const bibleRenderGraphProjectionState = $state<{
  projection: ProjectionEnvelope<BibleRenderGraphProjection> | null;
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

export function getCachedBibleRenderGraphProjection(): ProjectionEnvelope<BibleRenderGraphProjection> | null {
  return bibleRenderGraphProjectionState.projection;
}

function replaceBibleRenderGraphProjectionIfFresh(
  projection: ProjectionEnvelope<BibleRenderGraphProjection>,
): void {
  if (shouldReplaceProjection(bibleRenderGraphProjectionState.projection, projection)) {
    bibleRenderGraphProjectionState.projection = projection;
  }
}

export async function refreshBibleRenderGraphProjection(): Promise<
  ProjectionEnvelope<BibleRenderGraphProjection>
> {
  bibleRenderGraphProjectionState.pending = true;
  bibleRenderGraphProjectionState.error = undefined;

  try {
    const projection = await getBibleRenderGraphProjection();
    replaceBibleRenderGraphProjectionIfFresh(projection);
    return projection;
  } catch (error) {
    bibleRenderGraphProjectionState.error = errorMessage(
      error,
      'Failed to load bible render graph projection',
    );
    throw error;
  } finally {
    bibleRenderGraphProjectionState.pending = false;
  }
}

export function clearBibleRenderGraphProjection(): void {
  bibleRenderGraphProjectionState.projection = null;
  bibleRenderGraphProjectionState.pending = false;
  bibleRenderGraphProjectionState.error = undefined;
}
