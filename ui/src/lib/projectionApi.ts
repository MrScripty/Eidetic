import type {
  BibleGraphNodeId,
  BibleGraphNodeListProjection,
  BibleRenderGraphProjection,
  BibleRenderGraphProjectionRequest,
  BibleNodeDetailProjection,
} from './bibleGraphTypes.js';
import type { BibleGraphSchemaListProjection } from './bibleGraphSchemaTypes.js';
import type { ChangeReviewProjection } from './changeReviewTypes.js';
import type {
  ContextInfluenceProjection,
  ContextInfluenceProjectionRequest,
  ContextStackProjection,
  ContextStackProjectionRequest,
} from './contextInfluenceTypes.js';
import type { ObjectFieldProjection, ObjectKind, ProjectionEnvelope } from './projectionTypes.js';
import type { PropagationProposalListProjection } from './propagationProposalTypes.js';
import type { ScriptDocumentId, ScriptDocumentProjection } from './scriptTypes.js';
import type { SelectedNodeEditorProjection } from './selectedNodeEditorTypes.js';
import type { BibleReferenceProposalListProjection } from './semanticProposalTypes.js';
import type { StoryArcListProjection, StoryArcProgressionProjection } from './storyArcTypes.js';
import type { NodeId } from './timelineTypes.js';
import type { TimelineRenderProjection } from './timelineRenderTypes.js';
import { invokeDesktop } from './desktopTransport.js';

export interface ObjectFieldProjectionKey {
  object_kind: ObjectKind;
  object_id: string;
}

export interface BibleGraphNodeProjectionKey {
  node_id: BibleGraphNodeId;
}

export interface ScriptDocumentProjectionKey {
  document_id: ScriptDocumentId;
}

export interface SelectedNodeEditorProjectionKey {
  node_id?: NodeId | null;
}

export function getObjectFieldProjection({
  object_kind,
  object_id,
}: ObjectFieldProjectionKey): Promise<ProjectionEnvelope<ObjectFieldProjection>> {
  return invokeDesktop<ProjectionEnvelope<ObjectFieldProjection>>('projection_object_field', {
    query: { object_kind, object_id },
  });
}

export function getBibleGraphNodeProjection({
  node_id,
}: BibleGraphNodeProjectionKey): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  return invokeDesktop<ProjectionEnvelope<BibleNodeDetailProjection>>(
    'projection_bible_graph_node',
    {
      query: { node_id },
    },
  );
}

export function getBibleGraphNodeListProjection(): Promise<
  ProjectionEnvelope<BibleGraphNodeListProjection>
> {
  return invokeDesktop<ProjectionEnvelope<BibleGraphNodeListProjection>>(
    'projection_bible_graph_nodes',
  );
}

export function getBibleGraphSchemaListProjection(): Promise<
  ProjectionEnvelope<BibleGraphSchemaListProjection>
> {
  return invokeDesktop<ProjectionEnvelope<BibleGraphSchemaListProjection>>(
    'projection_bible_graph_schemas',
  );
}

export function getBibleRenderGraphProjection(
  query?: BibleRenderGraphProjectionRequest,
): Promise<ProjectionEnvelope<BibleRenderGraphProjection>> {
  const args = query === undefined ? undefined : { query };
  return invokeDesktop<ProjectionEnvelope<BibleRenderGraphProjection>>(
    'projection_bible_render_graph',
    args,
  );
}

export function getContextInfluenceProjection({
  target_node_id,
}: ContextInfluenceProjectionRequest): Promise<ProjectionEnvelope<ContextInfluenceProjection>> {
  return invokeDesktop<ProjectionEnvelope<ContextInfluenceProjection>>(
    'projection_context_influence',
    {
      query: { target_node_id },
    },
  );
}

export function getContextStackProjection({
  target_node_id,
}: ContextStackProjectionRequest): Promise<ProjectionEnvelope<ContextStackProjection>> {
  return invokeDesktop<ProjectionEnvelope<ContextStackProjection>>('projection_context_stack', {
    query: { target_node_id },
  });
}

export function getScriptDocumentProjection({
  document_id,
}: ScriptDocumentProjectionKey): Promise<ProjectionEnvelope<ScriptDocumentProjection>> {
  return invokeDesktop<ProjectionEnvelope<ScriptDocumentProjection>>('projection_script_document', {
    query: { document_id },
  });
}

export function getBibleReferenceProposalListProjection(): Promise<
  ProjectionEnvelope<BibleReferenceProposalListProjection>
> {
  return invokeDesktop<ProjectionEnvelope<BibleReferenceProposalListProjection>>(
    'projection_bible_reference_proposals',
  );
}

export function getPropagationProposalListProjection(): Promise<
  ProjectionEnvelope<PropagationProposalListProjection>
> {
  return invokeDesktop<ProjectionEnvelope<PropagationProposalListProjection>>(
    'projection_propagation_proposals',
  );
}

export function getStoryArcListProjection(): Promise<ProjectionEnvelope<StoryArcListProjection>> {
  return invokeDesktop<ProjectionEnvelope<StoryArcListProjection>>('projection_story_arcs');
}

export function getStoryArcProgressionProjection(): Promise<
  ProjectionEnvelope<StoryArcProgressionProjection>
> {
  return invokeDesktop<ProjectionEnvelope<StoryArcProgressionProjection>>(
    'projection_story_arc_progression',
  );
}

export function getTimelineRenderProjection(): Promise<
  ProjectionEnvelope<TimelineRenderProjection>
> {
  return invokeDesktop<ProjectionEnvelope<TimelineRenderProjection>>('projection_timeline_render');
}

export function getSelectedNodeEditorProjection({
  node_id,
}: SelectedNodeEditorProjectionKey = {}): Promise<
  ProjectionEnvelope<SelectedNodeEditorProjection>
> {
  return invokeDesktop<ProjectionEnvelope<SelectedNodeEditorProjection>>(
    'projection_selected_node',
    {
      query: { node_id: node_id ?? null },
    },
  );
}

export function getChangeReviewProjection(): Promise<ProjectionEnvelope<ChangeReviewProjection>> {
  return invokeDesktop<ProjectionEnvelope<ChangeReviewProjection>>('projection_change_review');
}
