import type { ObjectFieldProjection, ObjectKind, ProjectionEnvelope } from './types.js';

const BASE = '/api';

export interface ObjectFieldProjectionKey {
  object_kind: ObjectKind;
  object_id: string;
}

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'GET',
    headers: { Accept: 'application/json' },
  });
  const body = await res.json().catch(() => null);
  if (!res.ok) {
    throw new Error((body as Record<string, string> | null)?.error || `HTTP ${res.status}`);
  }
  if (body && typeof body === 'object' && 'error' in body && typeof body.error === 'string') {
    throw new Error(body.error);
  }
  return body as T;
}

export function getObjectFieldProjection({
  object_kind,
  object_id,
}: ObjectFieldProjectionKey): Promise<ProjectionEnvelope<ObjectFieldProjection>> {
  const params = new URLSearchParams({ object_kind, object_id });
  return getJson(`/projections/object-field?${params.toString()}`);
}
