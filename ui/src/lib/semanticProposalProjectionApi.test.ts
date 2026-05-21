import { afterEach, describe, expect, it, vi } from 'vitest';

import { getBibleReferenceProposalListProjection } from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('semantic proposal projection api helpers', () => {
  it('uses the desktop bible reference proposal projection command', async () => {
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

    await expect(getBibleReferenceProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_reference_proposals', undefined);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

    await expect(getBibleReferenceProposalListProjection()).rejects.toThrow(
      'desktop transport is unavailable',
    );

    expect(fetch).not.toHaveBeenCalled();
  });
});
