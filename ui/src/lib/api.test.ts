import { afterEach, describe, expect, it, vi } from 'vitest';

import { createProject, getProject } from './api.js';

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
