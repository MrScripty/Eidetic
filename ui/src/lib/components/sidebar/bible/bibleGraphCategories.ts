import type {
  BibleGraphSchemaListProjection,
  BibleGraphSchemaProjection,
} from '$lib/bibleGraphSchemaTypes.js';
import type {
  BibleGraphNode,
  BibleGraphNodeCategory,
  BibleGraphNodeId,
} from '$lib/bibleGraphTypes.js';

export type BibleGraphRootCategory =
  | 'Character'
  | 'Location'
  | 'Prop'
  | 'Culture'
  | 'Theme'
  | 'Event'
  | 'Rule'
  | 'Reference';
export type BibleGraphFilter = BibleGraphRootCategory | 'All';
export type BibleGraphCategory = BibleGraphRootCategory | 'Other';

export const bibleGraphCategories: BibleGraphRootCategory[] = [
  'Character',
  'Location',
  'Prop',
  'Culture',
  'Theme',
  'Event',
  'Rule',
  'Reference',
];

export const bibleGraphFilters: BibleGraphFilter[] = ['All', ...bibleGraphCategories];

export const canonicalParents: Record<BibleGraphRootCategory, BibleGraphNodeId> = {
  Character: 'canonical.characters',
  Location: 'canonical.places',
  Prop: 'canonical.objects',
  Culture: 'canonical.cultures',
  Theme: 'canonical.themes',
  Event: 'canonical.events',
  Rule: 'canonical.rules',
  Reference: 'canonical.references',
};

export const canonicalRootSchemaKeys: Record<BibleGraphRootCategory, string> = {
  Character: 'canonical.root.characters',
  Location: 'canonical.root.places',
  Prop: 'canonical.root.objects',
  Culture: 'canonical.root.cultures',
  Theme: 'canonical.root.themes',
  Event: 'canonical.root.events',
  Rule: 'canonical.root.rules',
  Reference: 'canonical.root.references',
};

const categoriesByBackendCategory: Record<BibleGraphNodeCategory, BibleGraphCategory> = {
  character: 'Character',
  location: 'Location',
  prop: 'Prop',
  culture: 'Culture',
  theme: 'Theme',
  event: 'Event',
  rule: 'Rule',
  reference: 'Reference',
  canonical: 'Other',
  other: 'Other',
};

const backendCategoriesByRootCategory: Record<BibleGraphRootCategory, BibleGraphNodeCategory> = {
  Character: 'character',
  Location: 'location',
  Prop: 'prop',
  Culture: 'culture',
  Theme: 'theme',
  Event: 'event',
  Rule: 'rule',
  Reference: 'reference',
};

export function filterLabel(filter: BibleGraphFilter): string {
  if (filter === 'All') return 'All';
  return filter;
}

export function categorySchemaKey(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): string | undefined {
  return categorySchema(category, projection)?.schema_key;
}

export function categorySchemaAvailable(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): boolean {
  return categorySchemaKey(category, projection) !== undefined;
}

export function newNodeName(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): string {
  return categorySchema(category, projection)?.default_node_name ?? `New ${category}`;
}

export function categorySchema(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): BibleGraphSchemaProjection | undefined {
  const backendCategory = backendCategoriesByRootCategory[category];
  return projection?.schemas.find((schema) => schema.category === backendCategory);
}

export function categoryShortLabel(category: BibleGraphCategory): string {
  switch (category) {
    case 'Character':
      return 'CHR';
    case 'Location':
      return 'LOC';
    case 'Prop':
      return 'PRP';
    case 'Culture':
      return 'CUL';
    case 'Theme':
      return 'THM';
    case 'Event':
      return 'EVT';
    case 'Rule':
      return 'RUL';
    case 'Reference':
      return 'REF';
    case 'Other':
      return 'OTH';
  }
}

export function categoryColor(category: BibleGraphFilter | BibleGraphCategory): string {
  switch (category) {
    case 'Character':
      return 'var(--color-entity-character)';
    case 'Location':
      return 'var(--color-entity-location)';
    case 'Prop':
      return 'var(--color-entity-prop)';
    case 'Culture':
      return 'var(--color-entity-culture)';
    case 'Theme':
      return 'var(--color-entity-theme)';
    case 'Event':
      return 'var(--color-entity-event)';
    case 'Rule':
      return 'var(--color-entity-rule)';
    case 'Reference':
      return 'var(--color-entity-reference)';
    case 'All':
      return 'var(--color-text-secondary)';
    default:
      return 'var(--color-text-muted)';
  }
}

export function nodeCategory(node: BibleGraphNode): BibleGraphCategory {
  return categoryForSchemaAndParent(node.schema_key, node.parent_id);
}

export function categoryForRenderNode(category: BibleGraphNodeCategory): BibleGraphCategory {
  return categoriesByBackendCategory[category];
}

function categoryForSchemaAndParent(
  schemaKey: string,
  parentId: BibleGraphNodeId | null | undefined,
): BibleGraphCategory {
  switch (schemaKey) {
    case 'canonical.root.characters':
    case 'character':
      return 'Character';
    case 'canonical.root.places':
    case 'location':
      return 'Location';
    case 'canonical.root.objects':
    case 'prop':
    case 'object':
      return 'Prop';
    case 'canonical.root.cultures':
    case 'culture':
      return 'Culture';
    case 'canonical.root.themes':
    case 'theme':
      return 'Theme';
    case 'canonical.root.events':
    case 'event':
      return 'Event';
    case 'canonical.root.rules':
    case 'rule':
      return 'Rule';
    case 'canonical.root.references':
    case 'reference':
      return 'Reference';
    default:
      return parentCategory(parentId);
  }
}

function parentCategory(parentId: BibleGraphNodeId | null | undefined): BibleGraphCategory {
  switch (parentId) {
    case 'canonical.characters':
      return 'Character';
    case 'canonical.places':
      return 'Location';
    case 'canonical.objects':
      return 'Prop';
    case 'canonical.cultures':
      return 'Culture';
    case 'canonical.themes':
      return 'Theme';
    case 'canonical.events':
      return 'Event';
    case 'canonical.rules':
      return 'Rule';
    case 'canonical.references':
      return 'Reference';
    default:
      return 'Other';
  }
}
