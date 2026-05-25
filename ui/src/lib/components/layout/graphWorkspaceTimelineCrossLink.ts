import type { BibleGraphSelection } from '$lib/stores/bible.svelte.js';
import type { NodeId } from '$lib/timelineTypes.js';

export interface GraphWorkspaceTimelineCrossLinkState {
  graphSelection: BibleGraphSelection;
  selectedTimelineNodeId: NodeId | null;
}

export interface GraphWorkspaceTimelineCrossLinkResult {
  graphSelection: BibleGraphSelection;
  selectedTimelineNodeId: NodeId | null;
}

export function selectGraphContextLayerForTimeline(
  state: GraphWorkspaceTimelineCrossLinkState,
  timelineNodeId: NodeId,
): GraphWorkspaceTimelineCrossLinkResult {
  if (
    state.graphSelection.kind === 'context_layer' &&
    state.graphSelection.timelineNodeId === timelineNodeId
  ) {
    return {
      graphSelection: { kind: 'none' },
      selectedTimelineNodeId: state.selectedTimelineNodeId,
    };
  }

  return {
    graphSelection: { kind: 'context_layer', timelineNodeId },
    selectedTimelineNodeId: timelineNodeId,
  };
}
