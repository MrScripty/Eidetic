import {
  createScriptBlock,
  editScriptBlock,
  setScriptBlock,
  setScriptLock,
} from '$lib/commandApi.js';
import { getScriptDocumentProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';
import type {
  ScriptDocumentId,
  ScriptDocumentProjection,
  ScriptDocumentCommandResponse,
  SetScriptBlockCommand,
  SetScriptLockCommand,
  EditScriptBlockCommand,
  CreateScriptBlockCommand,
} from '../scriptTypes.js';

export interface ScriptDocumentProjectionKey {
  document_id: ScriptDocumentId;
}

export const MAIN_SCRIPT_DOCUMENT_ID = 'script.document.main';

export const scriptDocumentProjectionState = $state<{
  projections: Record<string, ProjectionEnvelope<ScriptDocumentProjection>>;
  pending: Record<string, boolean>;
  errors: Record<string, string | undefined>;
  contextRevision: number;
}>({
  projections: {},
  pending: {},
  errors: {},
  contextRevision: 0,
});

function projectionKey({ document_id }: ScriptDocumentProjectionKey): string {
  return encodeURIComponent(document_id);
}

interface CacheLifetime {
  activeRequests: number;
  latestRequestId: number;
}

// Object identity separates lifetimes even when the same document is reopened.
const cacheLifetimes = Object.create(null) as Record<string, CacheLifetime | undefined>;

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function cacheProjection(
  cacheKey: string,
  projection: ProjectionEnvelope<ScriptDocumentProjection>,
): void {
  if (
    shouldReplaceProjection(scriptDocumentProjectionState.projections[cacheKey] ?? null, projection)
  ) {
    scriptDocumentProjectionState.projections[cacheKey] = projection;
  }
}

export function getCachedScriptDocumentProjection(
  key: ScriptDocumentProjectionKey,
): ProjectionEnvelope<ScriptDocumentProjection> | undefined {
  return scriptDocumentProjectionState.projections[projectionKey(key)];
}

export function isScriptDocumentProjectionPending(key: ScriptDocumentProjectionKey): boolean {
  return scriptDocumentProjectionState.pending[projectionKey(key)] === true;
}

export function getScriptDocumentProjectionError(
  key: ScriptDocumentProjectionKey,
): string | undefined {
  return scriptDocumentProjectionState.errors[projectionKey(key)];
}

async function runScriptProjectionRequest<T>(
  cacheKey: string,
  request: () => Promise<T>,
  projectionOf: (result: T) => ProjectionEnvelope<ScriptDocumentProjection>,
  failureMessage: string,
): Promise<T> {
  let lifetime = cacheLifetimes[cacheKey];
  if (!lifetime) {
    lifetime = { activeRequests: 0, latestRequestId: 0 };
    cacheLifetimes[cacheKey] = lifetime;
  }
  const requestId = ++lifetime.latestRequestId;
  lifetime.activeRequests += 1;
  scriptDocumentProjectionState.pending[cacheKey] = true;
  scriptDocumentProjectionState.errors[cacheKey] = undefined;

  try {
    const result = await request();
    if (cacheLifetimes[cacheKey] === lifetime) {
      cacheProjection(cacheKey, projectionOf(result));
    }
    return result;
  } catch (error) {
    if (cacheLifetimes[cacheKey] === lifetime && requestId === lifetime.latestRequestId) {
      scriptDocumentProjectionState.errors[cacheKey] = errorMessage(error, failureMessage);
    }
    throw error;
  } finally {
    if (cacheLifetimes[cacheKey] === lifetime) {
      lifetime.activeRequests -= 1;
      scriptDocumentProjectionState.pending[cacheKey] = lifetime.activeRequests > 0;
    }
  }
}

export async function refreshScriptDocumentProjection(
  key: ScriptDocumentProjectionKey,
): Promise<ProjectionEnvelope<ScriptDocumentProjection>> {
  return runScriptProjectionRequest(
    projectionKey(key),
    () => getScriptDocumentProjection(key),
    (result) => result,
    'Failed to load script document',
  );
}

export async function applyScriptBlockCommand(
  payload: SetScriptBlockCommand,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  return runScriptProjectionRequest(
    projectionKey({ document_id: payload.document_id }),
    () => setScriptBlock(payload, commandId),
    (result) => result.projection,
    'Failed to apply script block command',
  );
}

export async function applyScriptLockCommand(
  payload: SetScriptLockCommand,
  documentId: ScriptDocumentId,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  return runScriptProjectionRequest(
    projectionKey({ document_id: documentId }),
    () => setScriptLock(payload, commandId),
    (result) => result.projection,
    'Failed to apply script lock command',
  );
}

export function clearScriptDocumentProjection(key: ScriptDocumentProjectionKey): void {
  const cacheKey = projectionKey(key);
  delete cacheLifetimes[cacheKey];
  delete scriptDocumentProjectionState.projections[cacheKey];
  delete scriptDocumentProjectionState.pending[cacheKey];
  delete scriptDocumentProjectionState.errors[cacheKey];
}

export function invalidateScriptContext(): void {
  scriptDocumentProjectionState.contextRevision += 1;
}

export async function applyScriptBlockEditCommand(
  payload: EditScriptBlockCommand,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  const response = await runScriptProjectionRequest(
    projectionKey({ document_id: payload.document_id }),
    () => editScriptBlock(payload, commandId),
    (result) => result.projection,
    'Failed to save script block',
  );
  invalidateScriptContext();
  return response;
}

export async function applyScriptBlockCreationCommand(
  payload: CreateScriptBlockCommand,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  const response = await runScriptProjectionRequest(
    projectionKey({ document_id: payload.document_id }),
    () => createScriptBlock(payload, commandId),
    (result) => result.projection,
    'Failed to write screenplay block',
  );
  invalidateScriptContext();
  return response;
}
