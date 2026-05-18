import type {
  ArcId,
  CommandOutcome,
  ProjectionEnvelope,
} from './types.js';

export type { ArcId } from './types.js';

export interface StoryArc {
  id: ArcId;
  name: string;
  description: string;
  arc_type: ArcType;
  color: Color;
  parent_arc_id: ArcId | null;
}

export type ArcType = 'APlot' | 'BPlot' | 'CRunner' | { Custom: string };

export interface Color {
  r: number;
  g: number;
  b: number;
}

export type Severity = 'Warning' | 'Error';

export interface ProgressionIssue {
  severity: Severity;
  message: string;
}

export interface ArcProgression {
  arc_id: string;
  arc_name: string;
  node_count: number;
  has_setup: boolean;
  has_resolution: boolean;
  coverage_percent: number;
  longest_gap_ms: number;
  issues: ProgressionIssue[];
}

export interface StoryArcListProjection {
  arcs: StoryArc[];
}

export interface StoryArcProgressionProjection {
  progressions: ArcProgression[];
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

export function colorToHex(c: Color): string {
  const r = c.r.toString(16).padStart(2, '0');
  const g = c.g.toString(16).padStart(2, '0');
  const b = c.b.toString(16).padStart(2, '0');
  return `#${r}${g}${b}`;
}
