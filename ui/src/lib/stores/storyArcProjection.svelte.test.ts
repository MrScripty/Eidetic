import { beforeEach, describe, expect, it, vi } from 'vitest';

import { createStoryArc, deleteStoryArc, setStoryArcMetadata } from '$lib/commandApi.js';
import { getStoryArcListProjection } from '$lib/projectionApi.js';
import {
  applyCreateStoryArcCommand,
  applyDeleteStoryArcCommand,
  applySetStoryArcMetadataCommand,
  clearStoryArcListProjection,
  getCachedStoryArcListProjection,
  refreshStoryArcListProjection,
  storyArcProjectionState,
} from './storyArcProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  createStoryArc: vi.fn(),
  deleteStoryArc: vi.fn(),
  setStoryArcMetadata: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getStoryArcListProjection: vi.fn(),
}));

const createStoryArcMock = vi.mocked(createStoryArc);
const deleteStoryArcMock = vi.mocked(deleteStoryArc);
const setStoryArcMetadataMock = vi.mocked(setStoryArcMetadata);
const getStoryArcListProjectionMock = vi.mocked(getStoryArcListProjection);

const projection = {
  version: 1,
  payload: {
    arcs: [
      {
        id: 'arc.mystery',
        parent_arc_id: null,
        name: 'Mystery',
        description: 'Central investigation',
        arc_type: 'APlot' as const,
        color: { r: 1, g: 2, b: 3 },
      },
    ],
  },
};

const newerProjection = {
  ...projection,
  version: 3,
  payload: {
    arcs: [
      {
        id: 'arc.romance',
        parent_arc_id: null,
        name: 'Romance',
        description: 'Relationship thread',
        arc_type: 'APlot' as const,
        color: { r: 7, g: 8, b: 9 },
      },
    ],
  },
};

const olderProjection = {
  ...projection,
  version: 2,
  payload: {
    arcs: [
      {
        id: 'arc.comedy',
        parent_arc_id: null,
        name: 'Comedy',
        description: 'Older comic thread',
        arc_type: 'APlot' as const,
        color: { r: 4, g: 5, b: 6 },
      },
    ],
  },
};

beforeEach(() => {
  clearStoryArcListProjection();
  createStoryArcMock.mockReset();
  deleteStoryArcMock.mockReset();
  setStoryArcMetadataMock.mockReset();
  getStoryArcListProjectionMock.mockReset();
});

describe('story arc projection store', () => {
  it('stores backend arc projections and bridges display arcs', async () => {
    getStoryArcListProjectionMock.mockResolvedValue(projection);

    await expect(refreshStoryArcListProjection()).resolves.toEqual(projection);

    expect(getStoryArcListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedStoryArcListProjection()).toEqual(projection);
    expect(storyArcProjectionState.pending).toBe(false);
    expect(storyArcProjectionState.error).toBeUndefined();
  });

  it('stores command response projections without optimistic arc patches', async () => {
    createStoryArcMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyCreateStoryArcCommand(
        {
          arc_id: 'arc.mystery',
          parent_arc_id: null,
          name: 'Mystery',
          description: 'Central investigation',
          arc_type: 'APlot',
          color: { r: 1, g: 2, b: 3 },
        },
        'command-story-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(createStoryArcMock).toHaveBeenCalledWith(
      {
        arc_id: 'arc.mystery',
        parent_arc_id: null,
        name: 'Mystery',
        description: 'Central investigation',
        arc_type: 'APlot',
        color: { r: 1, g: 2, b: 3 },
      },
      'command-story-1',
    );
    expect(getStoryArcListProjectionMock).not.toHaveBeenCalled();
    expect(getCachedStoryArcListProjection()).toEqual(projection);
  });

  it('stores update and delete command projections', async () => {
    setStoryArcMetadataMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });
    deleteStoryArcMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await applySetStoryArcMetadataCommand(
      {
        arc_id: 'arc.mystery',
        name: 'Mystery revised',
      },
      'command-story-2',
    );
    await applyDeleteStoryArcCommand(
      {
        arc_id: 'arc.mystery',
      },
      'command-story-3',
    );

    expect(setStoryArcMetadataMock).toHaveBeenCalledWith(
      {
        arc_id: 'arc.mystery',
        name: 'Mystery revised',
      },
      'command-story-2',
    );
    expect(deleteStoryArcMock).toHaveBeenCalledWith(
      {
        arc_id: 'arc.mystery',
      },
      'command-story-3',
    );
    expect(getCachedStoryArcListProjection()).toEqual(projection);
  });

  it('records errors without replacing cached projections', async () => {
    getStoryArcListProjectionMock.mockResolvedValue(projection);
    await refreshStoryArcListProjection();
    getStoryArcListProjectionMock.mockRejectedValue(new Error('arcs unavailable'));

    await expect(refreshStoryArcListProjection()).rejects.toThrow('arcs unavailable');

    expect(getCachedStoryArcListProjection()).toEqual(projection);
    expect(storyArcProjectionState.pending).toBe(false);
    expect(storyArcProjectionState.error).toBe('arcs unavailable');
  });

  it('does not replace cached arc projections with stale refresh results', async () => {
    getStoryArcListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshStoryArcListProjection();
    getStoryArcListProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshStoryArcListProjection()).resolves.toEqual(olderProjection);

    expect(getCachedStoryArcListProjection()).toEqual(newerProjection);
    expect(storyArcProjectionState.pending).toBe(false);
    expect(storyArcProjectionState.error).toBeUndefined();
  });

  it('does not replace cached arc projections with stale command responses', async () => {
    getStoryArcListProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshStoryArcListProjection();
    createStoryArcMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      applyCreateStoryArcCommand(
        {
          name: 'Comedy',
          description: 'Older command response',
          arc_type: 'APlot',
          color: { r: 4, g: 5, b: 6 },
        },
        'command-story-stale',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedStoryArcListProjection()).toEqual(newerProjection);
    expect(storyArcProjectionState.pending).toBe(false);
    expect(storyArcProjectionState.error).toBeUndefined();
  });
});
