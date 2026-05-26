import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  applyTimelineChildren,
  createAffectProposal,
  createBibleGraphNode,
  deleteBibleGraphEdge,
  deleteBibleGraphNode,
  createStoryArc,
  createTimelineNode,
  createTimelineRelationship,
  deleteStoryArc,
  deleteTimelineNode,
  deleteTimelineRelationship,
  ensureCanonicalBibleRoots,
  recordContextEvaluation,
  setAffectValue,
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
  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

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
    ).rejects.toThrow('desktop transport is unavailable');

    expect(fetch).not.toHaveBeenCalled();
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

  it('uses desktop affect set commands when Tauri transport is available', async () => {
    const response = {
      version: 2,
      payload: {
        target: { type: 'timeline_node', node_id: 'node.scene.beach' },
        values: [],
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });

    const payload = {
      affect_id: 'affect-1',
      target: { type: 'timeline_node' as const, node_id: 'node.scene.beach' },
      valence: -250,
      arousal: 650,
      intensity: 700,
      confidence: 900,
      mood_labels: ['uneasy'],
      provenance: 'user_authored' as const,
      rationale: 'Opening mood',
    };

    await expect(setAffectValue(payload, 'command-affect-1')).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_affect_set', {
      command: {
        id: 'command-affect-1',
        payload: {
          command_id: 'command-affect-1',
          ...payload,
        },
      },
    });
  });

  it('uses desktop affect proposal create commands when Tauri transport is available', async () => {
    const response = {
      version: 2,
      payload: {
        proposals: [],
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });

    const payload = {
      proposal_id: 'proposal.affect.scene-weather',
      source: 'manual_script_edit' as const,
      proposed_value: {
        id: 'affect-1',
        target: { type: 'project' as const },
        valence: -100,
        arousal: 650,
        intensity: 700,
        confidence: 900,
        mood_labels: ['rainy'],
        provenance: 'script_edit_detected' as const,
        rationale: 'User changed weather in generated script.',
      },
      summary: 'Detected rainy scene affect',
      rationale: 'Manual script text changed from sunny to rainy.',
      source_event_id: null,
    };

    await expect(createAffectProposal(payload, 'command-affect-proposal-1')).resolves.toEqual(
      response,
    );

    expect(invoke).toHaveBeenCalledWith('command_affect_proposal_create', {
      command: {
        id: 'command-affect-proposal-1',
        payload,
      },
    });
  });

  it('uses desktop timeline range commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_node_range', {
      command: {
        id: 'command-timeline-1',
        payload: {
          node_id: 'node.scene.beach',
          start_ms: 1_000,
          end_ms: 4_000,
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop context evaluation commands when Tauri transport is available', async () => {
    const response = {
      version: 2,
      payload: {
        target_node_id: 'timeline-node-1',
        evaluation_id: 'evaluation-1',
        task_kind: 'generate_timeline_context',
        records: [],
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });

    const payload = {
      evaluation: {
        id: 'evaluation-1',
        target_node_id: 'timeline-node-1',
        task_kind: 'generate_timeline_context' as const,
        summary: 'Scene context',
        distilled_context: 'Harbor context',
        created_at_ms: 100,
      },
      influences: [],
    };

    await expect(recordContextEvaluation(payload, 'command-context-1')).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_context_evaluation', {
      command: {
        id: 'command-context-1',
        payload,
      },
    });
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

  it('uses desktop timeline create relationship commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_create_relationship', {
      command: {
        id: 'command-timeline-relationship-1',
        payload: {
          relationship_id: 'relationship.theme',
          from_node_id: 'node.scene.beach',
          to_node_id: 'node.scene.arrival',
          relationship_type: 'Thematic',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline delete relationship commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      deleteTimelineRelationship(
        {
          relationship_id: 'relationship.theme',
        },
        'command-timeline-relationship-delete-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_timeline_delete_relationship', {
      command: {
        id: 'command-timeline-relationship-delete-1',
        payload: {
          relationship_id: 'relationship.theme',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline node lock commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_node_lock', {
      command: {
        id: 'command-timeline-lock-1',
        payload: {
          node_id: 'node.scene.beach',
          locked: true,
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline node notes commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_node_notes', {
      command: {
        id: 'command-timeline-notes-1',
        payload: {
          node_id: 'node.scene.beach',
          notes: 'New outline',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline split node commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_split_node', {
      command: {
        id: 'command-timeline-split-1',
        payload: {
          node_id: 'node.scene.beach',
          at_ms: 2_500,
          left_node_id: 'node.scene.beach.a',
          right_node_id: 'node.scene.beach.b',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline delete node commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      deleteTimelineNode(
        {
          node_id: 'node.scene.beach',
        },
        'command-timeline-delete-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_timeline_delete_node', {
      command: {
        id: 'command-timeline-delete-1',
        payload: {
          node_id: 'node.scene.beach',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline create node commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_timeline_create_node', {
      command: {
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
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop timeline apply children commands when Tauri transport is available', async () => {
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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
          ],
        },
        'command-timeline-children-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_timeline_apply_children', {
      command: {
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
          ],
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph node create command when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 2,
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
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
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

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_node', {
      command: {
        id: 'command-graph-1',
        payload: {
          node_id: 'node.character.ada',
          parent_id: null,
          schema_key: 'character',
          name: 'Ada',
          sort_order: 3,
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop canonical bible roots command when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 9,
        payload: {
          nodes: [],
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

    await expect(ensureCanonicalBibleRoots('command-roots-1')).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_roots', {
      command: {
        id: 'command-roots-1',
        payload: {},
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph node delete command when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 10,
        payload: {
          nodes: [],
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
      deleteBibleGraphNode(
        {
          node_id: 'node.character.ada',
        },
        'command-delete-node-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_delete_node', {
      command: {
        id: 'command-delete-node-1',
        payload: {
          node_id: 'node.character.ada',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph field commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        payload: {
          node: {
            id: 'node.location.harbor',
            parent_id: 'canonical.locations',
            schema_key: 'location',
            name: 'Harbor',
            system_owned: false,
            sort_order: 2,
          },
          parts: [],
          incoming_edges: [],
          outgoing_edges: [],
          snapshots: [],
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

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_field', {
      command: {
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
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph edge commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 5,
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
          snapshots: [],
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

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_edge', {
      command: {
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
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph edge delete commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 6,
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
          snapshots: [],
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
      deleteBibleGraphEdge(
        {
          edge_id: 'edge.ada.harbor',
        },
        'command-delete-edge-1',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_delete_edge', {
      command: {
        id: 'command-delete-edge-1',
        payload: {
          edge_id: 'edge.ada.harbor',
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('uses desktop bible graph snapshot field commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 6,
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
          snapshots: [],
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

    expect(invoke).toHaveBeenCalledWith('command_bible_graph_snapshot_field', {
      command: {
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
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
