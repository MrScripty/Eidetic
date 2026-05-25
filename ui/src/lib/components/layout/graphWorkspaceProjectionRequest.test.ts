import { describe, expect, it } from 'vitest';

import { graphWorkspaceProjectionRequest } from './graphWorkspaceProjectionRequest.js';

describe('graph workspace projection request', () => {
  it('requests focused backend graph projections for category filters', () => {
    expect(
      graphWorkspaceProjectionRequest({
        selectedTimelineNodeId: 'timeline.scene.beach',
        selectedGraphNodeId: 'node.character.ada',
        activeFilter: 'Character',
        search: ' Ada ',
      }),
    ).toMatchObject({
      focused_root_id: 'canonical.characters',
      selected_timeline_node_id: 'timeline.scene.beach',
      selected_node_id: 'node.character.ada',
      search: 'Ada',
    });
  });

  it('omits focused root and empty search for all-graph projections', () => {
    expect(
      graphWorkspaceProjectionRequest({
        activeFilter: 'All',
        search: '   ',
      }),
    ).toEqual({
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
  });
});
