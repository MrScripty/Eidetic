import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  createBibleGraphNode,
  deleteBibleGraphEdge,
  deleteBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphEdge,
  setBibleGraphField,
  setBibleGraphNodeName,
  setBibleGraphSnapshotField,
} from '$lib/commandApi.js';
import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
} from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';
import type { BibleNodeDetailProjection } from '../bibleGraphTypes.js';
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
  deleteBibleGraphEdgeProjection,
  deleteBibleGraphNodeProjection,
  setBibleGraphEdgeProjection,
  setBibleGraphFieldProjection,
  setBibleGraphNodeNameProjection,
  setBibleGraphSnapshotFieldProjection,
} from './bibleGraphNodeProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  createBibleGraphNode: vi.fn(),
  deleteBibleGraphEdge: vi.fn(),
  deleteBibleGraphNode: vi.fn(),
  ensureCanonicalBibleRoots: vi.fn(),
  setBibleGraphEdge: vi.fn(),
  setBibleGraphField: vi.fn(),
  setBibleGraphNodeName: vi.fn(),
  setBibleGraphSnapshotField: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getBibleGraphNodeListProjection: vi.fn(),
  getBibleGraphNodeProjection: vi.fn(),
}));

const createBibleGraphNodeMock = vi.mocked(createBibleGraphNode);
const deleteBibleGraphEdgeMock = vi.mocked(deleteBibleGraphEdge);
const deleteBibleGraphNodeMock = vi.mocked(deleteBibleGraphNode);
const ensureCanonicalBibleRootsMock = vi.mocked(ensureCanonicalBibleRoots);
const setBibleGraphEdgeMock = vi.mocked(setBibleGraphEdge);
const setBibleGraphFieldMock = vi.mocked(setBibleGraphField);
const setBibleGraphNodeNameMock = vi.mocked(setBibleGraphNodeName);
const setBibleGraphSnapshotFieldMock = vi.mocked(setBibleGraphSnapshotField);
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

