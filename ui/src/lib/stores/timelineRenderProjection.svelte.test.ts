import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setTimelineNodeRange, splitTimelineNode } from '$lib/commandApi.js';
import { getTimelineRenderProjection } from '$lib/projectionApi.js';
import {
  applySplitTimelineNodeCommand,
  applyTimelineNodeRangeCommand,
  clearTimelineRenderProjection,
  getCachedTimelineRenderProjection,
  refreshTimelineRenderProjection,
  timelineRenderProjectionState,
} from './timelineRenderProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  setTimelineNodeRange: vi.fn(),
  splitTimelineNode: vi.fn(),
}));

vi.mock('$lib/projectionApi.js', () => ({
  getTimelineRenderProjection: vi.fn(),
}));

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
});
