import type { BibleGraphSchemaListProjection } from '$lib/bibleGraphSchemaTypes.js';
import type {
  BibleGraphNode,
  BibleGraphNodeCommandResponse,
  BibleGraphNodeListProjection,
  BibleGraphRootsCommandResponse,
  CreateBibleGraphNodeCommand,
} from '$lib/bibleGraphTypes.js';
import {
  createBibleGraphNodeProjection,
  ensureCanonicalBibleRootProjections,
  getCachedBibleGraphNodeListProjection,
  refreshBibleGraphNodeListProjection,
} from '$lib/stores/bibleGraphNodeProjection.svelte.js';
import {
  getCachedBibleGraphSchemaListProjection,
  refreshBibleGraphSchemaListProjection,
} from '$lib/stores/bibleGraphSchemaProjection.svelte.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import {
  canonicalParents,
  canonicalRootSchemaKeys,
  categorySchemaAvailable,
  categorySchemaKey,
  newNodeName,
  type BibleGraphRootCategory,
} from './bibleGraphCategories.js';

export interface BibleGraphNodeCreateFlowDependencies {
  getCachedSchemaListProjection: () => ProjectionEnvelope<BibleGraphSchemaListProjection> | null;
  refreshSchemaListProjection: () => Promise<ProjectionEnvelope<BibleGraphSchemaListProjection>>;
  getCachedNodeListProjection: () => ProjectionEnvelope<BibleGraphNodeListProjection> | null;
  refreshNodeListProjection: () => Promise<ProjectionEnvelope<BibleGraphNodeListProjection>>;
  ensureCanonicalRoots: () => Promise<BibleGraphRootsCommandResponse>;
  createNode: (payload: CreateBibleGraphNodeCommand) => Promise<BibleGraphNodeCommandResponse>;
}

export interface BibleGraphNodeCreateFlowOptions {
  graphNodes?: BibleGraphNode[];
  deps?: BibleGraphNodeCreateFlowDependencies;
}

const defaultDeps: BibleGraphNodeCreateFlowDependencies = {
  getCachedSchemaListProjection: getCachedBibleGraphSchemaListProjection,
  refreshSchemaListProjection: refreshBibleGraphSchemaListProjection,
  getCachedNodeListProjection: getCachedBibleGraphNodeListProjection,
  refreshNodeListProjection: refreshBibleGraphNodeListProjection,
  ensureCanonicalRoots: ensureCanonicalBibleRootProjections,
  createNode: createBibleGraphNodeProjection,
};

export async function createBibleGraphNodeForCategory(
  category: BibleGraphRootCategory,
  options: BibleGraphNodeCreateFlowOptions = {},
): Promise<BibleGraphNodeCommandResponse> {
  const deps = options.deps ?? defaultDeps;
  const schemaProjection = await schemaListForCategory(category, deps);
  const schemaKey = categorySchemaKey(category, schemaProjection);
  if (!schemaKey) {
    throw new Error(`Schema unavailable for ${category}`);
  }

  const graphNodes = options.graphNodes ?? (await graphNodesForCreateFlow(deps));
  const parentId = await parentIdForCategory(category, graphNodes, deps);
  return deps.createNode({
    parent_id: parentId,
    schema_key: schemaKey,
    name: newNodeName(category),
    sort_order: nextSortOrderForCategory(category, graphNodes),
  });
}

async function schemaListForCategory(
  category: BibleGraphRootCategory,
  deps: BibleGraphNodeCreateFlowDependencies,
): Promise<BibleGraphSchemaListProjection> {
  const cachedProjection = deps.getCachedSchemaListProjection()?.payload;
  if (cachedProjection && categorySchemaAvailable(category, cachedProjection)) {
    return cachedProjection;
  }
  return (await deps.refreshSchemaListProjection()).payload;
}

async function graphNodesForCreateFlow(
  deps: BibleGraphNodeCreateFlowDependencies,
): Promise<BibleGraphNode[]> {
  const cachedNodes = deps.getCachedNodeListProjection()?.payload.nodes;
  if (cachedNodes) {
    return cachedNodes;
  }
  return (await deps.refreshNodeListProjection()).payload.nodes;
}

async function parentIdForCategory(
  category: BibleGraphRootCategory,
  graphNodes: BibleGraphNode[],
  deps: BibleGraphNodeCreateFlowDependencies,
): Promise<string> {
  const existingRoot = graphNodes.find(
    (node) => node.schema_key === canonicalRootSchemaKeys[category],
  );
  if (existingRoot) {
    return existingRoot.id;
  }

  const response = await deps.ensureCanonicalRoots();
  const ensuredRoot = response.projection.payload.nodes.find(
    (node) => node.schema_key === canonicalRootSchemaKeys[category],
  );
  return ensuredRoot?.id ?? canonicalParents[category];
}

function nextSortOrderForCategory(
  category: BibleGraphRootCategory,
  graphNodes: BibleGraphNode[],
): number {
  return graphNodes.filter((node) => node.parent_id === canonicalParents[category]).length;
}
