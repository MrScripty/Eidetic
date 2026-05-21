import { afterEach, describe, expect, it, vi } from 'vitest';

import { getChangeReviewProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('change review projection api helpers', () => {
  it('uses the desktop change review projection command', async () => {
    const response = {
      version: 2,
      change_event_id: 'event-1',
      payload: {
        changes: [],
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

    await expect(getChangeReviewProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_change_review', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

    await expect(getChangeReviewProjection()).rejects.toThrow('desktop transport is unavailable');

    expect(fetch).not.toHaveBeenCalled();
  });
});
