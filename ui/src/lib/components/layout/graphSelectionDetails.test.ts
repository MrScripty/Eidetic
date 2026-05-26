import { describe, expect, it } from 'vitest';

import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import type { ContextStackProjection } from '$lib/contextInfluenceTypes.js';
import { graphSelectionDetail } from './graphSelectionDetails.js';

const projection: BibleRenderGraphProjection = {
  nodes: [
    {
      node_id: 'node.character.ada',
      parent_id: 'root.characters',
      schema_key: 'character',
      category: 'character',
      label: 'Ada',
      system_owned: false,
      sort_order: 0,
      depth: 1,
      position: { x: 0, y: 0, z: 0 },
    },
    {
      node_id: 'node.location.beach',
      parent_id: 'root.locations',
      schema_key: 'location',
      category: 'location',
      label: 'Beach',
      system_owned: false,
      sort_order: 1,
      depth: 1,
      position: { x: 20, y: 0, z: 0 },
    },
  ],
  edges: [
    {
      edge_id: 'edge.ada.beach',
      from_node_id: 'node.character.ada',
      to_node_id: 'node.location.beach',
      edge_kind: 'located_in',
      label: 'located in',
      directed: true,
      sort_order: 0,
    },
  ],
  neighborhoods: [
    {
      node_id: 'node.character.ada',
      connected_node_ids: ['node.location.beach'],
      edge_ids: ['edge.ada.beach'],
    },
  ],
  influences: [
    {
      influence_id: 'influence-1',
      timeline_node_id: 'node.scene.beach',
      source_layer: 'scene',
      influence_kind: 'direct',
      confidence: 0.9,
      reason: 'Scene uses Ada at the beach.',
      provenance: 'ai_selected',
      bible_node_id: 'node.character.ada',
      bible_edge_id: 'edge.ada.beach',
      sort_order: 1,
    },
  ],
};

const contextStack: ContextStackProjection = {
  target_node_id: 'node.scene.beach',
  layers: [
    {
      node_id: 'node.scene.beach',
      level: 'Scene',
      label: 'Beach Scene',
      role: 'target',
      distilled_context: 'Ada arrives at the beach.',
      sort_order: 0,
    },
  ],
};

describe('graph selection details', () => {
  it('derives edge detail labels from the bounded projection', () => {
    expect(graphSelectionDetail(projection, { kind: 'edge', edgeId: 'edge.ada.beach' })).toEqual({
      kind: 'edge',
      edge: projection.edges[0],
      fromLabel: 'Ada',
      toLabel: 'Beach',
    });
  });

  it('derives influence detail without loading durable frontend graph state', () => {
    expect(
      graphSelectionDetail(projection, { kind: 'influence', influenceId: 'influence-1' }),
    ).toEqual({
      kind: 'influence',
      influence: projection.influences[0],
      nodeLabel: 'Ada',
      edgeLabel: 'located in',
    });
  });

  it('derives context layer influence counts by timeline node', () => {
    expect(
      graphSelectionDetail(
        projection,
        {
          kind: 'context_layer',
          timelineNodeId: 'node.scene.beach',
        },
        contextStack,
      ),
    ).toEqual({
      kind: 'context_layer',
      timelineNodeId: 'node.scene.beach',
      influenceCount: 1,
      layer: contextStack.layers[0],
    });
  });

  it('derives neighborhood detail from existing projection neighborhoods', () => {
    expect(
      graphSelectionDetail(projection, { kind: 'neighborhood', nodeId: 'node.character.ada' }),
    ).toEqual({
      kind: 'neighborhood',
      node: projection.nodes[0],
      neighborhood: projection.neighborhoods[0],
      connectedLabels: ['Beach'],
    });
  });
});
