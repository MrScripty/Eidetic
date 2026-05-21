import type { AiConfig, AiStatus, ModelListResponse } from './aiTypes.js';
import type { ChildPlan } from './childPlanningTypes.js';
import { hasDesktopTransport, invokeDesktop } from './desktopTransport.js';
import type { Project, ReferenceDocument, ReferenceType } from './projectTypes.js';

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
  if (hasDesktopTransport()) {
    return invokeDesktop<Project>('project_create', { name, template });
  }
  return request('/project', {
    method: 'POST',
    body: JSON.stringify({ name, template }),
  });
}

export function getProject(): Promise<Project> {
  if (hasDesktopTransport()) {
    return invokeDesktop<Project>('project_get');
  }
  return request('/project');
}

export function updateProject(updates: { name?: string; premise?: string }): Promise<Project> {
  if (hasDesktopTransport()) {
    return invokeDesktop<Project>('project_update', updates);
  }
  return request('/project', {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}

// --- References ---

function referenceTypeToWireValue(docType: ReferenceType): string {
  return typeof docType === 'string' ? docType : docType.Custom;
}

export function listReferences(): Promise<ReferenceDocument[]> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ReferenceDocument[]>('reference_list');
  }
  return request('/references');
}

export function uploadReference(
  name: string,
  content: string,
  docType: ReferenceType,
): Promise<ReferenceDocument> {
  const doc_type = referenceTypeToWireValue(docType);
  if (hasDesktopTransport()) {
    return invokeDesktop<ReferenceDocument>('reference_upload', {
      request: { name, content, doc_type },
    });
  }
  return request('/references', {
    method: 'POST',
    body: JSON.stringify({ name, content, doc_type }),
  });
}

export function deleteReference(id: string): Promise<{ deleted: boolean }> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ deleted: boolean }>('reference_delete', { id });
  }
  return request(`/references/${id}`, { method: 'DELETE' });
}

// --- AI ---

export function generateContent(nodeId: string): Promise<{ status: string; node_id: string }> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ status: string; node_id: string }>('ai_generate_content', {
      request: { node_id: nodeId },
    });
  }
  return request('/ai/generate', {
    method: 'POST',
    body: JSON.stringify({ node_id: nodeId }),
  });
}

export function getAiStatus(): Promise<AiStatus> {
  if (hasDesktopTransport()) {
    return invokeDesktop<AiStatus>('ai_status');
  }
  return request('/ai/status');
}

export function updateAiConfig(updates: Partial<AiConfig>): Promise<AiConfig> {
  if (hasDesktopTransport()) {
    return invokeDesktop<AiConfig>('ai_config_update', { updates });
  }
  return request('/ai/config', {
    method: 'PUT',
    body: JSON.stringify(updates),
  });
}

export function getAiContext(nodeId: string): Promise<{ system: string; user: string }> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ system: string; user: string }>('ai_context_preview', {
      nodeId,
    });
  }
  return request(`/ai/context/${nodeId}`);
}

export function generateChildren(nodeId: string): Promise<ChildPlan> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ChildPlan>('ai_generate_children', {
      request: { node_id: nodeId },
    });
  }
  return request('/ai/generate-children', {
    method: 'POST',
    body: JSON.stringify({ node_id: nodeId }),
  });
}

export function generateBatch(
  parentNodeId: string,
): Promise<{ status: string; parent_node_id: string; child_count: number }> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ status: string; parent_node_id: string; child_count: number }>(
      'ai_generate_batch',
      {
        request: { parent_node_id: parentNodeId },
      },
    );
  }
  return request('/ai/generate-batch', {
    method: 'POST',
    body: JSON.stringify({ parent_node_id: parentNodeId }),
  });
}

// --- Model Library ---

export function listModels(params?: {
  q?: string;
  model_type?: string;
  limit?: number;
  offset?: number;
}): Promise<ModelListResponse> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ModelListResponse>('model_list', {
      params: {
        q: params?.q ?? '',
        model_type: params?.model_type ?? null,
        limit: params?.limit ?? 100,
        offset: params?.offset ?? 0,
      },
    });
  }
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
  if (hasDesktopTransport()) {
    const bytes = await invokeDesktop<number[]>('export_pdf');
    return new Blob([Uint8Array.from(bytes)], { type: 'application/pdf' });
  }
  const res = await fetch(`${BASE}/export/pdf`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json();
    throw new Error(err.error || 'Export failed');
  }
  return res.blob();
}

// --- Persistence ---

export function saveProject(path?: string): Promise<{ saved?: string; error?: string }> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ saved?: string; error?: string }>('project_save', { path });
  }
  return request('/project/save', {
    method: 'POST',
    body: JSON.stringify({ path }),
  });
}

export function loadProject(path: string): Promise<Project> {
  if (hasDesktopTransport()) {
    return invokeDesktop<Project>('project_load', { path });
  }
  return request('/project/load', {
    method: 'POST',
    body: JSON.stringify({ path }),
  });
}

export function listProjects(): Promise<{ name: string; path: string; modified: string }[]> {
  if (hasDesktopTransport()) {
    return invokeDesktop<{ name: string; path: string; modified: string }[]>('project_list');
  }
  return request('/project/list');
}
