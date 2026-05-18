import type {
  BeatType,
  CommandOutcome,
  ProjectionEnvelope,
  RelationshipId,
  RelationshipType,
  StoryLevel,
  TimelineRenderProjection,
} from './types.js';

export interface SetTimelineNodeRangeCommand {
  node_id: string;
  start_ms: number;
  end_ms: number;
}

export interface SplitTimelineNodeCommand {
  node_id: string;
  at_ms: number;
}

export interface DeleteTimelineNodeCommand {
  node_id: string;
}

export interface CreateTimelineNodeCommand {
  node_id: string;
  parent_id: string | null;
  level: StoryLevel;
  name: string;
  start_ms: number;
  end_ms: number;
  beat_type: BeatType | null;
}

export interface ApplyTimelineChildrenCommand {
  parent_id: string;
  children: ApplyTimelineChildCommand[];
}

export interface ApplyTimelineChildCommand {
  node_id: string;
  name: string;
  outline: string;
  weight: number;
  beat_type: BeatType | null;
}

export interface CreateTimelineRelationshipCommand {
  relationship_id: RelationshipId;
  from_node_id: string;
  to_node_id: string;
  relationship_type: RelationshipType;
}

export interface DeleteTimelineRelationshipCommand {
  relationship_id: RelationshipId;
}

export interface TimelineCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<TimelineRenderProjection>;
}
