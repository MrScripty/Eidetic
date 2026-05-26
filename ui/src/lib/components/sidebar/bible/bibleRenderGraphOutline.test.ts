import { describe, expect, it } from 'vitest';

import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import { bibleRenderGraphOutlineItems } from './bibleRenderGraphOutline.js';

const projection: BibleRenderGraphProjection = {
  selected_node_id: 'node.character.ada',
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

describe('bible render graph outline', () => {
  it('derives keyboard outline items from backend render graph projections', () => {
    const items = bibleRenderGraphOutlineItems(projection);

    expect(items).toEqual([
      {
        node_id: 'node.character.ada',
        label: 'Ada',
        depth: 1,
        connected_node_count: 1,
        edge_count: 1,
        influence_count: 1,
        active: true,
        selected: true,
      },
      {
        node_id: 'node.location.beach',
        label: 'Beach',
        depth: 1,
        connected_node_count: 0,
        edge_count: 0,
        influence_count: 0,
        active: false,
        selected: false,
      },
    ]);
  });

  it('filters by label without mutating projection order', () => {
    const items = bibleRenderGraphOutlineItems(projection, null, 'bea');

    expect(items.map((item) => item.node_id)).toEqual(['node.location.beach']);
  });

  it('allows callers to override projected selection with transient selection', () => {
    const items = bibleRenderGraphOutlineItems(projection, 'node.location.beach');

    expect(items.map((item) => [item.node_id, item.selected])).toEqual([
      ['node.character.ada', false],
      ['node.location.beach', true],
    ]);
  });
});
