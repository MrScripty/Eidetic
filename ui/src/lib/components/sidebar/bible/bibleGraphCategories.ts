import type { BibleGraphSchemaListProjection } from '$lib/bibleGraphSchemaTypes.js';
import type { BibleGraphNode, BibleGraphNodeId, EntityCategory } from '$lib/types.js';

export type BibleGraphFilter = EntityCategory | 'All';
export type BibleGraphCategory = EntityCategory | 'Other';

export const bibleGraphCategories: EntityCategory[] = [
  'Character',
  'Location',
  'Prop',
  'Theme',
  'Event',
];

export const bibleGraphFilters: BibleGraphFilter[] = ['All', ...bibleGraphCategories];

export const canonicalParents: Record<EntityCategory, BibleGraphNodeId> = {
  Character: 'canonical.characters',
  Location: 'canonical.places',
  Prop: 'canonical.objects',
  Theme: 'canonical.themes',
  Event: 'canonical.events',
};

export const canonicalRootSchemaKeys: Record<EntityCategory, string> = {
  Character: 'canonical.root.characters',
  Location: 'canonical.root.places',
  Prop: 'canonical.root.objects',
  Theme: 'canonical.root.themes',
  Event: 'canonical.root.events',
};

export const schemaKeys: Record<EntityCategory, string> = {
  Character: 'character',
  Location: 'location',
  Prop: 'prop',
  Theme: 'theme',
  Event: 'event',
};

export const defaultNames: Record<EntityCategory, string> = {
  Character: 'New Character',
  Location: 'New Location',
  Prop: 'New Prop',
  Theme: 'New Theme',
  Event: 'New Event',
};

export function filterLabel(filter: BibleGraphFilter): string {
  if (filter === 'All') return 'All';
  return categoryShortLabel(filter);
}

export function categorySchemaKey(
  category: EntityCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): string | undefined {
  const schemaKey = schemaKeys[category];
  if (!projection?.schemas.some((schema) => schema.schema_key === schemaKey)) return undefined;
  return schemaKey;
}

export function categorySchemaAvailable(
  category: EntityCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): boolean {
  return categorySchemaKey(category, projection) !== undefined;
}

export function newNodeName(category: EntityCategory): string {
  return defaultNames[category];
}

export function categoryShortLabel(category: BibleGraphCategory): string {
  switch (category) {
    case 'Character':
      return 'CHR';
    case 'Location':
      return 'LOC';
    case 'Prop':
      return 'PRP';
    case 'Theme':
      return 'THM';
    case 'Event':
      return 'EVT';
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
    case 'Theme':
      return 'var(--color-entity-theme)';
    case 'Event':
      return 'var(--color-entity-event)';
    case 'All':
      return 'var(--color-text-secondary)';
    default:
      return 'var(--color-text-muted)';
  }
}

export function nodeCategory(node: BibleGraphNode): BibleGraphCategory {
  switch (node.schema_key) {
    case 'canonical.root.characters':
    case 'character':
      return 'Character';
    case 'canonical.root.places':
    case 'location':
      return 'Location';
    case 'canonical.root.objects':
    case 'prop':
      return 'Prop';
    case 'canonical.root.themes':
    case 'theme':
      return 'Theme';
    case 'canonical.root.events':
    case 'event':
      return 'Event';
    default:
      return parentCategory(node.parent_id);
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
    case 'canonical.themes':
      return 'Theme';
    case 'canonical.events':
      return 'Event';
    default:
      return 'Other';
  }
}
