import { afterEach, describe, expect, it, vi } from 'vitest';

import { getPropagationProposalListProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('propagation proposal projection api helpers', () => {
  it('uses the desktop propagation proposal projection command', async () => {
    const response = {
      version: 4,
      change_event_id: 'event-4',
      payload: {
        proposals: [],
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

    await expect(getPropagationProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_propagation_proposals', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

    await expect(getPropagationProposalListProjection()).rejects.toThrow(
      'desktop transport is unavailable',
    );

    expect(fetch).not.toHaveBeenCalled();
  });
});
