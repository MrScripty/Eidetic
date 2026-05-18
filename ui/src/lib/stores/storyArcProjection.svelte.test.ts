import { beforeEach, describe, expect, it, vi } from 'vitest';

import { createStoryArc, deleteStoryArc, setStoryArcMetadata } from '$lib/commandApi.js';
import { getStoryArcListProjection } from '$lib/projectionApi.js';
import { storyState } from './story.svelte.js';
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
    expect(storyState.arcs).toEqual(projection.payload.arcs);
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
    expect(storyState.arcs).toEqual(projection.payload.arcs);
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
    expect(storyState.arcs).toEqual(projection.payload.arcs);
  });

  it('records errors without replacing cached projections', async () => {
    getStoryArcListProjectionMock.mockResolvedValue(projection);
    await refreshStoryArcListProjection();
    getStoryArcListProjectionMock.mockRejectedValue(new Error('arcs unavailable'));

    await expect(refreshStoryArcListProjection()).rejects.toThrow('arcs unavailable');

    expect(getCachedStoryArcListProjection()).toEqual(projection);
    expect(storyState.arcs).toEqual(projection.payload.arcs);
    expect(storyArcProjectionState.pending).toBe(false);
    expect(storyArcProjectionState.error).toBe('arcs unavailable');
  });
});
