import { describe, expect, it } from 'vitest';

import { selectGraphContextLayerForTimeline } from './graphWorkspaceTimelineCrossLink.js';

describe('selectGraphContextLayerForTimeline', () => {
  it('selects the context layer and the matching timeline node', () => {
    expect(
      selectGraphContextLayerForTimeline(
        {
          graphSelection: { kind: 'none' },
          selectedTimelineNodeId: null,
        },
        'node.scene.beach',
      ),
    ).toEqual({
      graphSelection: { kind: 'context_layer', timelineNodeId: 'node.scene.beach' },
      selectedTimelineNodeId: 'node.scene.beach',
    });
  });

  it('toggles the graph context selection without clearing the timeline selection', () => {
    expect(
      selectGraphContextLayerForTimeline(
        {
          graphSelection: { kind: 'context_layer', timelineNodeId: 'node.scene.beach' },
          selectedTimelineNodeId: 'node.scene.beach',
        },
        'node.scene.beach',
      ),
    ).toEqual({
      graphSelection: { kind: 'none' },
      selectedTimelineNodeId: 'node.scene.beach',
    });
  });
});
