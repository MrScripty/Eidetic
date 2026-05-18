import type { BeatType, CommandOutcome, ProjectionEnvelope, StoryLevel, TimelineRenderProjection } from './types.js';

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

export interface TimelineCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<TimelineRenderProjection>;
}
