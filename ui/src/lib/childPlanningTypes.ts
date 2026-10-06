import type { BeatType, NodeId, StoryLevel } from './timelineTypes.js';
import type { ScriptContextBlock } from './scriptTypes.js';

export interface ChildProposal {
  name: string;
  beat_type: BeatType | null;
  outline: string;
  weight: number;
  characters?: string[];
  location?: string | null;
  props?: string[];
}

export interface ChildPlan {
  id: string;
  parent_node_id: NodeId;
  target_child_level: StoryLevel;
  children: ChildProposal[];
  script_context?: ScriptContextBlock[] | null;
}

export interface ChildPlanRecord {
  plan: ChildPlan;
  status: 'pending' | 'applied' | 'rejected';
  created_at_ms: number;
}

export interface ChildPlanListProjection {
  plans: ChildPlanRecord[];
}
