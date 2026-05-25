import { beforeEach, describe, expect, it } from 'vitest';

import {
  bibleState,
  clearBibleGraphFocusedNeighborhood,
  clearBibleGraphSelection,
  focusBibleGraphNeighborhood,
  selectBibleGraphEdge,
  selectBibleGraphInfluence,
  selectBibleGraphNode,
  selectedBibleGraphNodeId,
} from './bible.svelte.js';

beforeEach(() => {
  bibleState.graphSelection = { kind: 'none' };
  bibleState.graphFocusedNeighborhoodNodeId = null;
});

describe('bible selection store', () => {
  it('stores selected bible graph node ids', () => {
    selectBibleGraphNode('node.character.ada');

    expect(bibleState.graphSelection).toEqual({ kind: 'node', nodeId: 'node.character.ada' });
    expect(selectedBibleGraphNodeId()).toBe('node.character.ada');

    selectBibleGraphNode(null);

    expect(bibleState.graphSelection).toEqual({ kind: 'none' });
    expect(selectedBibleGraphNodeId()).toBeNull();
  });

  it('stores typed non-node graph selections without node detail ambiguity', () => {
    selectBibleGraphEdge('edge.ada.beach');

    expect(bibleState.graphSelection).toEqual({ kind: 'edge', edgeId: 'edge.ada.beach' });
    expect(selectedBibleGraphNodeId()).toBeNull();

    selectBibleGraphInfluence('influence-1');

    expect(bibleState.graphSelection).toEqual({
      kind: 'influence',
      influenceId: 'influence-1',
    });
    expect(selectedBibleGraphNodeId()).toBeNull();

    clearBibleGraphSelection();

    expect(bibleState.graphSelection).toEqual({ kind: 'none' });
  });

  it('stores explicit graph focus scope separately from normal selection', () => {
    selectBibleGraphNode('node.character.ada');

    expect(bibleState.graphSelection).toEqual({ kind: 'node', nodeId: 'node.character.ada' });
    expect(bibleState.graphFocusedNeighborhoodNodeId).toBeNull();

    focusBibleGraphNeighborhood('node.place.beach');

    expect(bibleState.graphSelection).toEqual({
      kind: 'neighborhood',
      nodeId: 'node.place.beach',
    });
    expect(bibleState.graphFocusedNeighborhoodNodeId).toBe('node.place.beach');

    clearBibleGraphSelection();

    expect(bibleState.graphSelection).toEqual({ kind: 'none' });
    expect(bibleState.graphFocusedNeighborhoodNodeId).toBe('node.place.beach');

    clearBibleGraphFocusedNeighborhood();

    expect(bibleState.graphFocusedNeighborhoodNodeId).toBeNull();
  });
});
