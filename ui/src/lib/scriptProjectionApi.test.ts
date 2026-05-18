import { afterEach, describe, expect, it, vi } from 'vitest';

import { getScriptDocumentProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('script projection api helpers', () => {
  it('fetches script document projections with encoded query params', async () => {
    const response = {
      version: 5,
      change_event_id: 'event-script-1',
      payload: {
        document: {
          id: 'script.document/main one',
          title: 'Pilot',
          sort_order: 0,
        },
        segments: [],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      getScriptDocumentProjection({
        document_id: 'script.document/main one',
      }),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/projections/script/document?document_id=script.document%2Fmain+one',
      {
        method: 'GET',
        headers: { Accept: 'application/json' },
      },
    );
  });

  it('throws backend errors without local fallback state', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'script document not found' }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(
      getScriptDocumentProjection({
        document_id: 'script.document.main',
      }),
    ).rejects.toThrow('script document not found');
  });
});
