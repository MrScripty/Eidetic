import type {
  BibleGraphNodeCommandResponse,
  BibleGraphRootsCommandResponse,
  CommandEnvelope,
  CreateBibleGraphNodeCommand,
  EnsureCanonicalBibleRootsCommand,
  ObjectFieldCommandResponse,
  SetBibleGraphFieldCommand,
  SetObjectFieldCommand,
} from './types.js';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
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

export function createCommandId(): string {
  return crypto.randomUUID();
}

export function setObjectField(
  payload: SetObjectFieldCommand,
  commandId = createCommandId(),
): Promise<ObjectFieldCommandResponse> {
  const command: CommandEnvelope<SetObjectFieldCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/object-field', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createBibleGraphNode(
  payload: CreateBibleGraphNodeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<CreateBibleGraphNodeCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/bible-graph/node', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setBibleGraphField(
  payload: SetBibleGraphFieldCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<SetBibleGraphFieldCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/bible-graph/field', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function ensureCanonicalBibleRoots(
  commandId = createCommandId(),
): Promise<BibleGraphRootsCommandResponse> {
  const command: CommandEnvelope<EnsureCanonicalBibleRootsCommand> = {
    id: commandId,
    payload: {},
  };

  return request('/commands/bible-graph/canonical-roots', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}
