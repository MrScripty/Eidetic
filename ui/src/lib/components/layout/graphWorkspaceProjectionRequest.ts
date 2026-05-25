import type { BibleRenderGraphProjectionRequest } from '$lib/bibleGraphTypes.js';
import { bibleRenderGraphRequestForWorkspaceSelection } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
import { canonicalParents, type BibleGraphFilter } from '../sidebar/bible/bibleGraphCategories.js';

export interface GraphWorkspaceProjectionSelection {
  selectedTimelineNodeId?: string | null;
  selectedGraphNodeId?: string | null;
  activeFilter: BibleGraphFilter;
  search?: string | null;
}

export function graphWorkspaceProjectionRequest({
  selectedTimelineNodeId,
  selectedGraphNodeId,
  activeFilter,
  search,
}: GraphWorkspaceProjectionSelection): BibleRenderGraphProjectionRequest {
  return bibleRenderGraphRequestForWorkspaceSelection({
    selectedTimelineNodeId,
    selectedGraphNodeId,
    focusedRootId: activeFilter === 'All' ? null : canonicalParents[activeFilter],
    search,
  });
}
