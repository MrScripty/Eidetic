import type { CommandOutcome, ProjectionEnvelope, TimelineRenderProjection } from './types.js';

export interface SetTimelineNodeRangeCommand {
  node_id: string;
  start_ms: number;
  end_ms: number;
}

export interface TimelineCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<TimelineRenderProjection>;
}
