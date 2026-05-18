import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
} from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';
import type { BibleNodeDetailProjection } from '../types.js';
import {
  bibleGraphNodeProjectionState,
  clearBibleGraphNodeListProjection,
  clearBibleGraphNodeProjection,
  getCachedBibleGraphNodeListProjection,
  getBibleGraphNodeProjectionError,
  getCachedBibleGraphNodeProjection,
  isBibleGraphNodeProjectionPending,
  refreshBibleGraphNodeListProjection,
  refreshBibleGraphNodeProjection,
} from './bibleGraphNodeProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphNodeListProjection: vi.fn(),
  getBibleGraphNodeProjection: vi.fn(),
}));

const getBibleGraphNodeProjectionMock = vi.mocked(getBibleGraphNodeProjection);
const getBibleGraphNodeListProjectionMock = vi.mocked(getBibleGraphNodeListProjection);

const key = {
  node_id: 'node.character/ada one',
};

const projection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    node: {
      id: 'node.character/ada one',
      parent_id: null,
      schema_key: 'character',
      name: 'Ada',
      system_owned: false,
      sort_order: 3,
    },
    parts: [],
    incoming_edges: [],
    outgoing_edges: [
      {
        id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      },
    ],
    snapshots: [],
  },
};

const listProjection = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    nodes: [projection.payload.node],
  },
};

function resetProjectionState(): void {
  for (const keyString of Object.keys(bibleGraphNodeProjectionState.projections)) {
    delete bibleGraphNodeProjectionState.projections[keyString];
  }
  for (const keyString of Object.keys(bibleGraphNodeProjectionState.pending)) {
    delete bibleGraphNodeProjectionState.pending[keyString];
  }
  for (const keyString of Object.keys(bibleGraphNodeProjectionState.errors)) {
    delete bibleGraphNodeProjectionState.errors[keyString];
  }
  bibleGraphNodeProjectionState.nodeList = null;
  bibleGraphNodeProjectionState.nodeListPending = false;
  bibleGraphNodeProjectionState.nodeListError = undefined;
}

beforeEach(() => {
  resetProjectionState();
  getBibleGraphNodeProjectionMock.mockReset();
  getBibleGraphNodeListProjectionMock.mockReset();
});

describe('bible graph node projection store', () => {
  it('stores backend graph node projection reads and clears pending state', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);

    await expect(refreshBibleGraphNodeProjection(key)).resolves.toEqual(projection);

    expect(getBibleGraphNodeProjectionMock).toHaveBeenCalledWith(key);
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('preserves edge payloads from backend graph node projection reads', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);

    await refreshBibleGraphNodeProjection(key);

    expect(getCachedBibleGraphNodeProjection(key)?.payload.outgoing_edges).toEqual([
      {
        id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      },
    ]);
  });

  it('records read errors without caching a projection', async () => {
    getBibleGraphNodeProjectionMock.mockRejectedValue(new Error('node not found'));

    await expect(refreshBibleGraphNodeProjection(key)).rejects.toThrow('node not found');

    expect(getCachedBibleGraphNodeProjection(key)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('node not found');
  });

  it('stores backend graph node list reads and clears pending state', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);

    await expect(refreshBibleGraphNodeListProjection()).resolves.toEqual(listProjection);

    expect(getBibleGraphNodeListProjectionMock).toHaveBeenCalledWith();
    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBeUndefined();
  });

  it('records node list read errors without caching a projection', async () => {
    getBibleGraphNodeListProjectionMock.mockRejectedValue(new Error('list unavailable'));

    await expect(refreshBibleGraphNodeListProjection()).rejects.toThrow('list unavailable');

    expect(getCachedBibleGraphNodeListProjection()).toBeNull();
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBe('list unavailable');
  });

  it('clears cached graph node projection state for one node', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);

    clearBibleGraphNodeProjection(key);

    expect(getCachedBibleGraphNodeProjection(key)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('clears cached graph node list projection state', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();

    clearBibleGraphNodeListProjection();

    expect(getCachedBibleGraphNodeListProjection()).toBeNull();
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBeUndefined();
  });
});
