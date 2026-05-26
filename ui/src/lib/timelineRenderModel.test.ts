import { describe, expect, it } from 'vitest';

import {
  adjacentTimelineRenderClipBounds,
  findTimelineRenderClipByNodeId,
  timelineRenderModelFromProjection,
  timelineRenderClipsByLevel,
  timelineRenderClipsByTrackId,
  timelineRenderTrackIndexForClip,
  visibleTimelineRenderTracks,
} from './timelineRenderModel.js';
import type { TimelineRenderProjection } from './timelineRenderTypes.js';

describe('timeline render model', () => {
  it('derives sorted clip indexes and normalized timing from backend projections', () => {
    const projection: TimelineRenderProjection = {
      total_duration_ms: 10_000,
      structure_segments: [
        {
          segment_type: 'Act',
          time_range: { start_ms: 0, end_ms: 10_000 },
          label: 'Act One',
        },
      ],
      tracks: [
        {
          track_id: 'track.beat',
          level: 'Beat',
          label: 'Beats',
          sort_order: 50,
          collapsed: false,
        },
        {
          track_id: 'track.scene',
          level: 'Scene',
          label: 'Scenes',
          sort_order: 40,
          collapsed: false,
        },
      ],
      clips: [
        {
          node_id: 'node.beat.two',
          parent_id: 'node.scene.one',
          track_id: 'track.beat',
          level: 'Beat',
          name: 'Second beat',
          start_ms: 5_000,
          end_ms: 9_000,
          sort_order: 20,
          locked: false,
          content_status: 'NotesOnly',
          beat_type: null,
          arc_ids: [],
        },
        {
          node_id: 'node.beat.one',
          parent_id: 'node.scene.one',
          track_id: 'track.beat',
          level: 'Beat',
          name: 'First beat',
          start_ms: 1_000,
          end_ms: 3_000,
          sort_order: 10,
          locked: false,
          content_status: 'HasContent',
          beat_type: 'Setup',
          arc_ids: ['arc.a'],
        },
      ],
      relationships: [
        {
          relationship_id: 'rel.causal',
          from_node_id: 'node.beat.one',
          to_node_id: 'node.beat.two',
          relationship_type: 'Causal',
        },
      ],
      gaps: [
        {
          level: 'Beat',
          time_range: { start_ms: 3_000, end_ms: 5_000 },
          preceding_node_id: 'node.beat.one',
          following_node_id: 'node.beat.two',
        },
      ],
      affect_overlays: [
        {
          affect_id: 'affect.beat.one',
          node_id: 'node.beat.one',
          start_ms: 1_000,
          end_ms: 3_000,
          valence: -250,
          arousal: 650,
          intensity: 700,
          confidence: 900,
          mood_labels: ['uneasy'],
          provenance: 'user_authored',
        },
      ],
    };

    const model = timelineRenderModelFromProjection(projection);

    expect(model.tracks.map((track) => track.track_id)).toEqual(['track.scene', 'track.beat']);
    expect(model.clips.map((clip) => clip.node_id)).toEqual(['node.beat.one', 'node.beat.two']);
    expect(model.clips[0]).toMatchObject({
      clip_id: 'timeline.clip.node.beat.one',
      start_ratio: 0.1,
      end_ratio: 0.3,
      duration_ms: 2_000,
    });
    expect(model.clip_ids_by_track_id['track.beat']).toEqual([
      'timeline.clip.node.beat.one',
      'timeline.clip.node.beat.two',
    ]);
    expect(model.clip_ids_by_node_id['node.beat.two']).toBe('timeline.clip.node.beat.two');
    expect(model.structure_segments).toEqual(projection.structure_segments);
    expect(model.relationships).toEqual(projection.relationships);
    expect(model.gaps).toEqual(projection.gaps);
    expect(model.affect_overlays).toEqual(projection.affect_overlays);
    expect(visibleTimelineRenderTracks(model).map((track) => track.track_id)).toEqual([
      'track.scene',
      'track.beat',
    ]);
    expect(timelineRenderClipsByTrackId(model, 'track.beat').map((clip) => clip.node_id)).toEqual([
      'node.beat.one',
      'node.beat.two',
    ]);
    expect(timelineRenderClipsByLevel(model, 'Beat').map((clip) => clip.node_id)).toEqual([
      'node.beat.one',
      'node.beat.two',
    ]);
    expect(findTimelineRenderClipByNodeId(model, 'node.beat.one')?.clip_id).toBe(
      'timeline.clip.node.beat.one',
    );
    const firstClip = model.clips[0];
    expect(firstClip).toBeDefined();
    expect(timelineRenderTrackIndexForClip(model, firstClip!)).toBe(1);
    expect(adjacentTimelineRenderClipBounds(model, firstClip!)).toEqual({
      left_ms: 0,
      right_ms: 5_000,
    });
    expect(findTimelineRenderClipByNodeId(model, 'node.missing')).toBeNull();
  });

  it('excludes collapsed tracks from visible track selectors', () => {
    const projection: TimelineRenderProjection = {
      total_duration_ms: 10_000,
      tracks: [
        {
          track_id: 'track.scene',
          level: 'Scene',
          label: 'Scenes',
          sort_order: 10,
          collapsed: true,
        },
        {
          track_id: 'track.beat',
          level: 'Beat',
          label: 'Beats',
          sort_order: 20,
          collapsed: false,
        },
      ],
      clips: [],
      relationships: [],
    };

    const model = timelineRenderModelFromProjection(projection);

    expect(visibleTimelineRenderTracks(model).map((track) => track.track_id)).toEqual([
      'track.beat',
    ]);
  });

  it('clamps malformed timing without mutating the source projection', () => {
    const projection: TimelineRenderProjection = {
      total_duration_ms: 0,
      tracks: [],
      clips: [
        {
          node_id: 'node.beat.invalid',
          parent_id: null,
          track_id: 'track.beat',
          level: 'Beat',
          name: 'Invalid beat',
          start_ms: -100,
          end_ms: -200,
          sort_order: 1,
          locked: false,
          content_status: 'Empty',
          beat_type: null,
          arc_ids: [],
        },
      ],
      relationships: [],
    };

    const model = timelineRenderModelFromProjection(projection);
    const clip = model.clips[0];

    expect(model.duration_ms).toBe(0);
    expect(clip).toBeDefined();
    expect(clip?.start_ratio).toBe(0);
    expect(clip?.end_ratio).toBe(0);
    expect(clip?.duration_ms).toBe(0);
    expect(projection.clips[0]?.start_ms).toBe(-100);
  });
});
