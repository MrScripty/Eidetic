import { beforeEach, describe, expect, it, vi } from 'vitest';

import {
  applyTimelineChildren,
  createTimelineNode,
  createTimelineRelationship,
  deleteTimelineNode,
  deleteTimelineRelationship,
  setTimelineNodeLock,
  setTimelineNodeNotes,
  setTimelineNodeRange,
  splitTimelineNode,
} from '$lib/commandApi.js';
import { getTimelineRenderProjection } from '$lib/projectionApi.js';
import {
  applyCreateTimelineNodeCommand,
  applyCreateTimelineRelationshipCommand,
  applyDeleteTimelineNodeCommand,
  applyDeleteTimelineRelationshipCommand,
  applySplitTimelineNodeCommand,
  applyTimelineChildrenCommand,
  applyTimelineNodeLockCommand,
  applyTimelineNodeNotesCommand,
  applyTimelineNodeRangeCommand,
  clearTimelineRenderProjection,
  getCachedTimelineRenderModel,
  getCachedTimelineRenderProjection,
  refreshTimelineRenderProjection,
  timelineRenderProjectionState,
} from './timelineRenderProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  applyTimelineChildren: vi.fn(),
  createTimelineNode: vi.fn(),
  createTimelineRelationship: vi.fn(),
  deleteTimelineNode: vi.fn(),
  deleteTimelineRelationship: vi.fn(),
  setTimelineNodeLock: vi.fn(),
  setTimelineNodeNotes: vi.fn(),
  setTimelineNodeRange: vi.fn(),
  splitTimelineNode: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getTimelineRenderProjection: vi.fn(),
}));

const applyTimelineChildrenMock = vi.mocked(applyTimelineChildren);
const createTimelineNodeMock = vi.mocked(createTimelineNode);
const createTimelineRelationshipMock = vi.mocked(createTimelineRelationship);
const deleteTimelineNodeMock = vi.mocked(deleteTimelineNode);
const deleteTimelineRelationshipMock = vi.mocked(deleteTimelineRelationship);
const setTimelineNodeLockMock = vi.mocked(setTimelineNodeLock);
const setTimelineNodeNotesMock = vi.mocked(setTimelineNodeNotes);
const setTimelineNodeRangeMock = vi.mocked(setTimelineNodeRange);
const splitTimelineNodeMock = vi.mocked(splitTimelineNode);
const getTimelineRenderProjectionMock = vi.mocked(getTimelineRenderProjection);

const projection = {
  version: 7,
  change_event_id: 'event-timeline-1',
  payload: {
    total_duration_ms: 120_000,
    tracks: [
      {
        track_id: 'track.scene',
        level: 'Scene' as const,
        label: 'Scenes',
        sort_order: 30,
        collapsed: false,
      },
    ],
    clips: [
      {
        node_id: 'node.scene.beach',
        parent_id: 'node.sequence.opening',
        track_id: 'track.scene',
        level: 'Scene' as const,
        name: 'Beach argument',
        start_ms: 1_000,
        end_ms: 4_000,
        sort_order: 10,
        locked: false,
        content_status: 'NotesOnly' as const,
        beat_type: null,
        arc_ids: ['arc.a'],
      },
    ],
    relationships: [
      {
        relationship_id: 'rel.theme',
        from_node_id: 'node.scene.beach',
        to_node_id: 'node.scene.beach',
        relationship_type: 'Thematic' as const,
      },
    ],
  },
};

beforeEach(() => {
  clearTimelineRenderProjection();
  applyTimelineChildrenMock.mockReset();
  createTimelineNodeMock.mockReset();
  createTimelineRelationshipMock.mockReset();
  deleteTimelineNodeMock.mockReset();
  deleteTimelineRelationshipMock.mockReset();
  setTimelineNodeLockMock.mockReset();
  setTimelineNodeNotesMock.mockReset();
  setTimelineNodeRangeMock.mockReset();
  splitTimelineNodeMock.mockReset();
  getTimelineRenderProjectionMock.mockReset();
});

