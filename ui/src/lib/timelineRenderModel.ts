import type {
  TimelineRenderClip,
  TimelineRenderProjection,
  TimelineRenderRelationship,
  TimelineRenderTrack,
} from './timelineRenderTypes.js';

export interface TimelineRenderModel {
  duration_ms: number;
  tracks: TimelineRenderModelTrack[];
  clips: TimelineRenderModelClip[];
  relationships: TimelineRenderRelationship[];
  clip_ids_by_track_id: Record<string, string[]>;
  clip_ids_by_node_id: Record<string, string>;
}

export interface TimelineRenderModelTrack extends TimelineRenderTrack {
  clip_ids: string[];
}

export interface TimelineRenderModelClip extends TimelineRenderClip {
  clip_id: string;
  start_ratio: number;
  end_ratio: number;
  duration_ms: number;
}

export function timelineRenderModelFromProjection(
  projection: TimelineRenderProjection,
): TimelineRenderModel {
  const durationMs = Math.max(projection.total_duration_ms, 1);
  const clipIdsByTrackId: Record<string, string[]> = {};
  const clipIdsByNodeId: Record<string, string> = {};

  const tracks = projection.tracks
    .slice()
    .sort((left, right) => left.sort_order - right.sort_order)
    .map((track) => {
      const clipIds: string[] = [];
      clipIdsByTrackId[track.track_id] = clipIds;
      return {
        ...track,
        clip_ids: clipIds,
      };
    });

  const clips = projection.clips
    .slice()
    .sort((left, right) => {
      if (left.start_ms !== right.start_ms) {
        return left.start_ms - right.start_ms;
      }
      if (left.sort_order !== right.sort_order) {
        return left.sort_order - right.sort_order;
      }
      return left.node_id.localeCompare(right.node_id);
    })
    .map((clip) => {
      const startMs = clampTime(clip.start_ms, durationMs);
      const endMs = clampTime(Math.max(clip.end_ms, clip.start_ms), durationMs);
      const clipId = `timeline.clip.${clip.node_id}`;
      const modelClip = {
        ...clip,
        clip_id: clipId,
        start_ratio: startMs / durationMs,
        end_ratio: endMs / durationMs,
        duration_ms: endMs - startMs,
      };

      clipIdsByNodeId[clip.node_id] = clipId;
      let trackClipIds = clipIdsByTrackId[clip.track_id];
      if (!trackClipIds) {
        trackClipIds = [];
        clipIdsByTrackId[clip.track_id] = trackClipIds;
      }
      trackClipIds.push(clipId);

      return modelClip;
    });

  return {
    duration_ms: projection.total_duration_ms,
    tracks,
    clips,
    relationships: projection.relationships.slice(),
    clip_ids_by_track_id: clipIdsByTrackId,
    clip_ids_by_node_id: clipIdsByNodeId,
  };
}

export function findTimelineRenderClipByNodeId(
  model: TimelineRenderModel,
  nodeId: string,
): TimelineRenderModelClip | null {
  const clipId = model.clip_ids_by_node_id[nodeId];
  if (!clipId) return null;
  return model.clips.find((clip) => clip.clip_id === clipId) ?? null;
}

export function timelineRenderTrackIndexForClip(
  model: TimelineRenderModel,
  clip: TimelineRenderModelClip,
): number {
  return model.tracks.findIndex((track) => track.track_id === clip.track_id);
}

function clampTime(value: number, durationMs: number): number {
  return Math.min(Math.max(value, 0), durationMs);
}
