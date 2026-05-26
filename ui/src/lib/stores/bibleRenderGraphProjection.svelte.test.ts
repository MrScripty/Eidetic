import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getBibleRenderGraphProjection } from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import {
  bibleRenderGraphRequestForTimelineSelection,
  bibleRenderGraphRequestForWorkspaceSelection,
  bibleRenderGraphProjectionState,
  clearBibleRenderGraphProjection,
  getActiveBibleRenderGraphProjectionRequest,
  getCachedBibleRenderGraphProjection,
  refreshBibleRenderGraphProjection,
} from './bibleRenderGraphProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getBibleRenderGraphProjection: vi.fn(),
}));

const getBibleRenderGraphProjectionMock = vi.mocked(getBibleRenderGraphProjection);

const projection: ProjectionEnvelope<BibleRenderGraphProjection> = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    nodes: [
      {
        node_id: 'node.character.ada',
        parent_id: null,
        schema_key: 'character',
        category: 'character',
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

const newerProjection: ProjectionEnvelope<BibleRenderGraphProjection> = {
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
        category: 'character',
        label: 'Bea',
        system_owned: false,
        sort_order: 0,
        depth: 0,
        position: { x: 1, y: 0, z: 0 },
      },
    ],
  },
};

const olderProjection: ProjectionEnvelope<BibleRenderGraphProjection> = {
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
        category: 'character',
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
  it('builds bounded selected timeline graph requests', () => {
    expect(bibleRenderGraphRequestForTimelineSelection('node.scene.beach')).toEqual({
      selected_timeline_node_id: 'node.scene.beach',
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
    expect(bibleRenderGraphRequestForTimelineSelection(null)).toEqual({
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
  });

  it('builds bounded graph workspace requests without normal selection scope', () => {
    expect(
      bibleRenderGraphRequestForWorkspaceSelection({
        selectedTimelineNodeId: 'node.scene.beach',
        activeTimelineMs: 12_345.67,
        focusedRootId: 'canonical.characters',
        search: ' Ada ',
      }),
    ).toEqual({
      focused_root_id: 'canonical.characters',
      selected_timeline_node_id: 'node.scene.beach',
      active_timeline_ms: 12_345,
      search: 'Ada',
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
  });

  it('uses selected node scope only for explicit focus-neighborhood requests', () => {
    expect(
      bibleRenderGraphRequestForWorkspaceSelection({
        focusedNeighborhoodNodeId: 'node.character.ada',
      }),
    ).toEqual({
      selected_node_id: 'node.character.ada',
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
  });

  it('stores backend bible render graph projections', async () => {
    getBibleRenderGraphProjectionMock.mockResolvedValue(projection);

    await expect(refreshBibleRenderGraphProjection()).resolves.toEqual(projection);

    expect(getBibleRenderGraphProjectionMock).toHaveBeenCalledWith(undefined);
    expect(getCachedBibleRenderGraphProjection()).toEqual(projection);
    expect(bibleRenderGraphProjectionState.pending).toBe(false);
    expect(bibleRenderGraphProjectionState.error).toBeUndefined();
  });

  it('requests selected timeline node bounded render graph projections', async () => {
    getBibleRenderGraphProjectionMock.mockResolvedValue(projection);
    const request = { selected_timeline_node_id: 'node.scene.beach' };

    await expect(refreshBibleRenderGraphProjection(request)).resolves.toEqual(projection);

    expect(getBibleRenderGraphProjectionMock).toHaveBeenCalledWith(request);
    expect(getActiveBibleRenderGraphProjectionRequest()).toEqual(request);
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
