import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
} from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';
import type { BibleNodeDetailProjection } from '../bibleGraphTypes.js';
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

const newerProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  ...projection,
  version: 4,
  change_event_id: 'event-newer',
  payload: {
    ...projection.payload,
    node: {
      ...projection.payload.node,
      name: 'Ada newer',
    },
  },
};

const olderProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  ...projection,
  version: 3,
  change_event_id: 'event-older',
  payload: {
    ...projection.payload,
    node: {
      ...projection.payload.node,
      name: 'Ada older',
    },
  },
};

const newerListProjection = {
  ...listProjection,
  version: 4,
  change_event_id: 'event-list-newer',
  payload: {
    nodes: [newerProjection.payload.node],
  },
};

const olderListProjection = {
  ...listProjection,
  version: 3,
  change_event_id: 'event-list-older',
  payload: {
    nodes: [olderProjection.payload.node],
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

  it('does not replace cached graph node projections with stale refresh results', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleGraphNodeProjection(key);
    getBibleGraphNodeProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshBibleGraphNodeProjection(key)).resolves.toEqual(olderProjection);

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(newerProjection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('does not replace cached graph node lists with stale refresh results', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValueOnce(newerListProjection);
    await refreshBibleGraphNodeListProjection();
    getBibleGraphNodeListProjectionMock.mockResolvedValueOnce(olderListProjection);

    await expect(refreshBibleGraphNodeListProjection()).resolves.toEqual(olderListProjection);

    expect(getCachedBibleGraphNodeListProjection()).toEqual(newerListProjection);
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBeUndefined();
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
