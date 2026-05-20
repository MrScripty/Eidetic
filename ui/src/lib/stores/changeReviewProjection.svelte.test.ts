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

const newerProjection = {
  ...projection,
  version: 5,
  change_event_id: 'event-5',
  payload: {
    changes: [
      {
        event: {
          id: 'event-5',
          command_id: 'command-5',
          kind: 'user_edit' as const,
          summary: 'manual rain change',
          created_at_ms: 500,
        },
        revisions: [],
      },
    ],
  },
};

const olderProjection = {
  ...projection,
  version: 4,
  change_event_id: 'event-4',
  payload: {
    changes: [
      {
        event: {
          id: 'event-4',
          command_id: 'command-4',
          kind: 'ai_proposal_rejected' as const,
          summary: 'reject stale proposal',
          created_at_ms: 400,
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

  it('does not replace cached change review projections with stale refresh results', async () => {
    getChangeReviewProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshChangeReviewProjection();
    getChangeReviewProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshChangeReviewProjection()).resolves.toEqual(olderProjection);

    expect(getCachedChangeReviewProjection()).toEqual(newerProjection);
    expect(changeReviewProjectionState.pending).toBe(false);
    expect(changeReviewProjectionState.error).toBeUndefined();
  });
});
