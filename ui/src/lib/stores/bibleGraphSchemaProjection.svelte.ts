import { getBibleGraphSchemaListProjection } from '$lib/projectionApi.js';
import type { BibleGraphSchemaListProjection } from '$lib/bibleGraphSchemaTypes.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';

export const bibleGraphSchemaProjectionState = $state<{
  projection: ProjectionEnvelope<BibleGraphSchemaListProjection> | null;
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

export function getCachedBibleGraphSchemaListProjection(): ProjectionEnvelope<BibleGraphSchemaListProjection> | null {
  return bibleGraphSchemaProjectionState.projection;
}

export async function refreshBibleGraphSchemaListProjection(): Promise<
  ProjectionEnvelope<BibleGraphSchemaListProjection>
> {
  bibleGraphSchemaProjectionState.pending = true;
  bibleGraphSchemaProjectionState.error = undefined;

  try {
    const projection = await getBibleGraphSchemaListProjection();
    bibleGraphSchemaProjectionState.projection = projection;
    return projection;
  } catch (error) {
    bibleGraphSchemaProjectionState.error = errorMessage(
      error,
      'Failed to load bible graph schemas',
    );
    throw error;
  } finally {
    bibleGraphSchemaProjectionState.pending = false;
  }
}

export function clearBibleGraphSchemaListProjection(): void {
  bibleGraphSchemaProjectionState.projection = null;
  bibleGraphSchemaProjectionState.pending = false;
  bibleGraphSchemaProjectionState.error = undefined;
}
