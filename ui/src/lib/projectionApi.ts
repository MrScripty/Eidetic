import type {
  BibleGraphNodeId,
  BibleGraphNodeListProjection,
  BibleNodeDetailProjection,
  ObjectFieldProjection,
  ObjectKind,
  ProjectionEnvelope,
  ScriptDocumentId,
  ScriptDocumentProjection,
} from './types.js';
import type { BibleGraphSchemaListProjection } from './bibleGraphSchemaTypes.js';

const BASE = '/api';

export interface ObjectFieldProjectionKey {
  object_kind: ObjectKind;
  object_id: string;
}

export interface BibleGraphNodeProjectionKey {
  node_id: BibleGraphNodeId;
}

export interface ScriptDocumentProjectionKey {
  document_id: ScriptDocumentId;
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

export function getBibleGraphNodeProjection({
  node_id,
}: BibleGraphNodeProjectionKey): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  const params = new URLSearchParams({ node_id });
  return getJson(`/projections/bible-graph/node?${params.toString()}`);
}

export function getBibleGraphNodeListProjection(): Promise<
  ProjectionEnvelope<BibleGraphNodeListProjection>
> {
  return getJson('/projections/bible-graph/nodes');
}

export function getBibleGraphSchemaListProjection(): Promise<
  ProjectionEnvelope<BibleGraphSchemaListProjection>
> {
  return getJson('/projections/bible-graph/schemas');
}

export function getScriptDocumentProjection({
  document_id,
}: ScriptDocumentProjectionKey): Promise<ProjectionEnvelope<ScriptDocumentProjection>> {
  const params = new URLSearchParams({ document_id });
  return getJson(`/projections/script/document?${params.toString()}`);
}
