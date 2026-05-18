import type { BeatType, NodeId, StoryLevel } from './timelineTypes.js';

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
  parent_node_id: NodeId;
  target_child_level: StoryLevel;
  children: ChildProposal[];
}
