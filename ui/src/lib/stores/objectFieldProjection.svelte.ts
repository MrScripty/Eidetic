import { setObjectField } from '$lib/commandApi.js';
import { getObjectFieldProjection } from '$lib/projectionApi.js';
import type {
  CommandId,
  ObjectFieldCommandResponse,
  ObjectFieldProjection,
  ObjectKind,
  ProjectionEnvelope,
  SetObjectFieldCommand,
} from '../types.js';

export interface ObjectFieldProjectionKey {
  object_kind: ObjectKind;
  object_id: string;
}

export const objectFieldProjectionState = $state<{
  projections: Record<string, ProjectionEnvelope<ObjectFieldProjection>>;
  pending: Record<string, boolean>;
  errors: Record<string, string | undefined>;
}>({
  projections: {},
  pending: {},
  errors: {},
});

function projectionKey({ object_kind, object_id }: ObjectFieldProjectionKey): string {
  return `${object_kind}:${encodeURIComponent(object_id)}`;
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedObjectFieldProjection(
  key: ObjectFieldProjectionKey,
): ProjectionEnvelope<ObjectFieldProjection> | undefined {
  return objectFieldProjectionState.projections[projectionKey(key)];
}

export function isObjectFieldProjectionPending(key: ObjectFieldProjectionKey): boolean {
  return objectFieldProjectionState.pending[projectionKey(key)] === true;
}

export function getObjectFieldProjectionError(key: ObjectFieldProjectionKey): string | undefined {
  return objectFieldProjectionState.errors[projectionKey(key)];
}

export async function refreshObjectFieldProjection(
  key: ObjectFieldProjectionKey,
): Promise<ProjectionEnvelope<ObjectFieldProjection>> {
  const cacheKey = projectionKey(key);
  objectFieldProjectionState.pending[cacheKey] = true;
  objectFieldProjectionState.errors[cacheKey] = undefined;

  try {
    const projection = await getObjectFieldProjection(key);
    objectFieldProjectionState.projections[cacheKey] = projection;
    return projection;
  } catch (error) {
    objectFieldProjectionState.errors[cacheKey] = errorMessage(error, 'Failed to load projection');
    throw error;
  } finally {
    objectFieldProjectionState.pending[cacheKey] = false;
  }
}

export async function applyObjectFieldCommand(
  payload: SetObjectFieldCommand,
  commandId?: CommandId,
): Promise<ObjectFieldCommandResponse> {
  const cacheKey = projectionKey({
    object_kind: payload.object_kind,
    object_id: payload.object_id,
  });
  objectFieldProjectionState.pending[cacheKey] = true;
  objectFieldProjectionState.errors[cacheKey] = undefined;

  try {
    const response = await setObjectField(payload, commandId);
    objectFieldProjectionState.projections[cacheKey] = response.projection;
    return response;
  } catch (error) {
    objectFieldProjectionState.errors[cacheKey] = errorMessage(error, 'Failed to apply command');
    throw error;
  } finally {
    objectFieldProjectionState.pending[cacheKey] = false;
  }
}

export function clearObjectFieldProjection(key: ObjectFieldProjectionKey): void {
  const cacheKey = projectionKey(key);
  delete objectFieldProjectionState.projections[cacheKey];
  delete objectFieldProjectionState.pending[cacheKey];
  delete objectFieldProjectionState.errors[cacheKey];
}
