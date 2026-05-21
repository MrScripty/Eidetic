import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createProject,
  exportPdf,
  getAiContext,
  getAiStatus,
  getProject,
  listModels,
  listProjects,
  saveProject,
  updateAiConfig,
  updateProject,
} from './api.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('api request handling', () => {
  it('throws backend errors for non-ok responses', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'missing project' }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(getProject()).rejects.toThrow('missing project');
  });

  it('uses the desktop project command when Tauri transport is available', async () => {
    const invoke = vi.fn().mockResolvedValue({ name: 'Desktop Project', premise: '' });
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(getProject()).resolves.toMatchObject({
      name: 'Desktop Project',
    });
    expect(invoke).toHaveBeenCalledWith('project_get', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop project mutation commands when Tauri transport is available', async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({ name: 'Created', premise: '' })
      .mockResolvedValueOnce({ name: 'Renamed', premise: 'New premise' })
      .mockResolvedValueOnce({ saved: '/tmp/project.db' })
      .mockResolvedValueOnce([]);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await createProject('Created', 'multi_cam');
    await updateProject({ name: 'Renamed', premise: 'New premise' });
    await saveProject('/tmp/project.db');
    await listProjects();

    expect(invoke).toHaveBeenNthCalledWith(1, 'project_create', {
      name: 'Created',
      template: 'multi_cam',
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'project_update', {
      name: 'Renamed',
      premise: 'New premise',
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'project_save', {
      path: '/tmp/project.db',
    });
    expect(invoke).toHaveBeenNthCalledWith(4, 'project_list', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop AI status and config commands when Tauri transport is available', async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({
        backend: 'llama_cpp',
        model: 'served-model',
        connected: true,
        message: 'ok',
      })
      .mockResolvedValueOnce({
        backend_type: 'llama_cpp',
        model: 'served-model',
        temperature: 0.4,
        max_tokens: 2048,
        base_url: 'http://127.0.0.1:18080/v1',
        api_key: null,
      });
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await getAiStatus();
    await updateAiConfig({ model: 'served-model', temperature: 0.4 });

    expect(invoke).toHaveBeenNthCalledWith(1, 'ai_status', undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, 'ai_config_update', {
      updates: { model: 'served-model', temperature: 0.4 },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses the desktop AI context command when Tauri transport is available', async () => {
    const invoke = vi.fn().mockResolvedValue({
      system: 'system prompt',
      user: 'user prompt',
    });
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await getAiContext('00000000-0000-0000-0000-000000000001');

    expect(invoke).toHaveBeenCalledWith('ai_context_preview', {
      nodeId: '00000000-0000-0000-0000-000000000001',
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses the desktop model list command when Tauri transport is available', async () => {
    const invoke = vi.fn().mockResolvedValue({
      models: [],
      total_count: 0,
    });
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await listModels({ q: 'llama', model_type: 'llm', limit: 25, offset: 5 });

    expect(invoke).toHaveBeenCalledWith('model_list', {
      params: {
        q: 'llama',
        model_type: 'llm',
        limit: 25,
        offset: 5,
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses the desktop PDF export command when Tauri transport is available', async () => {
    const invoke = vi.fn().mockResolvedValue([37, 80, 68, 70]);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    const blob = await exportPdf();

    expect(invoke).toHaveBeenCalledWith('export_pdf', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
    expect(blob.type).toBe('application/pdf');
    await expect(blob.arrayBuffer()).resolves.toEqual(Uint8Array.from([37, 80, 68, 70]).buffer);
  });

  it('normalizes desktop command errors', async () => {
    vi.stubGlobal('window', {
      __TAURI__: {
        core: {
          invoke: vi.fn().mockRejectedValue({ kind: 'not_found', message: 'no project loaded' }),
        },
      },
    });

    await expect(getProject()).rejects.toThrow('no project loaded');
  });

  it('rejects ok responses that still carry an error payload', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'legacy error body' }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(getProject()).rejects.toThrow('legacy error body');
  });

  it('sends json requests and returns parsed payloads for success responses', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ id: 'project-1', name: 'Pilot' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(createProject('Pilot', 'multi_cam')).resolves.toMatchObject({
      id: 'project-1',
      name: 'Pilot',
    });
    expect(fetchMock).toHaveBeenCalledWith(
      '/api/project',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: 'Pilot', template: 'multi_cam' }),
      }),
    );
  });
});
