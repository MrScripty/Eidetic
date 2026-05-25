import type { BibleRenderGraphProjectionRequest } from '$lib/bibleGraphTypes.js';

export interface GraphWorkspaceBootstrapDependencies {
  ensureCanonicalRoots(): Promise<unknown>;
  refreshRenderGraph(request: BibleRenderGraphProjectionRequest): Promise<unknown>;
}

export async function ensureGraphWorkspaceScaffoldProjection(
  request: BibleRenderGraphProjectionRequest,
  dependencies: GraphWorkspaceBootstrapDependencies,
): Promise<void> {
  await dependencies.ensureCanonicalRoots();
  await dependencies.refreshRenderGraph(request);
}
