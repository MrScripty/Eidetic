import { beforeEach, describe, expect, it } from 'vitest';

import { bibleState, selectBibleGraphNode, selectEntity } from './bible.svelte.js';

beforeEach(() => {
  bibleState.selectedEntityId = null;
  bibleState.selectedGraphNodeId = null;
});

describe('bible selection store', () => {
  it('keeps legacy entity and graph node selection mutually exclusive', () => {
    selectEntity('entity-1');

    expect(bibleState.selectedEntityId).toBe('entity-1');
    expect(bibleState.selectedGraphNodeId).toBeNull();

    selectBibleGraphNode('node.character.ada');

    expect(bibleState.selectedEntityId).toBeNull();
    expect(bibleState.selectedGraphNodeId).toBe('node.character.ada');
  });
});
