import type { ProjectionEnvelope } from '$lib/projectionTypes.js';

export function shouldReplaceProjection<TPayload>(
  current: ProjectionEnvelope<TPayload> | null,
  next: ProjectionEnvelope<TPayload>,
): boolean {
  return current === null || next.version >= current.version;
}
