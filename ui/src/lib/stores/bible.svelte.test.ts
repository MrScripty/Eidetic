import { beforeEach, describe, expect, it } from 'vitest';

import { bibleState, selectBibleGraphNode } from './bible.svelte.js';

beforeEach(() => {
  bibleState.selectedGraphNodeId = null;
});

describe('bible selection store', () => {
  it('stores selected bible graph node ids', () => {
    selectBibleGraphNode('node.character.ada');

    expect(bibleState.selectedGraphNodeId).toBe('node.character.ada');

    selectBibleGraphNode(null);

    expect(bibleState.selectedGraphNodeId).toBeNull();
  });
});
