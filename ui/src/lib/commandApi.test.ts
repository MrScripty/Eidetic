import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  applyTimelineChildren,
  createBibleGraphNode,
  createStoryArc,
  createTimelineNode,
  createTimelineRelationship,
  deleteStoryArc,
  deleteTimelineNode,
  deleteTimelineRelationship,
  ensureCanonicalBibleRoots,
  setBibleGraphEdge,
  setBibleGraphField,
  setBibleGraphSnapshotField,
  setObjectField,
  setStoryArcMetadata,
  setTimelineNodeLock,
  setTimelineNodeNotes,
  setTimelineNodeRange,
  splitTimelineNode,
} from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('command api helpers', () => {
  it('sends object field commands and returns versioned projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 2,
        change_event_id: 'event-1',
        payload: {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          deleted: false,
          fields: {
            weather: { type: 'text', value: 'rainy' },
          },
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setObjectField(
        {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
        },
        'command-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/object-field',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-1',
          payload: {
            object_kind: 'bible_part_field',
            object_id: 'field-weather',
            field_key: 'weather',
            value: { type: 'text', value: 'rainy' },
          },
        }),
      }),
    );
  });

  it('uses desktop object field command when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 2,
        payload: {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          deleted: false,
          fields: {},
        },
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setObjectField(
        {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
        },
        'command-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_object_field', {
      command: {
        id: 'command-1',
        payload: {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('throws backend errors without updating local state', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'command conflict' }), {
          status: 409,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(
      setObjectField(
        {
          object_kind: 'bible_part_field',
          object_id: 'field-weather',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
        },
        'command-1',
      ),
    ).rejects.toThrow('command conflict');
  });

  it('sends timeline node range commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [
            {
              track_id: 'track.scene',
              level: 'Scene',
              label: 'Scenes',
              sort_order: 30,
              collapsed: false,
            },
          ],
          clips: [
            {
              node_id: 'node.scene.beach',
              parent_id: null,
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Beach argument',
              start_ms: 1_000,
              end_ms: 4_000,
              sort_order: 10,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setTimelineNodeRange(
        {
          node_id: 'node.scene.beach',
          start_ms: 1_000,
          end_ms: 4_000,
        },
        'command-timeline-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/node-range',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-1',
          payload: {
            node_id: 'node.scene.beach',
            start_ms: 1_000,
            end_ms: 4_000,
          },
        }),
      }),
    );
  });

  it('sends story arc commands and returns arc list projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          arcs: [
            {
              id: 'arc.mystery',
              parent_arc_id: null,
              name: 'Mystery',
              description: 'Central investigation',
              arc_type: 'APlot',
              color: { r: 1, g: 2, b: 3 },
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockImplementation(() =>
      Promise.resolve(
        new Response(JSON.stringify(response), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createStoryArc(
        {
          arc_id: 'arc.mystery',
          parent_arc_id: null,
          name: 'Mystery',
          description: 'Central investigation',
          arc_type: 'APlot',
          color: { r: 1, g: 2, b: 3 },
        },
        'command-story-1',
      ),
    ).resolves.toEqual(response);

    await expect(
      setStoryArcMetadata(
        {
          arc_id: 'arc.mystery',
          name: 'Mystery revised',
        },
        'command-story-2',
      ),
    ).resolves.toEqual(response);

    await expect(
      deleteStoryArc(
        {
          arc_id: 'arc.mystery',
        },
        'command-story-3',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenNthCalledWith(
      1,
      '/api/commands/story/create-arc',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-story-1',
          payload: {
            arc_id: 'arc.mystery',
            parent_arc_id: null,
            name: 'Mystery',
            description: 'Central investigation',
            arc_type: 'APlot',
            color: { r: 1, g: 2, b: 3 },
          },
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      2,
      '/api/commands/story/update-arc',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-story-2',
          payload: {
            arc_id: 'arc.mystery',
            name: 'Mystery revised',
          },
        }),
      }),
    );
    expect(fetchMock).toHaveBeenNthCalledWith(
      3,
      '/api/commands/story/delete-arc',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-story-3',
          payload: {
            arc_id: 'arc.mystery',
          },
        }),
      }),
    );
  });

  it('uses desktop story arc commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 3,
        payload: {
          arcs: [],
        },
      },
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(response)
      .mockResolvedValueOnce(response)
      .mockResolvedValueOnce(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await createStoryArc(
      {
        arc_id: 'arc.mystery',
        parent_arc_id: null,
        name: 'Mystery',
        description: 'Central investigation',
        arc_type: 'APlot',
        color: { r: 1, g: 2, b: 3 },
      },
      'command-story-1',
    );
    await setStoryArcMetadata(
      {
        arc_id: 'arc.mystery',
        name: 'Mystery revised',
      },
      'command-story-2',
    );
    await deleteStoryArc(
      {
        arc_id: 'arc.mystery',
      },
      'command-story-3',
    );

    expect(invoke).toHaveBeenNthCalledWith(1, 'command_story_create', {
      command: expect.objectContaining({ id: 'command-story-1' }),
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'command_story_update', {
      command: expect.objectContaining({ id: 'command-story-2' }),
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'command_story_delete', {
      command: expect.objectContaining({ id: 'command-story-3' }),
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('sends timeline create relationship commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [],
          clips: [],
          relationships: [
            {
              relationship_id: 'relationship.theme',
              from_node_id: 'node.scene.beach',
              to_node_id: 'node.scene.arrival',
              relationship_type: 'Thematic',
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createTimelineRelationship(
        {
          relationship_id: 'relationship.theme',
          from_node_id: 'node.scene.beach',
          to_node_id: 'node.scene.arrival',
          relationship_type: 'Thematic',
        },
        'command-timeline-relationship-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/create-relationship',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-relationship-1',
          payload: {
            relationship_id: 'relationship.theme',
            from_node_id: 'node.scene.beach',
            to_node_id: 'node.scene.arrival',
            relationship_type: 'Thematic',
          },
        }),
      }),
    );
  });

  it('sends timeline delete relationship commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [],
          clips: [],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      deleteTimelineRelationship(
        {
          relationship_id: 'relationship.theme',
        },
        'command-timeline-relationship-delete-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/delete-relationship',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-relationship-delete-1',
          payload: {
            relationship_id: 'relationship.theme',
          },
        }),
      }),
    );
  });

  it('sends timeline node lock commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [],
          clips: [
            {
              node_id: 'node.scene.beach',
              parent_id: null,
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Beach argument',
              start_ms: 1_000,
              end_ms: 4_000,
              sort_order: 10,
              locked: true,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setTimelineNodeLock(
        {
          node_id: 'node.scene.beach',
          locked: true,
        },
        'command-timeline-lock-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/node-lock',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-lock-1',
          payload: {
            node_id: 'node.scene.beach',
            locked: true,
          },
        }),
      }),
    );
  });

  it('sends timeline node notes commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [],
          clips: [
            {
              node_id: 'node.scene.beach',
              parent_id: null,
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Beach argument',
              start_ms: 1_000,
              end_ms: 4_000,
              sort_order: 10,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setTimelineNodeNotes(
        {
          node_id: 'node.scene.beach',
          notes: 'New outline',
        },
        'command-timeline-notes-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/node-notes',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-notes-1',
          payload: {
            node_id: 'node.scene.beach',
            notes: 'New outline',
          },
        }),
      }),
    );
  });

  it('sends timeline split node commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [
            {
              track_id: 'track.scene',
              level: 'Scene',
              label: 'Scenes',
              sort_order: 30,
              collapsed: false,
            },
          ],
          clips: [
            {
              node_id: 'node.scene.beach.a',
              parent_id: null,
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Beach argument A',
              start_ms: 1_000,
              end_ms: 2_500,
              sort_order: 10,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
            {
              node_id: 'node.scene.beach.b',
              parent_id: null,
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Beach argument B',
              start_ms: 2_500,
              end_ms: 4_000,
              sort_order: 11,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      splitTimelineNode(
        {
          node_id: 'node.scene.beach',
          at_ms: 2_500,
          left_node_id: 'node.scene.beach.a',
          right_node_id: 'node.scene.beach.b',
        },
        'command-timeline-split-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/split-node',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-split-1',
          payload: {
            node_id: 'node.scene.beach',
            at_ms: 2_500,
            left_node_id: 'node.scene.beach.a',
            right_node_id: 'node.scene.beach.b',
          },
        }),
      }),
    );
  });

  it('sends timeline delete node commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [],
          clips: [],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      deleteTimelineNode(
        {
          node_id: 'node.scene.beach',
        },
        'command-timeline-delete-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/delete-node',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-delete-1',
          payload: {
            node_id: 'node.scene.beach',
          },
        }),
      }),
    );
  });

  it('sends timeline create node commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [
            {
              track_id: 'track.scene',
              level: 'Scene',
              label: 'Scenes',
              sort_order: 30,
              collapsed: false,
            },
          ],
          clips: [
            {
              node_id: 'node.scene.new',
              parent_id: 'node.sequence.opening',
              track_id: 'track.scene',
              level: 'Scene',
              name: 'New Scene',
              start_ms: 1_000,
              end_ms: 4_000,
              sort_order: 10,
              locked: false,
              content_status: 'Empty',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createTimelineNode(
        {
          node_id: 'node.scene.new',
          parent_id: 'node.sequence.opening',
          level: 'Scene',
          name: 'New Scene',
          start_ms: 1_000,
          end_ms: 4_000,
          beat_type: null,
        },
        'command-timeline-create-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/create-node',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-create-1',
          payload: {
            node_id: 'node.scene.new',
            parent_id: 'node.sequence.opening',
            level: 'Scene',
            name: 'New Scene',
            start_ms: 1_000,
            end_ms: 4_000,
            beat_type: null,
          },
        }),
      }),
    );
  });

  it('sends timeline apply children commands and returns timeline render projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 1,
        payload: {
          total_duration_ms: 120_000,
          tracks: [
            {
              track_id: 'track.scene',
              level: 'Scene',
              label: 'Scenes',
              sort_order: 30,
              collapsed: false,
            },
          ],
          clips: [
            {
              node_id: 'node.scene.first',
              parent_id: 'node.sequence.opening',
              track_id: 'track.scene',
              level: 'Scene',
              name: 'First child',
              start_ms: 1_000,
              end_ms: 2_500,
              sort_order: 0,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
            {
              node_id: 'node.scene.second',
              parent_id: 'node.sequence.opening',
              track_id: 'track.scene',
              level: 'Scene',
              name: 'Second child',
              start_ms: 2_500,
              end_ms: 4_000,
              sort_order: 1,
              locked: false,
              content_status: 'NotesOnly',
              beat_type: null,
              arc_ids: [],
            },
          ],
          relationships: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      applyTimelineChildren(
        {
          parent_id: 'node.sequence.opening',
          child_plan_id: 'child_plan.generated',
          children: [
            {
              node_id: 'node.scene.first',
              name: 'First child',
              outline: 'First outline',
              weight: 1,
              beat_type: null,
            },
            {
              node_id: 'node.scene.second',
              name: 'Second child',
              outline: 'Second outline',
              weight: 1,
              beat_type: null,
            },
          ],
        },
        'command-timeline-children-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/timeline/apply-children',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-timeline-children-1',
          payload: {
            parent_id: 'node.sequence.opening',
            child_plan_id: 'child_plan.generated',
            children: [
              {
                node_id: 'node.scene.first',
                name: 'First child',
                outline: 'First outline',
                weight: 1,
                beat_type: null,
              },
              {
                node_id: 'node.scene.second',
                name: 'Second child',
                outline: 'Second outline',
                weight: 1,
                beat_type: null,
              },
            ],
          },
        }),
      }),
    );
  });

  it('sends bible graph node create commands and returns versioned projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 2,
        change_event_id: 'event-1',
        payload: {
          node: {
            id: 'node.character.ada',
            parent_id: null,
            schema_key: 'character',
            name: 'Ada',
            system_owned: false,
            sort_order: 3,
          },
          parts: [],
          incoming_edges: [],
          outgoing_edges: [],
          snapshots: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createBibleGraphNode(
        {
          node_id: 'node.character.ada',
          parent_id: null,
          schema_key: 'character',
          name: 'Ada',
          sort_order: 3,
        },
        'command-graph-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/bible-graph/node',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-graph-1',
          payload: {
            node_id: 'node.character.ada',
            parent_id: null,
            schema_key: 'character',
            name: 'Ada',
            sort_order: 3,
          },
        }),
      }),
    );
  });

  it('sends canonical bible root commands and returns node list projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 9,
        change_event_id: 'event-roots',
        payload: {
          nodes: [
            {
              id: 'canonical.characters',
              parent_id: null,
              schema_key: 'canonical.root.characters',
              name: 'Characters',
              system_owned: true,
              sort_order: 0,
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(ensureCanonicalBibleRoots('command-roots-1')).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/bible-graph/canonical-roots',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-roots-1',
          payload: {},
        }),
      }),
    );
  });

  it('sends bible graph field commands and returns versioned node projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        change_event_id: 'event-field-1',
        payload: {
          node: {
            id: 'node.location.harbor',
            parent_id: 'canonical.locations',
            schema_key: 'location',
            name: 'Harbor',
            system_owned: false,
            sort_order: 2,
          },
          parts: [
            {
              part: {
                id: 'part.location.environment',
                node_id: 'node.location.harbor',
                part_key: 'environment',
                name: 'Environment',
                system_owned: false,
                sort_order: 10,
              },
              fields: [
                {
                  id: 'field.location.weather',
                  part_id: 'part.location.environment',
                  field_key: 'weather',
                  value: { type: 'text', value: 'rainy' },
                  sort_order: 20,
                },
              ],
            },
          ],
          incoming_edges: [],
          outgoing_edges: [],
          snapshots: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setBibleGraphField(
        {
          node_id: 'node.location.harbor',
          part_id: 'part.location.environment',
          part_key: 'environment',
          part_name: 'Environment',
          part_sort_order: 10,
          field_id: 'field.location.weather',
          field_key: 'weather',
          value: { type: 'text', value: 'rainy' },
          field_sort_order: 20,
        },
        'command-field-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/bible-graph/field',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-field-1',
          payload: {
            node_id: 'node.location.harbor',
            part_id: 'part.location.environment',
            part_key: 'environment',
            part_name: 'Environment',
            part_sort_order: 10,
            field_id: 'field.location.weather',
            field_key: 'weather',
            value: { type: 'text', value: 'rainy' },
            field_sort_order: 20,
          },
        }),
      }),
    );
  });

  it('sends bible graph edge commands and returns versioned source node projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 5,
        change_event_id: 'event-edge-1',
        payload: {
          node: {
            id: 'node.character.ada',
            parent_id: 'canonical.characters',
            schema_key: 'character',
            name: 'Ada',
            system_owned: false,
            sort_order: 3,
          },
          parts: [],
          incoming_edges: [],
          outgoing_edges: [
            {
              id: 'edge.ada.harbor',
              from_node_id: 'node.character.ada',
              to_node_id: 'node.location.harbor',
              edge_kind: 'located_in',
              label: 'located in',
              directed: true,
              sort_order: 1,
            },
          ],
          snapshots: [],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setBibleGraphEdge(
        {
          edge_id: 'edge.ada.harbor',
          from_node_id: 'node.character.ada',
          to_node_id: 'node.location.harbor',
          edge_kind: 'located_in',
          label: 'located in',
          directed: true,
          sort_order: 1,
        },
        'command-edge-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/bible-graph/edge',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-edge-1',
          payload: {
            edge_id: 'edge.ada.harbor',
            from_node_id: 'node.character.ada',
            to_node_id: 'node.location.harbor',
            edge_kind: 'located_in',
            label: 'located in',
            directed: true,
            sort_order: 1,
          },
        }),
      }),
    );
  });

  it('sends bible graph snapshot field commands and returns versioned node projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 6,
        change_event_id: 'event-snapshot-1',
        payload: {
          node: {
            id: 'node.character.ada',
            parent_id: 'canonical.characters',
            schema_key: 'character',
            name: 'Ada',
            system_owned: false,
            sort_order: 3,
          },
          parts: [],
          incoming_edges: [],
          outgoing_edges: [],
          snapshots: [
            {
              snapshot: {
                id: 'snapshot.character.ada.sequence-1',
                node_id: 'node.character.ada',
                at_ms: 12000,
                label: 'Sequence 1 state',
                sort_order: 1,
              },
              fields: [
                {
                  id: 'snapshot-field.character.status',
                  snapshot_id: 'snapshot.character.ada.sequence-1',
                  part_key: 'profile',
                  part_name: 'Profile',
                  field_key: 'tagline',
                  value: { type: 'text', value: 'Rain-soaked' },
                  sort_order: 2,
                },
              ],
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setBibleGraphSnapshotField(
        {
          snapshot_id: 'snapshot.character.ada.sequence-1',
          node_id: 'node.character.ada',
          at_ms: 12000,
          label: 'Sequence 1 state',
          snapshot_sort_order: 1,
          field_id: 'snapshot-field.character.status',
          part_key: 'profile',
          part_name: 'Profile',
          field_key: 'tagline',
          value: { type: 'text', value: 'Rain-soaked' },
          field_sort_order: 2,
        },
        'command-snapshot-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/bible-graph/snapshot-field',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-snapshot-1',
          payload: {
            snapshot_id: 'snapshot.character.ada.sequence-1',
            node_id: 'node.character.ada',
            at_ms: 12000,
            label: 'Sequence 1 state',
            snapshot_sort_order: 1,
            field_id: 'snapshot-field.character.status',
            part_key: 'profile',
            part_name: 'Profile',
            field_key: 'tagline',
            value: { type: 'text', value: 'Rain-soaked' },
            field_sort_order: 2,
          },
        }),
      }),
    );
  });
});
