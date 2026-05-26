import { describe, expect, it, vi } from 'vitest';

import type {
  BibleGraphNode,
  BibleGraphNodeCommandResponse,
  BibleGraphRootsCommandResponse,
} from '$lib/bibleGraphTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { BibleGraphSchemaListProjection } from '$lib/bibleGraphSchemaTypes.js';
import {
  createBibleGraphNodeForCategory,
  type BibleGraphNodeCreateFlowDependencies,
} from './bibleGraphNodeCreateFlow.js';

describe('bible graph node create flow', () => {
  it('creates category nodes through the backend command path', async () => {
    const root = node('canonical.characters', null, 'canonical.root.characters', 'Characters');
    const ada = node('node.character.ada', root.id, 'character', 'Ada');
    const response = responseFor(node('node.character.new', root.id, 'character', 'New Character'));
    const deps = depsFor({
      schemas: schemaEnvelope(['character']),
      nodes: nodeListEnvelope([root, ada]),
      createNode: vi.fn().mockResolvedValue(response),
    });

    await expect(createBibleGraphNodeForCategory('character', { deps })).resolves.toBe(response);

    expect(deps.createNode).toHaveBeenCalledWith({
      parent_id: root.id,
      schema_key: 'character',
      name: 'New Character',
      sort_order: 1,
    });
    expect(deps.ensureCanonicalRoots).not.toHaveBeenCalled();
  });

  it('ensures canonical roots when the category parent is missing', async () => {
    const root = node('canonical.places', null, 'canonical.root.places', 'Places');
    const response = responseFor(node('node.location.new', root.id, 'location', 'New Location'));
    const deps = depsFor({
      schemas: schemaEnvelope(['location']),
      nodes: nodeListEnvelope([]),
      roots: rootsResponse([root]),
      createNode: vi.fn().mockResolvedValue(response),
    });

    await createBibleGraphNodeForCategory('location', { deps });

    expect(deps.ensureCanonicalRoots).toHaveBeenCalledTimes(1);
    expect(deps.createNode).toHaveBeenCalledWith({
      parent_id: root.id,
      schema_key: 'location',
      name: 'New Location',
      sort_order: 0,
    });
  });

  it('refreshes schemas before rejecting unavailable categories', async () => {
    const deps = depsFor({
      schemas: schemaEnvelope([]),
      refreshedSchemas: schemaEnvelope([]),
      nodes: nodeListEnvelope([]),
    });

    await expect(createBibleGraphNodeForCategory('theme', { deps })).rejects.toThrow(
      'Schema unavailable for theme',
    );

    expect(deps.refreshSchemaListProjection).toHaveBeenCalledTimes(1);
    expect(deps.createNode).not.toHaveBeenCalled();
  });

  it('rejects create when backend canonical root projection is unavailable', async () => {
    const deps = depsFor({
      schemas: schemaEnvelope(['character']),
      nodes: nodeListEnvelope([]),
      roots: rootsResponse([]),
    });

    await expect(createBibleGraphNodeForCategory('character', { deps })).rejects.toThrow(
      'Canonical root unavailable for Character',
    );

    expect(deps.createNode).not.toHaveBeenCalled();
  });
});

function depsFor({
  schemas,
  refreshedSchemas,
  nodes,
  roots = rootsResponse([]),
  createNode = vi.fn(),
}: {
  schemas: ProjectionEnvelope<BibleGraphSchemaListProjection>;
  refreshedSchemas?: ProjectionEnvelope<BibleGraphSchemaListProjection>;
  nodes: ProjectionEnvelope<{ nodes: BibleGraphNode[] }>;
  roots?: BibleGraphRootsCommandResponse;
  createNode?: BibleGraphNodeCreateFlowDependencies['createNode'];
}): BibleGraphNodeCreateFlowDependencies {
  return {
    getCachedSchemaListProjection: vi.fn(() => schemas),
    refreshSchemaListProjection: vi.fn(async () => refreshedSchemas ?? schemas),
    getCachedNodeListProjection: vi.fn(() => nodes),
    refreshNodeListProjection: vi.fn(async () => nodes),
    ensureCanonicalRoots: vi.fn(async () => roots),
    createNode,
  };
}

