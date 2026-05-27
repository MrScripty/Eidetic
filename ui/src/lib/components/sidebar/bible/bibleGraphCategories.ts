import type {
  BibleGraphCategoryProjection,
  BibleGraphSchemaListProjection,
  BibleGraphSchemaProjection,
} from '$lib/bibleGraphSchemaTypes.js';
import type { BibleGraphNode, BibleGraphNodeCategory } from '$lib/bibleGraphTypes.js';

export type BibleGraphRootCategory = BibleGraphNodeCategory;
export type BibleGraphFilter = BibleGraphRootCategory | 'all';
export type BibleGraphCategory =
  | 'Character'
  | 'Location'
  | 'Prop'
  | 'Culture'
  | 'Theme'
  | 'Event'
  | 'Rule'
  | 'Reference'
  | 'Other';

export interface BibleGraphCategoryFilterOption {
  filter: BibleGraphFilter;
  label: string;
  rootNodeId?: string | null;
}

export interface BibleGraphCreateCategoryOption {
  category: BibleGraphRootCategory;
  label: string;
  shortLabel: string;
  createSchemaKey: string;
  defaultNodeName: string;
}

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

export function bibleGraphFilterOptions(
  projection: BibleGraphSchemaListProjection | undefined,
): BibleGraphCategoryFilterOption[] {
  return [
    { filter: 'all', label: 'All', rootNodeId: null },
    ...(projection?.categories.map((category) => ({
      filter: category.category,
      label: category.display_name,
      rootNodeId: category.root_node_id,
    })) ?? []),
  ];
}

export function bibleGraphCreateOptions(
  projection: BibleGraphSchemaListProjection | undefined,
): BibleGraphCreateCategoryOption[] {
  return (
    projection?.categories
      .filter(
        (
          category,
        ): category is BibleGraphCategoryProjection & {
          create_schema_key: string;
          default_node_name: string;
        } => Boolean(category.create_schema_key && category.default_node_name),
      )
      .map((category) => ({
        category: category.category,
        label: category.display_name,
        shortLabel: categoryShortLabel(categoryForRenderNode(category.category)),
        createSchemaKey: category.create_schema_key,
        defaultNodeName: category.default_node_name,
      })) ?? []
  );
}

export function filterLabel(
  filter: BibleGraphFilter,
  projection: BibleGraphSchemaListProjection | undefined,
): string {
  return (
    bibleGraphFilterOptions(projection).find((option) => option.filter === filter)?.label ?? 'All'
  );
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
  return categoryProjection(category, projection)?.default_node_name ?? `New ${category}`;
}

export function categorySchema(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): BibleGraphSchemaProjection | undefined {
  const createSchemaKey = categoryProjection(category, projection)?.create_schema_key;
  return projection?.schemas.find((schema) => schema.schema_key === createSchemaKey);
}

export function categoryProjection(
  category: BibleGraphRootCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): BibleGraphCategoryProjection | undefined {
  return projection?.categories.find((option) => option.category === category);
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

export function categoryColor(
  category: BibleGraphFilter | BibleGraphCategory,
  projection: BibleGraphSchemaListProjection | undefined,
): string {
  if (category === 'all') {
    return 'var(--color-text-secondary)';
  }
  const rootCategory = rootCategoryForColor(category);
  return (
    projection?.categories.find((option) => option.category === rootCategory)?.visual_style
      .fill_color ?? 'var(--color-text-muted)'
  );
}

export function nodeCategory(node: BibleGraphNode): BibleGraphCategory {
  return categoryForRenderNode(categoryForSchemaAndParent(node.schema_key, node.parent_id));
}

export function categoryForRenderNode(category: BibleGraphNodeCategory): BibleGraphCategory {
  return categoriesByBackendCategory[category];
}

function categoryForSchemaAndParent(
  schemaKey: string,
  parentId: string | null | undefined,
): BibleGraphNodeCategory {
  switch (schemaKey) {
    case 'canonical.root.characters':
    case 'character':
      return 'character';
    case 'canonical.root.places':
    case 'location':
      return 'location';
    case 'canonical.root.objects':
    case 'prop':
    case 'object':
      return 'prop';
    case 'canonical.root.cultures':
    case 'culture':
      return 'culture';
    case 'canonical.root.themes':
    case 'theme':
      return 'theme';
    case 'canonical.root.events':
    case 'event':
      return 'event';
    case 'canonical.root.rules':
    case 'rule':
      return 'rule';
    case 'canonical.root.references':
    case 'reference':
      return 'reference';
    default:
      return parentCategory(parentId);
  }
}

function parentCategory(parentId: string | null | undefined): BibleGraphNodeCategory {
  switch (parentId) {
    case 'canonical.characters':
      return 'character';
    case 'canonical.places':
      return 'location';
    case 'canonical.objects':
      return 'prop';
    case 'canonical.cultures':
      return 'culture';
    case 'canonical.themes':
      return 'theme';
    case 'canonical.events':
      return 'event';
    case 'canonical.rules':
      return 'rule';
    case 'canonical.references':
      return 'reference';
    default:
      return 'other';
  }
}

function rootCategoryForColor(
  category: BibleGraphRootCategory | BibleGraphCategory,
): BibleGraphRootCategory {
  switch (category) {
    case 'Character':
      return 'character';
    case 'Location':
      return 'location';
    case 'Prop':
      return 'prop';
    case 'Culture':
      return 'culture';
    case 'Theme':
      return 'theme';
    case 'Event':
      return 'event';
    case 'Rule':
      return 'rule';
    case 'Reference':
      return 'reference';
    case 'Other':
      return 'other';
    default:
      return category;
  }
}