const emptyListProjection = {
  version: 3,
  change_event_id: 'event-delete-node-1',
  payload: {
    nodes: [],
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

const snapshotProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  version: 5,
  change_event_id: 'event-snapshot-1',
  payload: {
    ...projection.payload,
    snapshots: [
      {
        snapshot: {
          id: 'snapshot.character.ada.sequence-1',
          node_id: 'node.character/ada one',
          at_ms: 12000,
          label: 'Sequence 1 state',
          sort_order: 1,
        },
        fields: [
          {
            id: 'snapshot-field.character.status',
            snapshot_id: 'snapshot.character.ada.sequence-1',
            part_key: 'profile',
            part_name: 'Profile',
            field_key: 'tagline',
            value: { type: 'text', value: 'Rain-soaked' },
            sort_order: 2,
          },
        ],
      },
    ],
  },
};

const newerProjection: ProjectionEnvelope<BibleNodeDetailProjection> = {
  ...projection,
  version: 7,
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
  version: 6,
  change_event_id: 'event-older',
  payload: {
    ...projection.payload,
    node: {
      ...projection.payload.node,
      name: 'Ada older',
    },
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
  createBibleGraphNodeMock.mockReset();
  deleteBibleGraphEdgeMock.mockReset();
  deleteBibleGraphNodeMock.mockReset();
  ensureCanonicalBibleRootsMock.mockReset();
  setBibleGraphEdgeMock.mockReset();
  setBibleGraphFieldMock.mockReset();
  setBibleGraphNodeNameMock.mockReset();
  setBibleGraphSnapshotFieldMock.mockReset();
  getBibleGraphNodeProjectionMock.mockReset();
  getBibleGraphNodeListProjectionMock.mockReset();
});

describe('bible graph node projection command cache writes', () => {
  it('stores create command response projections', async () => {
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

  it('stores node delete list projections and clears deleted detail projections', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeListProjection();
    await refreshBibleGraphNodeProjection(key);
    deleteBibleGraphNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: emptyListProjection,
    });

    await expect(
      deleteBibleGraphNodeProjection('node.character/ada one', 'command-delete-node-1'),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: emptyListProjection,
    });

    expect(deleteBibleGraphNodeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.character/ada one',
      },
      'command-delete-node-1',
    );
    expect(getCachedBibleGraphNodeListProjection()).toEqual(emptyListProjection);
    expect(getCachedBibleGraphNodeProjection(key)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('records node delete errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeListProjection();
    await refreshBibleGraphNodeProjection(key);
    deleteBibleGraphNodeMock.mockRejectedValue(new Error('node delete rejected'));

    await expect(deleteBibleGraphNodeProjection('node.character/ada one')).rejects.toThrow(
      'node delete rejected',
    );

    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('node delete rejected');
    expect(bibleGraphNodeProjectionState.nodeListError).toBe('node delete rejected');
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

  it('stores node name command projections and invalidates cached node lists', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();
    setBibleGraphNodeNameMock.mockResolvedValue({
      outcome: 'recorded',
      projection: newerProjection,
    });

    await expect(
      setBibleGraphNodeNameProjection(
        {
          node_id: 'node.character/ada one',
          name: 'Ada newer',
        },
        'command-node-name-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: newerProjection,
    });

    expect(setBibleGraphNodeNameMock).toHaveBeenCalledWith(
      {
        node_id: 'node.character/ada one',
        name: 'Ada newer',
      },
      'command-node-name-1',
    );
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(newerProjection);
    expect(getCachedBibleGraphNodeListProjection()).toBeNull();
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

  it('does not replace cached graph node projections with stale field command responses', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshBibleGraphNodeProjection(key);
    setBibleGraphFieldMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      setBibleGraphFieldProjection({
        node_id: 'node.character/ada one',
        part_id: 'part.character.profile',
        part_key: 'profile',
        part_name: 'Profile',
        part_sort_order: 1,
        field_id: 'field.character.profile.summary',
        field_key: 'summary',
        value: { type: 'text', value: 'Older detail' },
        field_sort_order: 2,
      }),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(newerProjection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
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

  it('does not invalidate cached edge targets for stale edge command responses', async () => {
    getBibleGraphNodeProjectionMock
      .mockResolvedValueOnce(newerProjection)
      .mockResolvedValueOnce(targetProjection);
    await refreshBibleGraphNodeProjection(key);
    await refreshBibleGraphNodeProjection(targetKey);
    setBibleGraphEdgeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

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
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(newerProjection);
    expect(getCachedBibleGraphNodeProjection(targetKey)).toEqual(targetProjection);
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

  it('stores edge delete command source projections and invalidates cached target projections', async () => {
    getBibleGraphNodeProjectionMock
      .mockResolvedValueOnce(projection)
      .mockResolvedValueOnce(targetProjection);
    await refreshBibleGraphNodeProjection(key);
    await refreshBibleGraphNodeProjection(targetKey);
    deleteBibleGraphEdgeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: edgeProjection,
    });

    await expect(
      deleteBibleGraphEdgeProjection(
        {
          id: 'edge.ada.beach',
          from_node_id: 'node.character/ada one',
          to_node_id: 'node.place.beach',
          edge_kind: 'located_in',
          label: 'located in',
          directed: true,
          sort_order: 1,
        },
        'command-delete-edge-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: edgeProjection,
    });

    expect(deleteBibleGraphEdgeMock).toHaveBeenCalledWith(
      {
        edge_id: 'edge.ada.beach',
      },
      'command-delete-edge-1',
    );
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(edgeProjection);
    expect(getCachedBibleGraphNodeProjection(targetKey)).toBeUndefined();
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('does not invalidate cached edge targets for stale edge delete responses', async () => {
    getBibleGraphNodeProjectionMock
      .mockResolvedValueOnce(newerProjection)
      .mockResolvedValueOnce(targetProjection);
    await refreshBibleGraphNodeProjection(key);
    await refreshBibleGraphNodeProjection(targetKey);
    deleteBibleGraphEdgeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: olderProjection,
    });

    await expect(
      deleteBibleGraphEdgeProjection({
        id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      }),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: olderProjection,
    });

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(newerProjection);
    expect(getCachedBibleGraphNodeProjection(targetKey)).toEqual(targetProjection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('records edge delete command errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);
    deleteBibleGraphEdgeMock.mockRejectedValue(new Error('edge delete rejected'));

    await expect(
      deleteBibleGraphEdgeProjection({
        id: 'edge.ada.beach',
        from_node_id: 'node.character/ada one',
        to_node_id: 'node.place.beach',
        edge_kind: 'located_in',
        label: 'located in',
        directed: true,
        sort_order: 1,
      }),
    ).rejects.toThrow('edge delete rejected');

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('edge delete rejected');
  });

  it('stores snapshot field command response projections without invalidating cached node lists', async () => {
    getBibleGraphNodeListProjectionMock.mockResolvedValue(listProjection);
    await refreshBibleGraphNodeListProjection();
    setBibleGraphSnapshotFieldMock.mockResolvedValue({
      outcome: 'recorded',
      projection: snapshotProjection,
    });

    await expect(
      setBibleGraphSnapshotFieldProjection(
        {
          snapshot_id: 'snapshot.character.ada.sequence-1',
          node_id: 'node.character/ada one',
          at_ms: 12000,
          label: 'Sequence 1 state',
          snapshot_sort_order: 1,
          field_id: 'snapshot-field.character.status',
          part_key: 'profile',
          part_name: 'Profile',
          field_key: 'tagline',
          value: { type: 'text', value: 'Rain-soaked' },
          field_sort_order: 2,
        },
        'command-snapshot-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: snapshotProjection,
    });

    expect(setBibleGraphSnapshotFieldMock).toHaveBeenCalledWith(
      {
        snapshot_id: 'snapshot.character.ada.sequence-1',
        node_id: 'node.character/ada one',
        at_ms: 12000,
        label: 'Sequence 1 state',
        snapshot_sort_order: 1,
        field_id: 'snapshot-field.character.status',
        part_key: 'profile',
        part_name: 'Profile',
        field_key: 'tagline',
        value: { type: 'text', value: 'Rain-soaked' },
        field_sort_order: 2,
      },
      'command-snapshot-1',
    );
    expect(getCachedBibleGraphNodeProjection(key)).toEqual(snapshotProjection);
    expect(getCachedBibleGraphNodeListProjection()).toEqual(listProjection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBeUndefined();
  });

  it('records snapshot field command errors and leaves cached projections unchanged', async () => {
    getBibleGraphNodeProjectionMock.mockResolvedValue(projection);
    await refreshBibleGraphNodeProjection(key);
    setBibleGraphSnapshotFieldMock.mockRejectedValue(new Error('snapshot rejected'));

    await expect(
      setBibleGraphSnapshotFieldProjection({
        snapshot_id: 'snapshot.character.ada.sequence-1',
        node_id: 'node.character/ada one',
        at_ms: 12000,
        label: 'Sequence 1 state',
        snapshot_sort_order: 1,
        field_id: 'snapshot-field.character.status',
        part_key: 'profile',
        part_name: 'Profile',
        field_key: 'tagline',
        value: { type: 'text', value: 'Rain-soaked' },
        field_sort_order: 2,
      }),
    ).rejects.toThrow('snapshot rejected');

    expect(getCachedBibleGraphNodeProjection(key)).toEqual(projection);
    expect(isBibleGraphNodeProjectionPending(key)).toBe(false);
    expect(getBibleGraphNodeProjectionError(key)).toBe('snapshot rejected');
  });
});
