export type BackendType = 'llama_cpp' | 'pumas' | 'open_router';

export interface AiConfig {
  backend_type: BackendType;
  model: string;
  temperature: number;
  max_tokens: number;
  base_url: string;
  api_key: string | null;
  pumas_profile?: string;
  embedding?: EmbeddingConfig;
}

export interface AiStatus {
  backend: BackendType;
  model?: string;
  connected: boolean;
  message?: string;
  error?: string;
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

export interface EmbeddingConfig {
  provider: 'disabled' | 'pumas' | 'open_ai_compatible';
  base_url: string;
  model: string;
  profile: string;
  revision: string;
  api_key: string | null;
}
export interface ReferenceIndexStatus {
  documents: {
    document_id: string;
    name: string;
    state: string;
    indexed_chunks: number;
    dimensions: number | null;
    model: string | null;
    revision: string | null;
    error: string | null;
  }[];
}
export interface PumasCatalog {
  models: { models: { id: string; official_name: string; model_type: string }[] };
  profiles: {
    snapshot: { profiles: { profile_id: string; provider: string; device: { mode: string } }[] };
  };
}
