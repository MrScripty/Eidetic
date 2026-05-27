import { describe, expect, it } from 'vitest';

import type { BibleGraphNode } from '$lib/bibleGraphTypes.js';
import {
  bibleGraphCreateOptions,
  bibleGraphFilterOptions,
  categoryColor,
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
    expect(bibleGraphFilterOptions(projection()).map((option) => option.label)).toEqual([
      'All',
      'Character',
      'Location',
      'Prop',
      'Culture',
      'Theme',
      'Event',
      'Rule',
      'Reference',
    ]);
    expect(
      bibleGraphFilterOptions(projection())
        .slice(1)
        .map((option) => option.rootNodeId),
    ).toEqual([
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
    expect(categorySchemaAvailable('character', projection())).toBe(true);
    expect(categorySchemaAvailable('culture', projection())).toBe(false);
    expect(categorySchemaAvailable('rule', projection())).toBe(false);
    expect(categorySchemaAvailable('reference', projection())).toBe(false);
    expect(bibleGraphCreateOptions(projection()).map((option) => option.category)).toEqual([
      'character',
      'location',
      'prop',
      'theme',
      'event',
    ]);
  });

  it('uses backend-owned category colors from the schema projection', () => {
    expect(categoryColor('character', projection())).toBe('#6495ed');
    expect(categoryColor('Prop', projection())).toBe('#f97316');
  });
});

function projection() {
  return {
    categories: [
      category('character', 'Character', 'canonical.characters', 'canonical.root.characters', true),
      category('location', 'Location', 'canonical.places', 'canonical.root.places', true),
      category('prop', 'Prop', 'canonical.objects', 'canonical.root.objects', true),
      category('culture', 'Culture', 'canonical.cultures', 'canonical.root.cultures', false),
      category('theme', 'Theme', 'canonical.themes', 'canonical.root.themes', true),
      category('event', 'Event', 'canonical.events', 'canonical.root.events', true),
      category('rule', 'Rule', 'canonical.rules', 'canonical.root.rules', false),
      category(
        'reference',
        'Reference',
        'canonical.references',
        'canonical.root.references',
        false,
      ),
    ],
    schemas: [
      schema(
        'character',
        'character',
        'Character',
        'canonical.characters',
        'canonical.root.characters',
      ),
      schema('location', 'location', 'Location', 'canonical.places', 'canonical.root.places'),
      schema('prop', 'prop', 'Prop', 'canonical.objects', 'canonical.root.objects'),
      schema('theme', 'theme', 'Theme', 'canonical.themes', 'canonical.root.themes'),
      schema('event', 'event', 'Event', 'canonical.events', 'canonical.root.events'),
    ],
  };
}

function category(
  category:
    | 'character'
    | 'location'
    | 'prop'
    | 'culture'
    | 'theme'
    | 'event'
    | 'rule'
    | 'reference',
  display_name: string,
  root_node_id: string,
  root_schema_key: string,
  creatable: boolean,
) {
  return {
    category,
    display_name,
    visual_style: { fill_color: fillColorForCategory(category) },
    root_node_id,
    root_schema_key,
    create_schema_key: creatable ? category : null,
    default_node_name: creatable ? `New ${display_name}` : null,
  };
}

function schema(
  schema_key: string,
  category: 'character' | 'location' | 'prop' | 'theme' | 'event',
  display_name: string,
  canonical_parent_id: string,
  canonical_root_schema_key: string,
) {
  return {
    schema_key,
    category,
    display_name,
    visual_style: { fill_color: fillColorForCategory(category) },
    default_node_name: `New ${display_name}`,
    canonical_parent_id,
    canonical_root_schema_key,
    parts: [],
  };
}

function fillColorForCategory(category: string): string {
  switch (category) {
    case 'character':
      return '#6495ed';
    case 'location':
      return '#22c55e';
    case 'prop':
      return '#f97316';
    case 'culture':
      return '#14b8a6';
    case 'theme':
      return '#a855f7';
    case 'event':
      return '#ef4444';
    case 'rule':
      return '#eab308';
    case 'reference':
      return '#38bdf8';
    default:
      return '#34495e';
  }
}
