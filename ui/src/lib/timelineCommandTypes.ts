import type { CommandOutcome, ProjectionEnvelope, TimelineRenderProjection } from './types.js';

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

export interface TimelineCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<TimelineRenderProjection>;
}
