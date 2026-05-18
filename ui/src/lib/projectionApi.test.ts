import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
  getObjectFieldProjection,
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
});
