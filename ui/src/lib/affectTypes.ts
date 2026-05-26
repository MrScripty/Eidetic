import type { BibleGraphNodeId, BibleGraphSnapshotId } from './bibleGraphTypes.js';
import type { ProjectionEnvelope } from './projectionTypes.js';
import type { ScriptSegmentId } from './scriptTypes.js';
import type { NodeId } from './timelineTypes.js';

export type AffectValueId = string;

export type AffectTarget =
  | { type: 'project' }
  | { type: 'timeline_node'; node_id: NodeId }
  | { type: 'script_segment'; segment_id: ScriptSegmentId }
  | { type: 'bible_node'; node_id: BibleGraphNodeId }
  | { type: 'bible_snapshot'; snapshot_id: BibleGraphSnapshotId };

export type AffectProvenance =
  | 'user_authored'
  | 'agent_proposed'
  | 'script_edit_detected'
  | 'imported';

export interface AffectValue {
  id: AffectValueId;
  target: AffectTarget;
  valence: number;
  arousal: number;
  intensity: number;
  confidence: number;
  mood_labels: string[];
  provenance: AffectProvenance;
  rationale?: string | null;
}

export interface AffectProjection {
  target: AffectTarget;
  values: AffectValue[];
}

export interface SetAffectValueCommand {
  command_id: string;
  affect_id: AffectValueId;
  target: AffectTarget;
  valence: number;
  arousal: number;
  intensity: number;
  confidence: number;
  mood_labels: string[];
  provenance: AffectProvenance;
  rationale?: string | null;
}

export type SetAffectValueInput = Omit<SetAffectValueCommand, 'command_id'>;

export interface AffectProjectionRequest {
  target: AffectTarget;
}

export type AffectCommandResponse = ProjectionEnvelope<AffectProjection>;
