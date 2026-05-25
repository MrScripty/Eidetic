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

    await expect(createBibleGraphNodeForCategory('Character', { deps })).resolves.toBe(response);

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

    await createBibleGraphNodeForCategory('Location', { deps });

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

    await expect(createBibleGraphNodeForCategory('Theme', { deps })).rejects.toThrow(
      'Schema unavailable for Theme',
    );

    expect(deps.refreshSchemaListProjection).toHaveBeenCalledTimes(1);
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
      schemas: schemaKeys.map((schema_key) => ({ schema_key, parts: [] })),
    },
  };
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
