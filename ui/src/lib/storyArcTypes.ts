import type {
  ArcId,
  ArcType,
  Color,
  CommandOutcome,
  ProjectionEnvelope,
  StoryArc,
} from './types.js';

export type { ArcId, ArcType, Color, StoryArc } from './types.js';

export interface StoryArcListProjection {
  arcs: StoryArc[];
}

export interface CreateStoryArcCommand {
  arc_id: ArcId;
  parent_arc_id?: ArcId | null;
  name: string;
  description?: string;
  arc_type: ArcType;
  color: Color;
}

export interface SetStoryArcMetadataCommand {
  arc_id: ArcId;
  name?: string;
  description?: string;
  arc_type?: ArcType;
  color?: Color;
}

export interface DeleteStoryArcCommand {
  arc_id: ArcId;
}

export interface StoryArcCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<StoryArcListProjection>;
}
