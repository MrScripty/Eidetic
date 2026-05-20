import type { BeatType, ContentStatus, NodeId, StoryLevel } from './timelineTypes.js';

export interface SelectedNodeEditorProjection {
  node?: SelectedNodeEditorNode | null;
  child_level?: StoryLevel | null;
  has_children: boolean;
  parent?: SelectedNodeEditorSummary | null;
  siblings: SelectedNodeEditorSummary[];
  current_sibling_index?: number | null;
  children: SelectedNodeEditorSummary[];
  adjacent_parents: SelectedNodeEditorAdjacentParents;
}

export interface SelectedNodeEditorNode {
  node_id: NodeId;
  parent_id?: NodeId | null;
  level: StoryLevel;
  sort_order: number;
  start_ms: number;
  end_ms: number;
  name: string;
  notes: string;
  content_status: ContentStatus;
  beat_type?: BeatType | null;
  locked: boolean;
}

export interface SelectedNodeEditorSummary {
  node_id: NodeId;
  parent_id?: NodeId | null;
  level: StoryLevel;
  sort_order: number;
  start_ms: number;
  end_ms: number;
  name: string;
  notes: string;
  beat_type?: BeatType | null;
}

export interface SelectedNodeEditorAdjacentParents {
  before?: SelectedNodeEditorSummary | null;
  after?: SelectedNodeEditorSummary | null;
}
