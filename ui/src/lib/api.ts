import type { AiConfig, AiStatus, DiffusionStatus, ModelListResponse } from './aiTypes.js';
import type { Project, ReferenceDocument, ReferenceType } from './projectTypes.js';
import type {
  ChildPlan,
  NodeContent,
  StoryLevel,
  StoryNode,
  Timeline,
  TimelineGap,
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

// --- Project ---

export function createProject(name: string, template: string): Promise<Project> {
  return request('/project', {
    method: 'POST',
    body: JSON.stringify({ name, template }),
  });
}

export function getProject(): Promise<Project> {
  return request('/project');
}

export function updateProject(updates: { name?: string; premise?: string }): Promise<Project> {
  return request('/project', {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}

// --- Timeline ---

export function getTimeline(): Promise<Timeline> {
  return request('/timeline');
}

// --- Nodes ---

export function getNodeChildren(id: string): Promise<StoryNode[]> {
  return request(`/timeline/nodes/${id}/children`);
}

// --- Node content ---

export function getNodeContent(id: string): Promise<NodeContent> {
  return request(`/nodes/${id}/content`);
}

// --- Gaps ---

export function getGaps(level?: StoryLevel): Promise<TimelineGap[]> {
  const query = level ? `?level=${level}` : '';
  return request(`/timeline/gaps${query}`);
}

// --- References ---

export function listReferences(): Promise<ReferenceDocument[]> {
  return request('/references');
}

export function uploadReference(
  name: string,
  content: string,
  docType: ReferenceType,
): Promise<ReferenceDocument> {
  return request('/references', {
    method: 'POST',
    body: JSON.stringify({ name, content, doc_type: docType }),
  });
}

export function deleteReference(id: string): Promise<{ deleted: boolean }> {
  return request(`/references/${id}`, { method: 'DELETE' });
}

// --- AI ---

export function generateContent(nodeId: string): Promise<{ status: string; node_id: string }> {
  return request('/ai/generate', {
    method: 'POST',
    body: JSON.stringify({ node_id: nodeId }),
  });
}

export function getAiStatus(): Promise<AiStatus> {
  return request('/ai/status');
}

export function updateAiConfig(updates: Partial<AiConfig>): Promise<AiConfig> {
  return request('/ai/config', {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}

export function getAiContext(nodeId: string): Promise<{ system: string; user: string }> {
  return request(`/ai/context/${nodeId}`);
}

export function generateChildren(nodeId: string): Promise<ChildPlan> {
  return request('/ai/generate-children', {
    method: 'POST',
    body: JSON.stringify({ node_id: nodeId }),
  });
}

export function generateBatch(
  parentNodeId: string,
): Promise<{ status: string; parent_node_id: string; child_count: number }> {
  return request('/ai/generate-batch', {
    method: 'POST',
    body: JSON.stringify({ parent_node_id: parentNodeId }),
  });
}

// --- Diffusion LLM ---

export function getDiffusionStatus(): Promise<DiffusionStatus> {
  return request('/ai/diffusion/status');
}

export function loadDiffusionModel(
  model_path: string,
  device: string = 'cuda',
): Promise<{ status: string; model_path: string; device: string }> {
  return request('/ai/diffusion/load', {
    method: 'POST',
    body: JSON.stringify({ model_path, device }),
  });
}

export function unloadDiffusionModel(): Promise<{ status: string }> {
  return request('/ai/diffusion/unload', { method: 'POST' });
}

// --- Model Library ---

export function listModels(params?: {
  q?: string;
  model_type?: string;
  limit?: number;
  offset?: number;
}): Promise<ModelListResponse> {
  const query = new URLSearchParams();
  if (params?.q) query.set('q', params.q);
  if (params?.model_type) query.set('model_type', params.model_type);
  if (params?.limit) query.set('limit', String(params.limit));
  if (params?.offset) query.set('offset', String(params.offset));
  const qs = query.toString();
  return request(`/models${qs ? `?${qs}` : ''}`);
}

// --- Export ---

export async function exportPdf(): Promise<Blob> {
  const res = await fetch(`${BASE}/export/pdf`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || 'Export failed');
  }
  return res.blob();
}

// --- Persistence ---

export function saveProject(path?: string): Promise<{ saved?: string; error?: string }> {
  return request('/project/save', {
    method: 'POST',
    body: JSON.stringify({ path }),
  });
}

export function loadProject(path: string): Promise<Project> {
  return request('/project/load', {
    method: 'POST',
    body: JSON.stringify({ path }),
  });
}

export function listProjects(): Promise<{ name: string; path: string; modified: string }[]> {
  return request('/project/list');
}
