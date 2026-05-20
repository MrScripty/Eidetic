import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import {
  clearSelectedNodeEditorProjection,
  getCachedSelectedNodeEditorProjection,
  refreshSelectedNodeEditorProjection,
  selectedNodeEditorProjectionState,
} from './selectedNodeEditorProjection.svelte.js';

vi.mock('$lib/projectionApi.js', () => ({
  getSelectedNodeEditorProjection: vi.fn(),
}));

const getSelectedNodeEditorProjectionMock = vi.mocked(getSelectedNodeEditorProjection);

const selectedProjection = {
  version: 3,
  change_event_id: 'event-selected-1',
  payload: {
    node: {
      node_id: 'node.scene.beach',
      parent_id: 'node.sequence.opening',
      level: 'Scene' as const,
      sort_order: 10,
      start_ms: 1_000,
      end_ms: 4_000,
      name: 'Beach argument',
      notes: 'Rainy',
      content_status: 'NotesOnly' as const,
      beat_type: null,
      locked: false,
    },
    child_level: 'Beat' as const,
    has_children: true,
    parent: null,
    siblings: [],
    current_sibling_index: 0,
    children: [
      {
        node_id: 'node.beat.first',
        parent_id: 'node.scene.beach',
        level: 'Beat' as const,
        sort_order: 1,
        start_ms: 1_000,
        end_ms: 2_000,
        name: 'First beat',
        notes: '',
        beat_type: null,
      },
    ],
    adjacent_parents: {},
  },
};

const emptyProjection = {
  version: 1,
  payload: {
    node: null,
    has_children: false,
    siblings: [],
    children: [],
    adjacent_parents: {},
  },
};

const newerProjection = {
  ...selectedProjection,
  version: 5,
  change_event_id: 'event-selected-5',
  payload: {
    ...selectedProjection.payload,
    node: {
      ...selectedProjection.payload.node,
      name: 'Newer beach argument',
    },
  },
};

const olderProjection = {
  ...selectedProjection,
  version: 4,
  change_event_id: 'event-selected-4',
  payload: {
    ...selectedProjection.payload,
    node: {
      ...selectedProjection.payload.node,
      name: 'Older beach argument',
    },
  },
};

beforeEach(() => {
  clearSelectedNodeEditorProjection();
  getSelectedNodeEditorProjectionMock.mockReset();
});

describe('selected node editor projection store', () => {
  it('stores backend selected-node editor projections keyed by node id', async () => {
    getSelectedNodeEditorProjectionMock.mockResolvedValue(selectedProjection);

    await expect(refreshSelectedNodeEditorProjection('node.scene.beach')).resolves.toEqual(
      selectedProjection,
    );

    expect(getSelectedNodeEditorProjectionMock).toHaveBeenCalledWith({
      node_id: 'node.scene.beach',
    });
    expect(getCachedSelectedNodeEditorProjection()).toEqual(selectedProjection);
    expect(selectedNodeEditorProjectionState.selectedNodeId).toBe('node.scene.beach');
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
    expect(selectedNodeEditorProjectionState.error).toBeUndefined();
  });

  it('loads empty backend projections for null selection', async () => {
    getSelectedNodeEditorProjectionMock.mockResolvedValue(emptyProjection);

    await expect(refreshSelectedNodeEditorProjection(null)).resolves.toEqual(emptyProjection);

    expect(getSelectedNodeEditorProjectionMock).toHaveBeenCalledWith({ node_id: null });
    expect(selectedNodeEditorProjectionState.selectedNodeId).toBeNull();
    expect(getCachedSelectedNodeEditorProjection()).toEqual(emptyProjection);
  });

  it('records read errors without replacing an existing projection', async () => {
    getSelectedNodeEditorProjectionMock.mockResolvedValue(selectedProjection);
    await refreshSelectedNodeEditorProjection('node.scene.beach');
    getSelectedNodeEditorProjectionMock.mockRejectedValue(new Error('selected node missing'));

    await expect(refreshSelectedNodeEditorProjection('node.scene.missing')).rejects.toThrow(
      'selected node missing',
    );

    expect(getCachedSelectedNodeEditorProjection()).toEqual(selectedProjection);
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
    expect(selectedNodeEditorProjectionState.error).toBe('selected node missing');
  });

  it('ignores stale responses from older refreshes', async () => {
    let resolveFirst: (value: typeof selectedProjection) => void = () => {};
    const firstRefresh = new Promise<typeof selectedProjection>((resolve) => {
      resolveFirst = resolve;
    });
    const secondProjection = {
      ...selectedProjection,
      payload: {
        ...selectedProjection.payload,
        node: {
          ...selectedProjection.payload.node,
          node_id: 'node.scene.second',
          name: 'Second scene',
        },
      },
    };
    getSelectedNodeEditorProjectionMock
      .mockReturnValueOnce(firstRefresh)
      .mockResolvedValueOnce(secondProjection);

    const first = refreshSelectedNodeEditorProjection('node.scene.beach');
    await refreshSelectedNodeEditorProjection('node.scene.second');
    resolveFirst(selectedProjection);
    await first;

    expect(getCachedSelectedNodeEditorProjection()).toEqual(secondProjection);
    expect(selectedNodeEditorProjectionState.selectedNodeId).toBe('node.scene.second');
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
  });

  it('does not replace cached selected-node projections with stale latest requests', async () => {
    getSelectedNodeEditorProjectionMock.mockResolvedValueOnce(newerProjection);
    await refreshSelectedNodeEditorProjection('node.scene.beach');
    getSelectedNodeEditorProjectionMock.mockResolvedValueOnce(olderProjection);

    await expect(refreshSelectedNodeEditorProjection('node.scene.beach')).resolves.toEqual(
      olderProjection,
    );

    expect(getCachedSelectedNodeEditorProjection()).toEqual(newerProjection);
    expect(selectedNodeEditorProjectionState.selectedNodeId).toBe('node.scene.beach');
    expect(selectedNodeEditorProjectionState.pending).toBe(false);
    expect(selectedNodeEditorProjectionState.error).toBeUndefined();
  });
});
