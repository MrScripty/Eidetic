import { describe, expect, it } from 'vitest';

import type { BibleGraphNode } from '$lib/bibleGraphTypes.js';
import {
  bibleGraphCategories,
  canonicalParents,
  categorySchemaAvailable,
  nodeCategory,
} from './bibleGraphCategories.js';

function graphNode(schemaKey: string, parentId: string | null = null): BibleGraphNode {
  return {
    id: `node.${schemaKey}`,
    parent_id: parentId,
    schema_key: schemaKey,
    name: schemaKey,
    system_owned: false,
    sort_order: 0,
  };
}

describe('bible graph categories', () => {
  it('covers every canonical bible root used by backend projections', () => {
    expect(bibleGraphCategories).toEqual([
      'Character',
      'Location',
      'Prop',
      'Culture',
      'Theme',
      'Event',
      'Rule',
      'Reference',
    ]);
    expect(Object.values(canonicalParents)).toEqual([
      'canonical.characters',
      'canonical.places',
      'canonical.objects',
      'canonical.cultures',
      'canonical.themes',
      'canonical.events',
      'canonical.rules',
      'canonical.references',
    ]);
  });

  it('classifies canonical root, schema, and parent-derived categories', () => {
    expect(nodeCategory(graphNode('canonical.root.cultures'))).toBe('Culture');
    expect(nodeCategory(graphNode('rule'))).toBe('Rule');
    expect(nodeCategory(graphNode('reference'))).toBe('Reference');
    expect(nodeCategory(graphNode('object'))).toBe('Prop');
    expect(nodeCategory(graphNode('custom', 'canonical.references'))).toBe('Reference');
  });

  it('leaves categories without editable schemas unavailable for add controls', () => {
    const projection = {
      schemas: [
        {
          schema_key: 'character',
          parts: [],
        },
      ],
    };

    expect(categorySchemaAvailable('Character', projection)).toBe(true);
    expect(categorySchemaAvailable('Culture', projection)).toBe(false);
    expect(categorySchemaAvailable('Rule', projection)).toBe(false);
    expect(categorySchemaAvailable('Reference', projection)).toBe(false);
  });
});
