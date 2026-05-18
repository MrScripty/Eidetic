import { setScriptBlock, setScriptLock } from '$lib/commandApi.js';
import { getScriptDocumentProjection } from '$lib/projectionApi.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';
import type {
  ScriptDocumentId,
  ScriptDocumentProjection,
  ScriptDocumentCommandResponse,
  SetScriptBlockCommand,
  SetScriptLockCommand,
} from '../types.js';

export interface ScriptDocumentProjectionKey {
  document_id: ScriptDocumentId;
}

export const MAIN_SCRIPT_DOCUMENT_ID = 'script.document.main';

export const scriptDocumentProjectionState = $state<{
  projections: Record<string, ProjectionEnvelope<ScriptDocumentProjection>>;
  pending: Record<string, boolean>;
  errors: Record<string, string | undefined>;
}>({
  projections: {},
  pending: {},
  errors: {},
});

function projectionKey({ document_id }: ScriptDocumentProjectionKey): string {
  return encodeURIComponent(document_id);
}

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
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

export async function refreshScriptDocumentProjection(
  key: ScriptDocumentProjectionKey,
): Promise<ProjectionEnvelope<ScriptDocumentProjection>> {
  const cacheKey = projectionKey(key);
  scriptDocumentProjectionState.pending[cacheKey] = true;
  scriptDocumentProjectionState.errors[cacheKey] = undefined;

  try {
    const projection = await getScriptDocumentProjection(key);
    scriptDocumentProjectionState.projections[cacheKey] = projection;
    return projection;
  } catch (error) {
    scriptDocumentProjectionState.errors[cacheKey] = errorMessage(
      error,
      'Failed to load script document',
    );
    throw error;
  } finally {
    scriptDocumentProjectionState.pending[cacheKey] = false;
  }
}

export async function applyScriptBlockCommand(
  payload: SetScriptBlockCommand,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  const cacheKey = projectionKey({
    document_id: payload.document_id,
  });
  scriptDocumentProjectionState.pending[cacheKey] = true;
  scriptDocumentProjectionState.errors[cacheKey] = undefined;

  try {
    const response = await setScriptBlock(payload, commandId);
    scriptDocumentProjectionState.projections[cacheKey] = response.projection;
    return response;
  } catch (error) {
    scriptDocumentProjectionState.errors[cacheKey] = errorMessage(
      error,
      'Failed to apply script block command',
    );
    throw error;
  } finally {
    scriptDocumentProjectionState.pending[cacheKey] = false;
  }
}

export async function applyScriptLockCommand(
  payload: SetScriptLockCommand,
  documentId: ScriptDocumentId,
  commandId?: CommandId,
): Promise<ScriptDocumentCommandResponse> {
  const cacheKey = projectionKey({
    document_id: documentId,
  });
  scriptDocumentProjectionState.pending[cacheKey] = true;
  scriptDocumentProjectionState.errors[cacheKey] = undefined;

  try {
    const response = await setScriptLock(payload, commandId);
    scriptDocumentProjectionState.projections[cacheKey] = response.projection;
    return response;
  } catch (error) {
    scriptDocumentProjectionState.errors[cacheKey] = errorMessage(
      error,
      'Failed to apply script lock command',
    );
    throw error;
  } finally {
    scriptDocumentProjectionState.pending[cacheKey] = false;
  }
}

export function clearScriptDocumentProjection(key: ScriptDocumentProjectionKey): void {
  const cacheKey = projectionKey(key);
  delete scriptDocumentProjectionState.projections[cacheKey];
  delete scriptDocumentProjectionState.pending[cacheKey];
  delete scriptDocumentProjectionState.errors[cacheKey];
}
