import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getChangeReviewProjection } from '$lib/projectionApi.js';
import {
  changeReviewProjectionState,
  clearChangeReviewProjection,
  getCachedChangeReviewProjection,
  refreshChangeReviewProjection,
} from './changeReviewProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getChangeReviewProjection: vi.fn(),
}));

const getChangeReviewProjectionMock = vi.mocked(getChangeReviewProjection);

const projection = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    changes: [
      {
        event: {
          id: 'event-1',
          command_id: 'command-1',
          kind: 'ai_proposal_accepted' as const,
          summary: 'accept bible reference Ada',
          created_at_ms: 100,
        },
        revisions: [],
      },
    ],
  },
};

beforeEach(() => {
  clearChangeReviewProjection();
  getChangeReviewProjectionMock.mockReset();
});

describe('change review projection store', () => {
  it('stores backend change review projections', async () => {
    getChangeReviewProjectionMock.mockResolvedValue(projection);

    await expect(refreshChangeReviewProjection()).resolves.toEqual(projection);

    expect(getChangeReviewProjectionMock).toHaveBeenCalledWith();
    expect(getCachedChangeReviewProjection()).toEqual(projection);
    expect(changeReviewProjectionState.pending).toBe(false);
    expect(changeReviewProjectionState.error).toBeUndefined();
  });

  it('records errors without replacing cached projections', async () => {
    getChangeReviewProjectionMock.mockResolvedValue(projection);
    await refreshChangeReviewProjection();
    getChangeReviewProjectionMock.mockRejectedValue(new Error('history unavailable'));

    await expect(refreshChangeReviewProjection()).rejects.toThrow('history unavailable');

    expect(getCachedChangeReviewProjection()).toEqual(projection);
    expect(changeReviewProjectionState.pending).toBe(false);
    expect(changeReviewProjectionState.error).toBe('history unavailable');
  });
});
