import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';
import { graphSelectionDetail } from '$lib/components/layout/graphSelectionDetails.js';
import { bibleState } from './bible.svelte.js';
import { applyGraphRendererCommands } from './graphRendererCommands.js';

const edgeProjection: BibleRenderGraphProjection = {
  nodes: [
    {
      node_id: 'node.character.ada',
      parent_id: 'canonical.characters',
      schema_key: 'character',
      label: 'Ada',
      system_owned: false,
      sort_order: 0,
      depth: 1,
      position: { x: 0, y: 0, z: 0 },
    },
    {
      node_id: 'node.place.beach',
      parent_id: 'canonical.places',
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
      to_node_id: 'node.place.beach',
      edge_kind: 'located_in',
      label: 'located in',
      directed: true,
      sort_order: 0,
    },
  ],
  neighborhoods: [],
  influences: [],
};

beforeEach(() => {
  bibleState.graphSelection = { kind: 'none' };
});

describe('graph renderer command application', () => {
  it('applies renderer selections through a transient selection target', () => {
    const target = {
      selectNode: vi.fn(),
      selectEdge: vi.fn(),
      selectInfluence: vi.fn(),
      inspectNode: vi.fn(),
      focusNode: vi.fn(),
      navigateToNode: vi.fn(),
      clearSelection: vi.fn(),
    };

    const applied = applyGraphRendererCommands(
      [
        { type: 'select_node', node_id: 'node.character.ada' },
        { type: 'select_edge', edge_id: 'edge.ada.beach' },
        { type: 'select_influence', influence_id: '00000000-0000-0000-0000-000000000001' },
        { type: 'inspect_node', node_id: 'node.location.beach' },
        { type: 'focus_node', node_id: 'node.location.beach' },
        { type: 'navigate_to_node', node_id: 'node.character.ada' },
        { type: 'clear_selection' },
      ],
      target,
    );

    expect(applied).toBe(7);
    expect(target.selectNode).toHaveBeenCalledWith('node.character.ada');
    expect(target.selectEdge).toHaveBeenCalledWith('edge.ada.beach');
    expect(target.selectInfluence).toHaveBeenCalledWith('00000000-0000-0000-0000-000000000001');
    expect(target.inspectNode).toHaveBeenCalledWith('node.location.beach');
    expect(target.focusNode).toHaveBeenCalledWith('node.location.beach');
    expect(target.navigateToNode).toHaveBeenCalledWith('node.character.ada');
    expect(target.clearSelection).toHaveBeenCalledOnce();
  });

  it('connects native edge selection commands to edge detail projection lookup', () => {
    applyGraphRendererCommands([{ type: 'select_edge', edge_id: 'edge.ada.beach' }]);

    expect(bibleState.graphSelection).toEqual({ kind: 'edge', edgeId: 'edge.ada.beach' });
    expect(graphSelectionDetail(edgeProjection, bibleState.graphSelection)).toEqual({
      kind: 'edge',
      edge: edgeProjection.edges[0],
      fromLabel: 'Ada',
      toLabel: 'Beach',
    });
  });
});
