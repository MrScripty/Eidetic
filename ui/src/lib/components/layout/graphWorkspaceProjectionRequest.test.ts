import { describe, expect, it } from 'vitest';

import { graphWorkspaceProjectionRequest } from './graphWorkspaceProjectionRequest.js';

describe('graph workspace projection request', () => {
  it('requests focused backend graph projections for category filters', () => {
    expect(
      graphWorkspaceProjectionRequest({
        selectedTimelineNodeId: 'timeline.scene.beach',
        activeTimelineMs: 42_500,
        activeFilter: 'character',
        activeFilterRootId: 'canonical.characters',
        search: ' Ada ',
      }),
    ).toMatchObject({
      focused_root_id: 'canonical.characters',
      selected_timeline_node_id: 'timeline.scene.beach',
      active_timeline_ms: 42_500,
      search: 'Ada',
    });
  });

  it('does not use normal graph selection as backend projection scope', () => {
    expect(
      graphWorkspaceProjectionRequest({
        selectedTimelineNodeId: 'timeline.scene.beach',
        activeTimelineMs: 42_500,
        activeFilter: 'all',
        search: '',
      }),
    ).not.toHaveProperty('selected_node_id');
  });

  it('requests selected node scope only for explicit focus-neighborhood actions', () => {
    expect(
      graphWorkspaceProjectionRequest({
        focusedNeighborhoodNodeId: 'node.character.ada',
        activeFilter: 'all',
      }),
    ).toMatchObject({
      selected_node_id: 'node.character.ada',
    });
  });

  it('omits focused root and empty search for all-graph projections', () => {
    expect(
      graphWorkspaceProjectionRequest({
        activeFilter: 'all',
        search: '   ',
      }),
    ).toEqual({
      neighborhood_depth: 1,
      max_nodes: 200,
      max_edges: 500,
    });
  });

  it('includes edge kind filters when graph controls request them', () => {
    expect(
      graphWorkspaceProjectionRequest({
        activeFilter: 'all',
        edgeKinds: ['located_in', 'supports_theme'],
      }),
    ).toMatchObject({
      edge_kinds: ['located_in', 'supports_theme'],
    });
  });
});
