import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  createBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphEdge,
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
  createBibleGraphNodeProjection,
  ensureCanonicalBibleRootProjections,
  getCachedBibleGraphNodeListProjection,
  getBibleGraphNodeProjectionError,
  getCachedBibleGraphNodeProjection,
  isBibleGraphNodeProjectionPending,
  refreshBibleGraphNodeListProjection,
  refreshBibleGraphNodeProjection,
  setBibleGraphEdgeProjection,
  setBibleGraphFieldProjection,
} from './bibleGraphNodeProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  createBibleGraphNode: vi.fn(),
  ensureCanonicalBibleRoots: vi.fn(),
  setBibleGraphEdge: vi.fn(),
  setBibleGraphField: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphNodeListProjection: vi.fn(),
  getBibleGraphNodeProjection: vi.fn(),
}));

const createBibleGraphNodeMock = vi.mocked(createBibleGraphNode);
const ensureCanonicalBibleRootsMock = vi.mocked(ensureCanonicalBibleRoots);
const setBibleGraphEdgeMock = vi.mocked(setBibleGraphEdge);
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

const targetKey = {
  node_id: 'node.place.beach',
};

const targetProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  version: 2,
  change_event_id: 'event-target-1',
  payload: {
    node: {
      id: 'node.place.beach',
      parent_id: null,
      schema_key: 'place',
      name: 'Beach',
      system_owned: false,
      sort_order: 4,
    },
    parts: [],
    incoming_edges: [],
    outgoing_edges: [],
    snapshots: [],
  },
};

const edgeProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  version: 4,
  change_event_id: 'event-edge-1',
  payload: {
    ...projection.payload,
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
  setBibleGraphEdgeMock.mockReset();
  setBibleGraphFieldMock.mockReset();
  getBibleGraphNodeProjectionMock.mockReset();
  getBibleGraphNodeListProjectionMock.mockReset();
});

describe('bible graph node projection command cache writes', () => {
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

  it('stores edge command source projections and invalidates cached target projections', async () => {
    getBibleGraphNodeProjectionMock
      .mockResolvedValueOnce(projection)
      .mockResolvedValueOnce(targetProjection);
    await refreshBibleGraphNodeProjection(key);
    await refreshBibleGraphNodeProjection(targetKey);
    setBibleGraphEdgeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: edgeProjection,
    });

    await expect(
      setBibleGraphEdgeProjection(
        {
          edge_id: 'edge.ada.beach',
          from_node_id: 'node.character/ada one',
          to_node_id: 'node.place.beach',
          edge_kind: 'located_in',
          label: 'located in',
          directed: true,
          sort_order: 1,
        },
        'command-edge-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: edgeProjection,
    });

    expect(setBibleGraphEdgeMock).toHaveBeenCalledWith(
      {
        edge_id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      },
      'command-edge-1',
    );
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(edgeProjection);
    expect(getCachedBibleGraphNodeProjection(targetKey)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('records edge command errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);
    setBibleGraphEdgeMock.mockRejectedValue(new Error('edge rejected'));

    await expect(
      setBibleGraphEdgeProjection({
        edge_id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      }),
    ).rejects.toThrow('edge rejected');

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('edge rejected');
  });
});
