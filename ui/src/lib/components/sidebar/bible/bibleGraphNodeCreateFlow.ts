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
  categorySchema,
  categorySchemaAvailable,
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
  const schema = categorySchema(category, schemaProjection);
  if (!schema) {
    throw new Error(`Schema unavailable for ${category}`);
  }

  const graphNodes = options.graphNodes ?? (await graphNodesForCreateFlow(deps));
  const parentId = await parentIdForCategory(schema, graphNodes, deps);
  return deps.createNode({
    parent_id: parentId,
    schema_key: schema.schema_key,
    name: newNodeName(category, schemaProjection),
    sort_order: nextSortOrderForParent(parentId, graphNodes),
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
  schema: NonNullable<ReturnType<typeof categorySchema>>,
  graphNodes: BibleGraphNode[],
  deps: BibleGraphNodeCreateFlowDependencies,
): Promise<string> {
  if (!schema.canonical_root_schema_key) {
    throw new Error(`Canonical root unavailable for ${schema.display_name}`);
  }
  const existingRoot = graphNodes.find(
    (node) => node.schema_key === schema.canonical_root_schema_key,
  );
  if (existingRoot) {
    return existingRoot.id;
  }

  const response = await deps.ensureCanonicalRoots();
  const ensuredRoot = response.projection.payload.nodes.find(
    (node) => node.schema_key === schema.canonical_root_schema_key,
  );
  if (!ensuredRoot) {
    throw new Error(`Canonical root unavailable for ${schema.display_name}`);
  }
  return ensuredRoot.id;
}

function nextSortOrderForParent(parentId: string, graphNodes: BibleGraphNode[]): number {
  return graphNodes.filter((node) => node.parent_id === parentId).length;
}
