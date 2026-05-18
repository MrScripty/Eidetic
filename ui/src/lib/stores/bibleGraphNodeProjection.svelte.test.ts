import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  createBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphField,
} from '$lib/commandApi.js';
import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
} from '$lib/projectionApi.js';
import { storyState } from './story.svelte.js';
import type { BibleNodeDetailProjection, ProjectionEnvelope } from '../types.js';
import {
  bibleGraphNodeProjectionState,
  clearBibleGraphNodeListProjection,
  clearBibleGraphNodeProjection,
  createBibleGraphNodeProjection,
  ensureCanonicalBibleRootProjections,
  getCachedBibleGraphNodeListProjection,
  getBibleGraphNodeProjectionError,
  getCachedBibleGraphNodeProjection,
  isBibleGraphNodeProjectionPending,
  refreshBibleGraphNodeListProjection,
  refreshBibleGraphNodeProjection,
  setBibleGraphFieldProjection,
} from './bibleGraphNodeProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  createBibleGraphNode: vi.fn(),
  ensureCanonicalBibleRoots: vi.fn(),
  setBibleGraphField: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphNodeListProjection: vi.fn(),
  getBibleGraphNodeProjection: vi.fn(),
}));

const createBibleGraphNodeMock = vi.mocked(createBibleGraphNode);
const ensureCanonicalBibleRootsMock = vi.mocked(ensureCanonicalBibleRoots);
const setBibleGraphFieldMock = vi.mocked(setBibleGraphField);
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
  },
};

const listProjection = {
  version: 2,
  change_event_id: 'event-1',
  payload: {
    nodes: [projection.payload.node],
  },
};

const fieldProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  version: 3,
  change_event_id: 'event-field-1',
  payload: {
    ...projection.payload,
    parts: [
      {
        part: {
          id: 'part.character.profile',
          node_id: 'node.character/ada one',
          part_key: 'profile',
          name: 'Profile',
          system_owned: false,
          sort_order: 1,
        },
        fields: [
          {
            id: 'field.character.profile.summary',
            part_id: 'part.character.profile',
            field_key: 'summary',
            value: { type: 'text', value: 'A precise engineer.' },
            sort_order: 2,
          },
        ],
      },
    ],
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
  storyState.entities = [];
  createBibleGraphNodeMock.mockReset();
  ensureCanonicalBibleRootsMock.mockReset();
  setBibleGraphFieldMock.mockReset();
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

  it('invalidates cached node list projections after create commands', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();
    createBibleGraphNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await createBibleGraphNodeProjection({
      node_id: 'node.character/ada one',
      parent_id: null,
      schema_key: 'character',
      name: 'Ada',
      sort_order: 3,
    });

    expect(getCachedBibleGraphNodeListProjection()).toBeNull();
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
  });

  it('stores canonical root command response node list projections', async () => {
    ensureCanonicalBibleRootsMock.mockResolvedValue({
      outcome: 'recorded',
      projection: listProjection,
    });

    await expect(ensureCanonicalBibleRootProjections('command-roots-1')).resolves.toEqual({
      outcome: 'recorded',
      projection: listProjection,
    });

    expect(ensureCanonicalBibleRootsMock).toHaveBeenCalledWith('command-roots-1');
    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBeUndefined();
  });

  it('records canonical root command errors without replacing cached lists', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();
    ensureCanonicalBibleRootsMock.mockRejectedValue(new Error('root command failed'));

    await expect(ensureCanonicalBibleRootProjections()).rejects.toThrow('root command failed');

    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(bibleGraphNodeProjectionState.nodeListPending).toBe(false);
    expect(bibleGraphNodeProjectionState.nodeListError).toBe('root command failed');
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

  it('stores field command response projections without invalidating cached node lists', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();
    setBibleGraphFieldMock.mockResolvedValue({
      outcome: 'recorded',
      projection: fieldProjection,
    });

    await expect(
      setBibleGraphFieldProjection(
        {
          node_id: 'node.character/ada one',
          part_id: 'part.character.profile',
          part_key: 'profile',
          part_name: 'Profile',
          part_sort_order: 1,
          field_id: 'field.character.profile.summary',
          field_key: 'summary',
          value: { type: 'text', value: 'A precise engineer.' },
          field_sort_order: 2,
        },
        'command-field-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: fieldProjection,
    });

    expect(setBibleGraphFieldMock).toHaveBeenCalledWith(
      {
        node_id: 'node.character/ada one',
        part_id: 'part.character.profile',
        part_key: 'profile',
        part_name: 'Profile',
        part_sort_order: 1,
        field_id: 'field.character.profile.summary',
        field_key: 'summary',
        value: { type: 'text', value: 'A precise engineer.' },
        field_sort_order: 2,
      },
      'command-field-1',
    );
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(fieldProjection);
    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('records field command errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);
    setBibleGraphFieldMock.mockRejectedValue(new Error('field rejected'));

    await expect(
      setBibleGraphFieldProjection({
        node_id: 'node.character/ada one',
        part_id: 'part.character.profile',
        part_key: 'profile',
        part_name: 'Profile',
        part_sort_order: 1,
        field_id: 'field.character.profile.summary',
        field_key: 'summary',
        value: { type: 'text', value: 'A precise engineer.' },
        field_sort_order: 2,
      }),
    ).rejects.toThrow('field rejected');

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('field rejected');
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