function schemaEnvelope(schemaKeys: string[]): ProjectionEnvelope<BibleGraphSchemaListProjection> {
  return {
    version: 1,
    payload: {
      categories: [
        categoryEnvelope('character', schemaKeys.includes('character')),
        categoryEnvelope('location', schemaKeys.includes('location')),
        categoryEnvelope('prop', schemaKeys.includes('prop')),
        categoryEnvelope('theme', schemaKeys.includes('theme')),
        categoryEnvelope('event', schemaKeys.includes('event')),
      ],
      schemas: schemaKeys.map((schema_key) => ({
        schema_key,
        category: categoryForSchema(schema_key),
        display_name: displayNameForSchema(schema_key),
        default_node_name: `New ${displayNameForSchema(schema_key)}`,
        canonical_parent_id: canonicalParentForSchema(schema_key),
        canonical_root_schema_key: canonicalRootSchemaForSchema(schema_key),
        parts: [],
      })),
    },
  };
}

function categoryEnvelope(
  category: BibleGraphSchemaListProjection['categories'][number]['category'],
  creatable: boolean,
): BibleGraphSchemaListProjection['categories'][number] {
  const displayName = displayNameForSchema(category);
  return {
    category,
    display_name: displayName,
    root_node_id: canonicalParentForSchema(category),
    root_schema_key: canonicalRootSchemaForSchema(category),
    create_schema_key: creatable ? category : null,
    default_node_name: creatable ? `New ${displayName}` : null,
  };
}

function categoryForSchema(
  schemaKey: string,
): BibleGraphSchemaListProjection['schemas'][number]['category'] {
  switch (schemaKey) {
    case 'character':
      return 'character';
    case 'location':
      return 'location';
    case 'prop':
      return 'prop';
    case 'theme':
      return 'theme';
    case 'event':
      return 'event';
    default:
      return 'other';
  }
}

function displayNameForSchema(schemaKey: string): string {
  switch (schemaKey) {
    case 'character':
      return 'Character';
    case 'location':
      return 'Location';
    case 'prop':
      return 'Prop';
    case 'theme':
      return 'Theme';
    case 'event':
      return 'Event';
    default:
      return 'Other';
  }
}

function canonicalParentForSchema(schemaKey: string): string {
  switch (schemaKey) {
    case 'character':
      return 'canonical.characters';
    case 'location':
      return 'canonical.places';
    case 'prop':
      return 'canonical.objects';
    case 'theme':
      return 'canonical.themes';
    case 'event':
      return 'canonical.events';
    default:
      return 'canonical.references';
  }
}

function canonicalRootSchemaForSchema(schemaKey: string): string {
  switch (schemaKey) {
    case 'character':
      return 'canonical.root.characters';
    case 'location':
      return 'canonical.root.places';
    case 'prop':
      return 'canonical.root.objects';
    case 'theme':
      return 'canonical.root.themes';
    case 'event':
      return 'canonical.root.events';
    default:
      return 'canonical.root.references';
  }
}

function nodeListEnvelope(
  nodes: BibleGraphNode[],
): ProjectionEnvelope<{ nodes: BibleGraphNode[] }> {
  return { version: 1, payload: { nodes } };
}

function rootsResponse(nodes: BibleGraphNode[]): BibleGraphRootsCommandResponse {
  return {
    outcome: 'recorded',
    projection: nodeListEnvelope(nodes),
  };
}

function responseFor(node: BibleGraphNode): BibleGraphNodeCommandResponse {
  return {
    outcome: 'recorded',
    projection: {
      version: 1,
      payload: {
        node,
        parts: [],
        incoming_edges: [],
        outgoing_edges: [],
        snapshots: [],
      },
    },
  };
}

function node(
  id: string,
  parent_id: string | null,
  schema_key: string,
  name: string,
): BibleGraphNode {
  return {
    id,
    parent_id,
    schema_key,
    name,
    system_owned: schema_key.startsWith('canonical.'),
    sort_order: 0,
  };
}
