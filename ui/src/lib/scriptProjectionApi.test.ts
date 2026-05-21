import { afterEach, describe, expect, it, vi } from 'vitest';

import { getScriptDocumentProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('script projection api helpers', () => {
  it('uses the desktop script document projection command', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      getScriptDocumentProjection({
        document_id: 'script.document/main one',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_script_document', {
      query: { document_id: 'script.document/main one' },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

    await expect(
      getScriptDocumentProjection({
        document_id: 'script.document.main',
      }),
    ).rejects.toThrow('desktop transport is unavailable');

    expect(fetch).not.toHaveBeenCalled();
  });
});
