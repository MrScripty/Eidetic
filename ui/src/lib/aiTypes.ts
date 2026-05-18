export type BackendType = 'ollama' | 'open_router';

export interface AiConfig {
  backend_type: BackendType;
  model: string;
  temperature: number;
  max_tokens: number;
  base_url: string;
  api_key: string | null;
}

export interface AiStatus {
  backend: BackendType;
  model?: string;
  connected: boolean;
  message?: string;
  error?: string;
}

export interface DiffuseRequest {
  node_id: string;
  anchor_ranges: { start: number; end: number }[];
  mask_budget: number;
}

export interface DiffusionStatus {
  model_loaded: boolean;
  model_path: string | null;
  device: string;
}

export interface ModelEntry {
  id: string;
  name: string;
  path: string;
  model_type: string;
  size_bytes: number | null;
  tags: string[];
}

export interface ModelListResponse {
  models: ModelEntry[];
  total_count: number;
}
