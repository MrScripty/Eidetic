import type {
  BibleGraphNodeId,
  BibleGraphNodeListProjection,
  BibleRenderGraphProjection,
  BibleNodeDetailProjection,
} from './bibleGraphTypes.js';
import type { BibleGraphSchemaListProjection } from './bibleGraphSchemaTypes.js';
import type { ChangeReviewProjection } from './changeReviewTypes.js';
import type { ObjectFieldProjection, ObjectKind, ProjectionEnvelope } from './projectionTypes.js';
import type { PropagationProposalListProjection } from './propagationProposalTypes.js';
import type { ScriptDocumentId, ScriptDocumentProjection } from './scriptTypes.js';
import type { SelectedNodeEditorProjection } from './selectedNodeEditorTypes.js';
import type { BibleReferenceProposalListProjection } from './semanticProposalTypes.js';
import type { StoryArcListProjection, StoryArcProgressionProjection } from './storyArcTypes.js';
import type { NodeId } from './timelineTypes.js';
import type { TimelineRenderProjection } from './timelineRenderTypes.js';
import { hasDesktopTransport, invokeDesktop } from './desktopTransport.js';

const BASE = '/api';

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

async function getJson<T>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: 'GET',
    headers: { Accept: 'application/json' },
  });
  const body = await res.json().catch(() => null);
  if (!res.ok) {
    throw new Error((body as Record<string, string> | null)?.error || `HTTP ${res.status}`);
  }
  if (body && typeof body === 'object' && 'error' in body && typeof body.error === 'string') {
    throw new Error(body.error);
  }
  return body as T;
}

export function getObjectFieldProjection({
  object_kind,
  object_id,
}: ObjectFieldProjectionKey): Promise<ProjectionEnvelope<ObjectFieldProjection>> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<ObjectFieldProjection>>('projection_object_field', {
      query: { object_kind, object_id },
    });
  }

  const params = new URLSearchParams({ object_kind, object_id });
  return getJson(`/projections/object-field?${params.toString()}`);
}

export function getBibleGraphNodeProjection({
  node_id,
}: BibleGraphNodeProjectionKey): Promise<ProjectionEnvelope<BibleNodeDetailProjection>> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<BibleNodeDetailProjection>>(
      'projection_bible_graph_node',
      {
        query: { node_id },
      },
    );
  }

  const params = new URLSearchParams({ node_id });
  return getJson(`/projections/bible-graph/node?${params.toString()}`);
}

export function getBibleGraphNodeListProjection(): Promise<
  ProjectionEnvelope<BibleGraphNodeListProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<BibleGraphNodeListProjection>>(
      'projection_bible_graph_nodes',
    );
  }

  return getJson('/projections/bible-graph/nodes');
}

export function getBibleGraphSchemaListProjection(): Promise<
  ProjectionEnvelope<BibleGraphSchemaListProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<BibleGraphSchemaListProjection>>(
      'projection_bible_graph_schemas',
    );
  }

  return getJson('/projections/bible-graph/schemas');
}

export function getBibleRenderGraphProjection(): Promise<
  ProjectionEnvelope<BibleRenderGraphProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<BibleRenderGraphProjection>>(
      'projection_bible_render_graph',
    );
  }

  return getJson('/projections/bible-graph/render');
}

export function getScriptDocumentProjection({
  document_id,
}: ScriptDocumentProjectionKey): Promise<ProjectionEnvelope<ScriptDocumentProjection>> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<ScriptDocumentProjection>>(
      'projection_script_document',
      {
        query: { document_id },
      },
    );
  }

  const params = new URLSearchParams({ document_id });
  return getJson(`/projections/script/document?${params.toString()}`);
}

export function getBibleReferenceProposalListProjection(): Promise<
  ProjectionEnvelope<BibleReferenceProposalListProjection>
> {
  return getJson('/projections/semantic/bible-reference-proposals');
}

export function getPropagationProposalListProjection(): Promise<
  ProjectionEnvelope<PropagationProposalListProjection>
> {
  return getJson('/projections/semantic/propagation-proposals');
}

export function getStoryArcListProjection(): Promise<ProjectionEnvelope<StoryArcListProjection>> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<StoryArcListProjection>>('projection_story_arcs');
  }

  return getJson('/projections/story/arcs');
}

export function getStoryArcProgressionProjection(): Promise<
  ProjectionEnvelope<StoryArcProgressionProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<StoryArcProgressionProjection>>(
      'projection_story_arc_progression',
    );
  }

  return getJson('/projections/story/arc-progression');
}

export function getTimelineRenderProjection(): Promise<
  ProjectionEnvelope<TimelineRenderProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<TimelineRenderProjection>>(
      'projection_timeline_render',
    );
  }

  return getJson('/projections/timeline/render');
}

export function getSelectedNodeEditorProjection({
  node_id,
}: SelectedNodeEditorProjectionKey = {}): Promise<
  ProjectionEnvelope<SelectedNodeEditorProjection>
> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<SelectedNodeEditorProjection>>(
      'projection_selected_node',
      {
        query: { node_id: node_id ?? null },
      },
    );
  }

  if (!node_id) {
    return getJson('/projections/timeline/selected-node');
  }
  const params = new URLSearchParams({ node_id });
  return getJson(`/projections/timeline/selected-node?${params.toString()}`);
}

export function getChangeReviewProjection(): Promise<ProjectionEnvelope<ChangeReviewProjection>> {
  if (hasDesktopTransport()) {
    return invokeDesktop<ProjectionEnvelope<ChangeReviewProjection>>('projection_change_review');
  }

  return getJson('/projections/history/changes');
}
