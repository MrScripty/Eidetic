import { describe, expect, it } from 'vitest';

import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import { graphWorkspaceEdgeItems, graphWorkspaceNeighborhoodItems } from './graphWorkspaceItems.js';

const projection: BibleRenderGraphProjection = {
  nodes: [
    {
      node_id: 'node.character.ada',
      parent_id: 'root.characters',
      schema_key: 'character',
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
  influences: [],
};

describe('graph workspace items', () => {
  it('derives inspectable edge rows from backend render graph projections', () => {
    expect(graphWorkspaceEdgeItems(projection)).toEqual([
      {
        edgeId: 'edge.ada.beach',
        label: 'located in',
        fromLabel: 'Ada',
        toLabel: 'Beach',
        directed: true,
      },
    ]);
  });

  it('derives inspectable neighborhood rows without storing graph facts', () => {
    expect(graphWorkspaceNeighborhoodItems(projection)).toEqual([
      {
        nodeId: 'node.character.ada',
        label: 'Ada',
        connectedNodeCount: 1,
        edgeCount: 1,
      },
    ]);
  });
});
