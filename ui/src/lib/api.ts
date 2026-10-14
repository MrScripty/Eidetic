import type {
  AiConfig,
  AiStatus,
  ModelListResponse,
  PumasCatalog,
  PumasUnserveResult,
  ReferenceIndexStatus,
} from './aiTypes.js';
import type { ChildPlan } from './childPlanningTypes.js';
import { invokeDesktop } from './desktopTransport.js';
import type { Project, ReferenceDocument, ReferenceType } from './projectTypes.js';

// --- Project ---

export function createProject(name: string, template: string): Promise<Project> {
  return invokeDesktop<Project>('project_create', { name, template });
}

export function getProject(): Promise<Project> {
  return invokeDesktop<Project>('project_get');
}

export function updateProject(updates: { name?: string; premise?: string }): Promise<Project> {
  return invokeDesktop<Project>('project_update', updates);
}

// --- References ---

function referenceTypeToWireValue(docType: ReferenceType): string {
  return typeof docType === 'string' ? docType : docType.Custom;
}

export function listReferences(): Promise<ReferenceDocument[]> {
  return invokeDesktop<ReferenceDocument[]>('reference_list');
}

export function uploadReference(
  name: string,
  content: string,
  docType: ReferenceType,
): Promise<ReferenceDocument> {
  const doc_type = referenceTypeToWireValue(docType);
  return invokeDesktop<ReferenceDocument>('reference_upload', {
    request: { name, content, doc_type },
  });
}

export function deleteReference(id: string): Promise<{ deleted: boolean }> {
  return invokeDesktop<{ deleted: boolean }>('reference_delete', { id });
}

// --- AI ---

export function generateContent(
  nodeId: string,
  storyTimeMs?: number,
): Promise<{ status: string; node_id: string }> {
  return invokeDesktop<{ status: string; node_id: string }>('ai_generate_content', {
    request: { node_id: nodeId, ...storyTimeRequest(storyTimeMs) },
  });
}

export function getAiStatus(): Promise<AiStatus> {
  return invokeDesktop<AiStatus>('ai_status');
}

export function updateAiConfig(updates: Partial<AiConfig>): Promise<AiConfig> {
  return invokeDesktop<AiConfig>('ai_config_update', { updates });
}

export function getAiContext(
  nodeId: string,
  storyTimeMs?: number,
): Promise<{ system: string; user: string }> {
  const time = storyTimeRequest(storyTimeMs);
  return invokeDesktop<{ system: string; user: string }>('ai_context_preview', {
    nodeId,
    ...(time.story_time_ms === undefined ? {} : { storyTimeMs: time.story_time_ms }),
  });
}

export function generateChildren(nodeId: string, storyTimeMs?: number): Promise<ChildPlan> {
  return invokeDesktop<ChildPlan>('ai_generate_children', {
    request: { node_id: nodeId, ...storyTimeRequest(storyTimeMs) },
  });
}

// This coordinate is explicit fictional time, not the timeline playhead.
function storyTimeRequest(storyTimeMs?: number): { story_time_ms?: number } {
  if (storyTimeMs === undefined) return {};
  if (!Number.isSafeInteger(storyTimeMs) || storyTimeMs < 0) {
    throw new RangeError('Story time must be a non-negative safe integer in milliseconds');
  }
  return { story_time_ms: storyTimeMs };
}

export function generateBatch(
  parentNodeId: string,
): Promise<{ status: string; parent_node_id: string; child_count: number }> {
  return invokeDesktop<{ status: string; parent_node_id: string; child_count: number }>(
    'ai_generate_batch',
    {
      request: { parent_node_id: parentNodeId },
    },
  );
}

// --- Model Library ---

export function listModels(params?: {
  q?: string;
  model_type?: string;
  limit?: number;
  offset?: number;
}): Promise<ModelListResponse> {
  return invokeDesktop<ModelListResponse>('model_list', {
    params: {
      q: params?.q ?? '',
      model_type: params?.model_type ?? null,
      limit: params?.limit ?? 100,
      offset: params?.offset ?? 0,
    },
  });
}

// --- Export ---

export async function exportPdf(): Promise<Blob> {
  const bytes = await invokeDesktop<number[]>('export_pdf');
  return new Blob([Uint8Array.from(bytes)], { type: 'application/pdf' });
}

// --- Persistence ---

export function saveProject(path?: string): Promise<{ saved?: string; error?: string }> {
  return invokeDesktop<{ saved?: string; error?: string }>('project_save', { path });
}

export function loadProject(path: string): Promise<Project> {
  return invokeDesktop<Project>('project_load', { path });
}

export function listProjects(): Promise<{ name: string; path: string; modified: string }[]> {
  return invokeDesktop<{ name: string; path: string; modified: string }[]>('project_list');
}

export function getAiConfig(): Promise<AiConfig> {
  return invokeDesktop<AiConfig>('ai_config_get');
}
export function getReferenceIndexStatus(): Promise<ReferenceIndexStatus> {
  return invokeDesktop<ReferenceIndexStatus>('reference_index_status');
}
export function reindexReferences(): Promise<void> {
  return invokeDesktop<void>('reference_reindex');
}
export function getPumasCatalog(endpoint: string): Promise<PumasCatalog> {
  return invokeDesktop<PumasCatalog>('pumas_catalog', { endpoint });
}
export function loadPumasModel(
  endpoint: string,
  model: string,
  profile: string,
  provider: string,
  deviceMode: string,
): Promise<unknown> {
  return invokeDesktop('pumas_load_model', {
    endpoint,
    request: {
      model_id: model,
      config: { profile_id: profile, provider, device_mode: deviceMode, keep_loaded: true },
    },
  });
}
export function unloadPumasModel(
  endpoint: string,
  model: string,
  profile: string,
  provider: string,
): Promise<PumasUnserveResult> {
  return invokeDesktop<PumasUnserveResult>('pumas_unload_model', {
    endpoint,
    request: { model_id: model, profile_id: profile, provider },
  });
}
