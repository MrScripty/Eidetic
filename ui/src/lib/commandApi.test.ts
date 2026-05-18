import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createBibleGraphNode,
  ensureCanonicalBibleRoots,
  setBibleGraphField,
  setObjectField,
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
});
