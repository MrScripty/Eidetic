import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
  getBibleGraphSchemaListProjection,
  getObjectFieldProjection,
  getStoryArcListProjection,
  getStoryArcProgressionProjection,
  getTimelineRenderProjection,
} from './projectionApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('projection api helpers', () => {
  it('fetches object field projections with encoded query params', async () => {
    const response = {
      version: 2,
      change_event_id: 'event-1',
      payload: {
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
        deleted: false,
        fields: {
          weather: { type: 'text', value: 'rainy' },
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
      getObjectFieldProjection({
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
      }),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/projections/object-field?object_kind=bible_part_field&object_id=field%2Fweather+one',
      {
        method: 'GET',
        headers: { Accept: 'application/json' },
      },
    );
  });

  it('throws backend errors without patching local state', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'projection unavailable' }), {
          status: 404,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(
      getObjectFieldProjection({
        object_kind: 'bible_part_field',
        object_id: 'field-weather',
      }),
    ).rejects.toThrow('projection unavailable');
  });

  it('fetches bible graph node projections with encoded query params', async () => {
    const response = {
      version: 2,
      change_event_id: 'event-1',
      payload: {
        node: {
          id: 'node.character/ada one',
          parent_id: null,
          schema_key: 'character',
          name: 'Ada',
          system_owned: false,
          sort_order: 3,
        },
        parts: [],
        incoming_edges: [],
        outgoing_edges: [],
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
      getBibleGraphNodeProjection({
        node_id: 'node.character/ada one',
      }),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/projections/bible-graph/node?node_id=node.character%2Fada+one',
      {
        method: 'GET',
        headers: { Accept: 'application/json' },
      },
    );
  });

  it('fetches bible graph node list projections without query params', async () => {
    const response = {
      version: 3,
      change_event_id: 'event-2',
      payload: {
        nodes: [
          {
            id: 'node.character.ada',
            parent_id: null,
            schema_key: 'character',
            name: 'Ada',
            system_owned: false,
            sort_order: 3,
          },
        ],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getBibleGraphNodeListProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/bible-graph/nodes', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('fetches bible graph schema list projections without query params', async () => {
    const response = {
      version: 1,
      payload: {
        schemas: [
          {
            schema_key: 'character',
            parts: [
              {
                part_key: 'profile',
                name: 'Profile',
                sort_order: 10,
                fields: [
                  {
                    field_key: 'summary',
                    sort_order: 10,
                  },
                  {
                    field_key: 'tagline',
                    sort_order: 20,
                  },
                ],
              },
            ],
          },
        ],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getBibleGraphSchemaListProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/bible-graph/schemas', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('fetches timeline render projections without query params', async () => {
    const response = {
      version: 4,
      change_event_id: 'event-3',
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
            parent_id: 'node.sequence.opening',
            track_id: 'track.scene',
            level: 'Scene',
            name: 'Beach argument',
            start_ms: 1_000,
            end_ms: 4_000,
            sort_order: 10,
            locked: false,
            content_status: 'NotesOnly',
            beat_type: null,
            arc_ids: ['arc.a'],
          },
        ],
        relationships: [
          {
            relationship_id: 'rel.theme',
            from_node_id: 'node.scene.beach',
            to_node_id: 'node.scene.beach',
            relationship_type: 'Thematic',
          },
        ],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getTimelineRenderProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/timeline/render', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('fetches story arc list projections without query params', async () => {
    const response = {
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
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getStoryArcListProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/story/arcs', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });

  it('fetches story arc progression projections without query params', async () => {
    const response = {
      version: 1,
      payload: {
        progressions: [
          {
            arc_id: 'arc.mystery',
            arc_name: 'Mystery',
            node_count: 4,
            has_setup: true,
            has_resolution: false,
            coverage_percent: 24,
            longest_gap_ms: 60_000,
            issues: [],
          },
        ],
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(getStoryArcProgressionProjection()).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith('/api/projections/story/arc-progression', {
      method: 'GET',
      headers: { Accept: 'application/json' },
    });
  });
});
