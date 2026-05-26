import type {
  BeatType,
  ContentStatus,
  NodeId,
  RelationshipId,
  RelationshipType,
  SegmentType,
  StoryLevel,
  TrackId,
  TimeRange,
} from './timelineTypes.js';
import type { ArcId } from './storyArcTypes.js';
import type { AffectProvenance, AffectValueId } from './affectTypes.js';

export interface TimelineRenderProjection {
  total_duration_ms: number;
  structure_segments?: TimelineRenderStructureSegment[];
  tracks: TimelineRenderTrack[];
  clips: TimelineRenderClip[];
  relationships: TimelineRenderRelationship[];
  gaps?: TimelineRenderGap[];
  affect_overlays?: TimelineRenderAffectSample[];
}

export interface TimelineRenderStructureSegment {
  segment_type: SegmentType;
  time_range: TimeRange;
  label: string;
}

export interface TimelineRenderTrack {
  track_id: TrackId;
  level: StoryLevel;
  label: string;
  sort_order: number;
  collapsed: boolean;
}

export interface TimelineRenderClip {
  node_id: NodeId;
  parent_id?: NodeId | null;
  track_id: TrackId;
  level: StoryLevel;
  name: string;
  start_ms: number;
  end_ms: number;
  sort_order: number;
  locked: boolean;
  content_status: ContentStatus;
  beat_type?: BeatType | null;
  arc_ids: ArcId[];
}

export interface TimelineRenderRelationship {
  relationship_id: RelationshipId;
  from_node_id: NodeId;
  to_node_id: NodeId;
  relationship_type: RelationshipType;
}

export interface TimelineRenderGap {
  level: StoryLevel;
  time_range: TimeRange;
  preceding_node_id?: NodeId | null;
  following_node_id?: NodeId | null;
}

export interface TimelineRenderAffectSample {
  affect_id: AffectValueId;
  node_id: NodeId;
  start_ms: number;
  end_ms: number;
  valence: number;
  arousal: number;
  intensity: number;
  confidence: number;
  mood_labels: string[];
  provenance: AffectProvenance;
}
