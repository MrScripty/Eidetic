import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getBibleRenderGraphProjection } from '$lib/projectionApi.js';
import {
  bibleRenderGraphProjectionState,
  clearBibleRenderGraphProjection,
  getCachedBibleRenderGraphProjection,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getBibleRenderGraphProjection: vi.fn(),
}));

const getBibleRenderGraphProjectionMock = vi.mocked(getBibleRenderGraphProjection);

const projection = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    nodes: [
      {
        node_id: 'node.character.ada',
        parent_id: null,
        schema_key: 'character',
        label: 'Ada',
        system_owned: false,
        sort_order: 0,
        depth: 0,
        position: { x: 0, y: 0, z: 0 },
      },
    ],
    edges: [],
    neighborhoods: [],
    influences: [],
  },
};

const newerProjection = {
  ...projection,
  version: 4,
  change_event_id: 'event-4',
  payload: {
    ...projection.payload,
    nodes: [
      {
        node_id: 'node.character.bea',
        parent_id: null,
        schema_key: 'character',
        label: 'Bea',
        system_owned: false,
        sort_order: 0,
        depth: 0,
        position: { x: 1, y: 0, z: 0 },
      },
    ],
  },
};

const olderProjection = {
  ...projection,
  version: 3,
  change_event_id: 'event-3',
  payload: {
    ...projection.payload,
    nodes: [
      {
        node_id: 'node.character.cal',
        parent_id: null,
        schema_key: 'character',
        label: 'Cal',
        system_owned: false,
        sort_order: 0,
        depth: 0,
        position: { x: 2, y: 0, z: 0 },
      },
    ],
  },
};

beforeEach(() => {
  clearBibleRenderGraphProjection();
  getBibleRenderGraphProjectionMock.mockReset();
});

describe('bible render graph projection store', () => {
  it('stores backend bible render graph projections', async () => {
    getBibleRenderGraphProjectionMock.mockResolvedValue(projection);

    await expect(refreshBibleRenderGraphProjection()).resolves.toEqual(projection);

    expect(getBibleRenderGraphProjectionMock).toHaveBeenCalledWith();
    expect(getCachedBibleRenderGraphProjection()).toEqual(projection);
    expect(bibleRenderGraphProjectionState.pending).toBe(false);
    expect(bibleRenderGraphProjectionState.error).toBeUndefined();
  });

  it('records errors without replacing cached projections', async () => {
    getBibleRenderGraphProjectionMock.mockResolvedValue(projection);
    await refreshBibleRenderGraphProjection();
    getBibleRenderGraphProjectionMock.mockRejectedValue(new Error('render graph unavailable'));

    await expect(refreshBibleRenderGraphProjection()).rejects.toThrow('render graph unavailable');

    expect(getCachedBibleRenderGraphProjection()).toEqual(projection);
    expect(bibleRenderGraphProjectionState.pending).toBe(false);
    expect(bibleRenderGraphProjectionState.error).toBe('render graph unavailable');
  });

  it('does not replace cached render graph projections with stale refresh results', async () => {
    getBibleRenderGraphProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleRenderGraphProjection();
    getBibleRenderGraphProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshBibleRenderGraphProjection()).resolves.toEqual(olderProjection);

    expect(getCachedBibleRenderGraphProjection()).toEqual(newerProjection);
    expect(bibleRenderGraphProjectionState.pending).toBe(false);
    expect(bibleRenderGraphProjectionState.error).toBeUndefined();
  });
});
