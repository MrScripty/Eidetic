import { beforeEach, describe, expect, it, vi } from 'vitest';

import { createBibleGraphNode } from '$lib/commandApi.js';
import { getBibleGraphNodeProjection } from '$lib/projectionApi.js';
import { storyState } from './story.svelte.js';
import {
  bibleGraphNodeProjectionState,
  clearBibleGraphNodeProjection,
  createBibleGraphNodeProjection,
  getBibleGraphNodeProjectionError,
  getCachedBibleGraphNodeProjection,
  isBibleGraphNodeProjectionPending,
  refreshBibleGraphNodeProjection,
} from './bibleGraphNodeProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  createBibleGraphNode: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphNodeProjection: vi.fn(),
}));

const createBibleGraphNodeMock = vi.mocked(createBibleGraphNode);
const getBibleGraphNodeProjectionMock = vi.mocked(getBibleGraphNodeProjection);

const key = {
  node_id: 'node.character/ada one',
};

const projection = {
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
    outgoing_edges: [],
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
}

beforeEach(() => {
  resetProjectionState();
  storyState.entities = [];
  createBibleGraphNodeMock.mockReset();
  getBibleGraphNodeProjectionMock.mockReset();
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

  it('records read errors without caching a projection', async () => {
    getBibleGraphNodeProjectionMock.mockRejectedValue(new Error('node not found'));

    await expect(refreshBibleGraphNodeProjection(key)).rejects.toThrow('node not found');

    expect(getCachedBibleGraphNodeProjection(key)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('node not found');
  });

  it('stores create command response projections without mutating legacy entity state', async () => {
    const originalEntities = storyState.entities;
    createBibleGraphNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      createBibleGraphNodeProjection(
        {
          node_id: 'node.character/ada one',
          parent_id: null,
          schema_key: 'character',
          name: 'Ada',
          sort_order: 3,
        },
        'command-graph-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(createBibleGraphNodeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.character/ada one',
        parent_id: null,
        schema_key: 'character',
        name: 'Ada',
        sort_order: 3,
      },
      'command-graph-1',
    );
    expect(getBibleGraphNodeProjectionMock).not.toHaveBeenCalled();
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(storyState.entities).toBe(originalEntities);
  });

  it('records create errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);
    createBibleGraphNodeMock.mockRejectedValue(new Error('node conflict'));

    await expect(
      createBibleGraphNodeProjection({
        node_id: 'node.character/ada one',
        parent_id: null,
        schema_key: 'character',
        name: 'Ada Duplicate',
        sort_order: 3,
      }),
    ).rejects.toThrow('node conflict');

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('node conflict');
  });

  it('clears cached graph node projection state for one node', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);

    clearBibleGraphNodeProjection(key);

    expect(getCachedBibleGraphNodeProjection(key)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });
});
