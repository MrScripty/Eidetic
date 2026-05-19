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
});
