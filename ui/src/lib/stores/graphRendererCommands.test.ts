import { describe, expect, it, vi } from 'vitest';

import { applyGraphRendererCommands } from './graphRendererCommands.js';

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
});
