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