describe('timeline render projection store', () => {
  it('stores backend timeline render projections and clears pending state', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);

    await expect(refreshTimelineRenderProjection()).resolves.toEqual(projection);

    expect(getTimelineRenderProjectionMock).toHaveBeenCalledWith();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('derives a render model from the cached backend projection', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();

    const model = getCachedTimelineRenderModel();

    expect(model?.duration_ms).toBe(120_000);
    expect(model?.tracks[0]?.clip_ids).toEqual(['timeline.clip.node.scene.beach']);
    expect(model?.clips[0]).toMatchObject({
      clip_id: 'timeline.clip.node.scene.beach',
      node_id: 'node.scene.beach',
      start_ratio: 1_000 / 120_000,
      end_ratio: 4_000 / 120_000,
      duration_ms: 3_000,
    });
    expect(model?.clip_ids_by_node_id['node.scene.beach']).toBe('timeline.clip.node.scene.beach');
  });

  it('records read errors without replacing an existing projection', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    getTimelineRenderProjectionMock.mockRejectedValue(new Error('timeline unavailable'));

    await expect(refreshTimelineRenderProjection()).rejects.toThrow('timeline unavailable');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('timeline unavailable');
  });

  it('clears cached projection state', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();

    clearTimelineRenderProjection();

    expect(getCachedTimelineRenderProjection()).toBeNull();
    expect(getCachedTimelineRenderModel()).toBeNull();
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('stores timeline command response projections without local patching', async () => {
    setTimelineNodeRangeMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyTimelineNodeRangeCommand(
        {
          node_id: 'node.scene.beach',
          start_ms: 1_000,
          end_ms: 4_000,
        },
        'command-timeline-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(setTimelineNodeRangeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
        start_ms: 1_000,
        end_ms: 4_000,
      },
      'command-timeline-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    setTimelineNodeRangeMock.mockRejectedValue(new Error('range invalid'));

    await expect(
      applyTimelineNodeRangeCommand({
        node_id: 'node.scene.beach',
        start_ms: 5_000,
        end_ms: 1_000,
      }),
    ).rejects.toThrow('range invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('range invalid');
  });

  it('stores create timeline command response projections without local patching', async () => {
    createTimelineNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyCreateTimelineNodeCommand(
        {
          node_id: 'node.scene.beach',
          parent_id: 'node.sequence.opening',
          level: 'Scene',
          name: 'Beach argument',
          start_ms: 1_000,
          end_ms: 4_000,
          beat_type: null,
        },
        'command-timeline-create-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(createTimelineNodeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
        parent_id: 'node.sequence.opening',
        level: 'Scene',
        name: 'Beach argument',
        start_ms: 1_000,
        end_ms: 4_000,
        beat_type: null,
      },
      'command-timeline-create-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records create timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    createTimelineNodeMock.mockRejectedValue(new Error('create invalid'));

    await expect(
      applyCreateTimelineNodeCommand({
        node_id: 'node.scene.new',
        parent_id: null,
        level: 'Scene',
        name: 'Parentless scene',
        start_ms: 1_000,
        end_ms: 4_000,
        beat_type: null,
      }),
    ).rejects.toThrow('create invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('create invalid');
  });

  it('stores apply children timeline command response projections without local patching', async () => {
    applyTimelineChildrenMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyTimelineChildrenCommand(
        {
          parent_id: 'node.sequence.opening',
          children: [
            {
              node_id: 'node.scene.beach',
              name: 'Beach argument',
              outline: 'Argue on the beach.',
              weight: 1,
              beat_type: null,
            },
          ],
        },
        'command-timeline-children-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(applyTimelineChildrenMock).toHaveBeenCalledWith(
      {
        parent_id: 'node.sequence.opening',
        children: [
          {
            node_id: 'node.scene.beach',
            name: 'Beach argument',
            outline: 'Argue on the beach.',
            weight: 1,
            beat_type: null,
          },
        ],
      },
      'command-timeline-children-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records apply children timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    applyTimelineChildrenMock.mockRejectedValue(new Error('children invalid'));

    await expect(
      applyTimelineChildrenCommand({
        parent_id: 'node.sequence.opening',
        children: [],
      }),
    ).rejects.toThrow('children invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('children invalid');
  });

  it('stores create relationship timeline command response projections without local patching', async () => {
    createTimelineRelationshipMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applyCreateTimelineRelationshipCommand(
        {
          relationship_id: 'rel.theme',
          from_node_id: 'node.scene.beach',
          to_node_id: 'node.scene.arrival',
          relationship_type: 'Thematic',
        },
        'command-timeline-relationship-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(createTimelineRelationshipMock).toHaveBeenCalledWith(
      {
        relationship_id: 'rel.theme',
        from_node_id: 'node.scene.beach',
        to_node_id: 'node.scene.arrival',
        relationship_type: 'Thematic',
      },
      'command-timeline-relationship-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records create relationship timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    createTimelineRelationshipMock.mockRejectedValue(new Error('relationship invalid'));

    await expect(
      applyCreateTimelineRelationshipCommand({
        relationship_id: 'rel.theme',
        from_node_id: 'node.scene.unknown',
        to_node_id: 'node.scene.arrival',
        relationship_type: 'Thematic',
      }),
    ).rejects.toThrow('relationship invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('relationship invalid');
  });

  it('stores delete relationship timeline command response projections without local patching', async () => {
    const emptyRelationshipProjection = {
      ...projection,
      version: 8,
      payload: {
        ...projection.payload,
        relationships: [],
      },
    };
    deleteTimelineRelationshipMock.mockResolvedValue({
      outcome: 'recorded',
      projection: emptyRelationshipProjection,
    });

    await expect(
      applyDeleteTimelineRelationshipCommand(
        {
          relationship_id: 'rel.theme',
        },
        'command-timeline-relationship-delete-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: emptyRelationshipProjection,
    });

    expect(deleteTimelineRelationshipMock).toHaveBeenCalledWith(
      {
        relationship_id: 'rel.theme',
      },
      'command-timeline-relationship-delete-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(emptyRelationshipProjection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records delete relationship timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    deleteTimelineRelationshipMock.mockRejectedValue(new Error('relationship delete invalid'));

    await expect(
      applyDeleteTimelineRelationshipCommand({
        relationship_id: 'rel.unknown',
      }),
    ).rejects.toThrow('relationship delete invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('relationship delete invalid');
  });

  it('stores node lock timeline command response projections without local patching', async () => {
    const lockedProjection = {
      ...projection,
      version: 8,
      payload: {
        ...projection.payload,
        clips: projection.payload.clips.map((clip) => ({ ...clip, locked: true })),
      },
    };
    setTimelineNodeLockMock.mockResolvedValue({
      outcome: 'recorded',
      projection: lockedProjection,
    });

    await expect(
      applyTimelineNodeLockCommand(
        {
          node_id: 'node.scene.beach',
          locked: true,
        },
        'command-timeline-lock-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: lockedProjection,
    });

    expect(setTimelineNodeLockMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
        locked: true,
      },
      'command-timeline-lock-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(lockedProjection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records node lock timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    setTimelineNodeLockMock.mockRejectedValue(new Error('lock invalid'));

    await expect(
      applyTimelineNodeLockCommand({
        node_id: 'node.scene.unknown',
        locked: true,
      }),
    ).rejects.toThrow('lock invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('lock invalid');
  });

  it('stores node notes timeline command response projections without local patching', async () => {
    const notesProjection = {
      ...projection,
      version: 8,
      payload: {
        ...projection.payload,
        clips: projection.payload.clips.map((clip) => ({
          ...clip,
          content_status: 'NotesOnly' as const,
        })),
      },
    };
    setTimelineNodeNotesMock.mockResolvedValue({
      outcome: 'recorded',
      projection: notesProjection,
    });

    await expect(
      applyTimelineNodeNotesCommand(
        {
          node_id: 'node.scene.beach',
          notes: 'New outline',
        },
        'command-timeline-notes-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: notesProjection,
    });

    expect(setTimelineNodeNotesMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
        notes: 'New outline',
      },
      'command-timeline-notes-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(notesProjection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records node notes timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    setTimelineNodeNotesMock.mockRejectedValue(new Error('notes invalid'));

    await expect(
      applyTimelineNodeNotesCommand({
        node_id: 'node.scene.unknown',
        notes: 'New outline',
      }),
    ).rejects.toThrow('notes invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('notes invalid');
  });

  it('stores split timeline command response projections without local patching', async () => {
    splitTimelineNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection,
    });

    await expect(
      applySplitTimelineNodeCommand(
        {
          node_id: 'node.scene.beach',
          at_ms: 2_500,
        },
        'command-timeline-split-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection,
    });

    expect(splitTimelineNodeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
        at_ms: 2_500,
      },
      'command-timeline-split-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records split timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    splitTimelineNodeMock.mockRejectedValue(new Error('split invalid'));

    await expect(
      applySplitTimelineNodeCommand({
        node_id: 'node.scene.beach',
        at_ms: 1_000,
      }),
    ).rejects.toThrow('split invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('split invalid');
  });

  it('stores delete timeline command response projections without local patching', async () => {
    const emptyProjection = {
      ...projection,
      version: 8,
      payload: {
        ...projection.payload,
        clips: [],
        relationships: [],
      },
    };
    deleteTimelineNodeMock.mockResolvedValue({
      outcome: 'recorded',
      projection: emptyProjection,
    });

    await expect(
      applyDeleteTimelineNodeCommand(
        {
          node_id: 'node.scene.beach',
        },
        'command-timeline-delete-1',
      ),
    ).resolves.toEqual({
      outcome: 'recorded',
      projection: emptyProjection,
    });

    expect(deleteTimelineNodeMock).toHaveBeenCalledWith(
      {
        node_id: 'node.scene.beach',
      },
      'command-timeline-delete-1',
    );
    expect(getTimelineRenderProjectionMock).not.toHaveBeenCalled();
    expect(getCachedTimelineRenderProjection()).toEqual(emptyProjection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBeUndefined();
  });

  it('records delete timeline command errors and leaves cached projections unchanged', async () => {
    getTimelineRenderProjectionMock.mockResolvedValue(projection);
    await refreshTimelineRenderProjection();
    deleteTimelineNodeMock.mockRejectedValue(new Error('delete invalid'));

    await expect(
      applyDeleteTimelineNodeCommand({
        node_id: 'node.scene.beach',
      }),
    ).rejects.toThrow('delete invalid');

    expect(getCachedTimelineRenderProjection()).toEqual(projection);
    expect(timelineRenderProjectionState.pending).toBe(false);
    expect(timelineRenderProjectionState.error).toBe('delete invalid');
  });
});
