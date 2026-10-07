import type { CommandOutcome, ProjectionEnvelope } from './projectionTypes.js';

export type ScriptDocumentId = string;
export type ScriptSegmentId = string;
export type ScriptBlockId = string;
export type ScriptSpanId = string;
export type ScriptLockId = string;
export type ScriptPatchId = string;

export interface ScriptContextBlock {
  document_id: ScriptDocumentId;
  segment_id: ScriptSegmentId;
  block_id: ScriptBlockId;
  source_node_id: string | null;
  revision_event_id: string;
  segment_revision_event_id: string;
  start_ms: number;
  end_ms: number;
  text: string;
}

export interface ScriptDocument {
  id: ScriptDocumentId;
  title: string;
  sort_order: number;
}

export interface ScriptSegment {
  id: ScriptSegmentId;
  document_id: ScriptDocumentId;
  source_node_id?: string | null;
  start_ms: number;
  end_ms: number;
  status: ScriptSegmentStatus;
  sort_order: number;
}

export type ScriptSegmentStatus = 'current' | 'stale' | 'regenerating';

export interface ScriptBlock {
  id: ScriptBlockId;
  segment_id: ScriptSegmentId;
  block_kind: ScriptBlockKind;
  text: string;
  sort_order: number;
}

export type ScriptBlockKind =
  | 'scene_heading'
  | 'action'
  | 'character'
  | 'parenthetical'
  | 'dialogue'
  | 'transition'
  | 'shot'
  | 'note';

export interface ScriptSpan {
  id: ScriptSpanId;
  block_id: ScriptBlockId;
  start_byte: number;
  end_byte: number;
  provenance: ScriptSpanProvenance;
}

export type ScriptSpanProvenance = 'ai_generated' | 'user_edited' | 'imported' | 'system';

export interface ScriptLock {
  id: ScriptLockId;
  span_id: ScriptSpanId;
  reason: string;
}

export interface ScriptPatch {
  id: ScriptPatchId;
  document_id: ScriptDocumentId;
  segments: ScriptSegmentProjection[];
}

export interface ScriptSegmentProjection {
  segment: ScriptSegment;
  impact?: ScriptImpactProjection | null;
  blocks: ScriptBlockProjection[];
}

export interface ScriptImpactProjection {
  generation_event_id: string;
  output_block_id?: string | null;
  lineage_available: boolean;
  needs_review: boolean;
  causes: ScriptImpactCause[];
}

export interface ScriptImpactCause {
  dependency_id: string;
  input:
    | { kind: 'timeline_node'; node_id: string }
    | { kind: 'bible_node'; node_id: string }
    | { kind: 'bible_edge'; edge_id: string }
    | { kind: 'story_arc_field'; arc_id: string; field: 'name' | 'description' | 'arc_type' }
    | {
        kind: 'bible_field';
        node_id: string;
        part_key: string;
        field_key: string;
        field_id?: string | null;
      }
    | { kind: 'script_block'; block_id: string }
    | { kind: 'script_segment'; segment_id: string };
  consumed_revision_event_id: string;
  current_revision_event_id: string | null;
  reason: 'changed' | 'deleted' | 'context_changed';
  input_excerpt: string | null;
}

export interface ScriptBlockProjection {
  block: ScriptBlock;
  revision_event_id?: string | null;
  spans: ScriptSpan[];
  locks: ScriptLock[];
}

export interface ScriptDocumentProjection {
  document: ScriptDocument;
  segments: ScriptSegmentProjection[];
}

export interface SetScriptBlockCommand {
  document_id: ScriptDocumentId;
  document_title: string;
  document_sort_order?: number;
  segment_id: ScriptSegmentId;
  source_node_id?: string | null;
  segment_start_ms: number;
  segment_end_ms: number;
  segment_status: ScriptSegmentStatus;
  segment_sort_order?: number;
  block_id: ScriptBlockId;
  block_kind: ScriptBlockKind;
  text: string;
  span_provenance?: ScriptSpanProvenance;
  sort_order?: number;
}

export interface SetScriptLockCommand {
  lock_id: ScriptLockId;
  span_id: ScriptSpanId;
  reason: string;
}

export interface EditScriptBlockCommand {
  document_id: ScriptDocumentId;
  block_id: ScriptBlockId;
  expected_revision_event_id: string;
  text: string;
}

export interface ScriptDocumentCommandResponse {
  outcome: CommandOutcome;
  projection: ProjectionEnvelope<ScriptDocumentProjection>;
}

export interface CreateScriptBlockCommand {
  document_id: ScriptDocumentId;
  source_node_id: string;
  expected_start_ms: number;
  expected_end_ms: number;
  block_kind: ScriptBlockKind;
  text: string;
}
