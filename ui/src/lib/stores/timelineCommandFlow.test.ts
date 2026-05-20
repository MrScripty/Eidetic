import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  applyTimelineNodeLockCommand,
  clearTimelineRenderProjection,
  getCachedTimelineRenderProjection,
  refreshTimelineRenderProjection,
} from './timelineRenderProjection.svelte.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { TimelineRenderProjection } from '$lib/timelineRenderTypes.js';

const unlockedProjection: ProjectionEnvelope<TimelineRenderProjection> = {
  version: 7,
  change_event_id: 'event.timeline.unlocked',
  payload: {
    total_duration_ms: 120_000,
    tracks: [
      {
        track_id: 'track.scene',
        level: 'Scene',
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
        level: 'Scene',
        name: 'Beach argument',
        start_ms: 1_000,
        end_ms: 4_000,
        sort_order: 10,
        locked: false,
        content_status: 'NotesOnly',
        beat_type: null,
        arc_ids: [],
      },
    ],
    relationships: [],
  },
};

const lockedProjection: ProjectionEnvelope<TimelineRenderProjection> = {
  ...unlockedProjection,
  version: 8,
  change_event_id: 'event.timeline.locked',
  payload: {
    ...unlockedProjection.payload,
    clips: unlockedProjection.payload.clips.map((clip) => ({ ...clip, locked: true })),
  },
};

afterEach(() => {
  vi.unstubAllGlobals();
});

beforeEach(() => {
  clearTimelineRenderProjection();
});

describe('timeline command projection flow', () => {
  it('sends a user-visible lock command through the command API and caches the returned projection', async () => {
    const response = {
      outcome: 'recorded',
      projection: lockedProjection,
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      applyTimelineNodeLockCommand(
        {
          node_id: 'node.scene.beach',
          locked: true,
        },
        'command-lock-visible-path',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/node-lock',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-lock-visible-path',
          payload: {
            node_id: 'node.scene.beach',
            locked: true,
          },
        }),
      }),
    );
    expect(getCachedTimelineRenderProjection()).toEqual(lockedProjection);
  });

  it('preserves the last confirmed projection when backend validation rejects the command', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(
        new Response(JSON.stringify(unlockedProjection), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      )
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ error: 'timeline node not found' }), {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }),
      );
    vi.stubGlobal('fetch', fetchMock);
    await refreshTimelineRenderProjection();

    await expect(
      applyTimelineNodeLockCommand(
        {
          node_id: 'node.scene.missing',
          locked: true,
        },
        'command-lock-invalid-path',
      ),
    ).rejects.toThrow('timeline node not found');

    expect(getCachedTimelineRenderProjection()).toEqual(unlockedProjection);
  });
});
