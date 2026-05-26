import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  getAffectProjection,
  getBibleGraphNodeListProjection,
  getBibleGraphNodeProjection,
  getBibleGraphSchemaListProjection,
  getBibleReferenceProposalListProjection,
  getBibleRenderGraphProjection,
  getChangeReviewProjection,
  getContextInfluenceProjection,
  getContextStackProjection,
  getObjectFieldProjection,
  getPropagationProposalListProjection,
  getScriptDocumentProjection,
  getSelectedNodeEditorProjection,
  getStoryArcListProjection,
  getStoryArcProgressionProjection,
  getTimelineRenderProjection,
} from './projectionApi.js';

function installDesktopInvoke(response: unknown) {
  const invoke = vi.fn().mockResolvedValue(response);
  vi.stubGlobal('window', {
    __TAURI__: {
      core: { invoke },
    },
  });
  return invoke;
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('projection api helpers', () => {
  it('requires desktop transport instead of falling back to HTTP', async () => {
    vi.stubGlobal('fetch', vi.fn());

    await expect(
      getObjectFieldProjection({
        object_kind: 'bible_part_field',
        object_id: 'field-weather',
      }),
    ).rejects.toThrow('desktop transport is unavailable');

    expect(fetch).not.toHaveBeenCalled();
  });

  it('uses the desktop object field projection command', async () => {
    const response = { version: 2, payload: { fields: {} } };
    const invoke = installDesktopInvoke(response);

    await expect(
      getObjectFieldProjection({
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_object_field', {
      query: {
        object_kind: 'bible_part_field',
        object_id: 'field/weather one',
      },
    });
  });

  it('uses the desktop affect projection command', async () => {
    const response = {
      version: 2,
      payload: {
        target: { type: 'timeline_node', node_id: 'node.scene.beach' },
        values: [],
      },
    };
    const invoke = installDesktopInvoke(response);

    await expect(
      getAffectProjection({
        target: { type: 'timeline_node', node_id: 'node.scene.beach' },
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_affect', {
      query: {
        target: { type: 'timeline_node', node_id: 'node.scene.beach' },
      },
    });
  });

  it('uses the desktop bible graph node projection command', async () => {
    const response = { version: 2, payload: { node: { id: 'node.character/ada one' } } };
    const invoke = installDesktopInvoke(response);

    await expect(
      getBibleGraphNodeProjection({
        node_id: 'node.character/ada one',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_graph_node', {
      query: { node_id: 'node.character/ada one' },
    });
  });

  it('uses the desktop bible graph node list projection command', async () => {
    const response = { version: 3, payload: { nodes: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getBibleGraphNodeListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_graph_nodes', undefined);
  });

  it('uses the desktop bible graph schema projection command', async () => {
    const response = { version: 1, payload: { schemas: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getBibleGraphSchemaListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_graph_schemas', undefined);
  });

  it('uses the desktop bible render graph projection command', async () => {
    const response = {
      version: 4,
      payload: { nodes: [], edges: [], neighborhoods: [], influences: [] },
    };
    const invoke = installDesktopInvoke(response);

    await expect(getBibleRenderGraphProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_render_graph', undefined);
  });

  it('passes bounded bible render graph projection queries to desktop', async () => {
    const response = {
      version: 4,
      payload: { nodes: [], edges: [], neighborhoods: [], influences: [] },
    };
    const invoke = installDesktopInvoke(response);

    await expect(
      getBibleRenderGraphProjection({
        selected_node_id: 'node.character.ada',
        selected_timeline_node_id: 'timeline-node-1',
        neighborhood_depth: 1,
        max_nodes: 25,
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_render_graph', {
      query: {
        selected_node_id: 'node.character.ada',
        selected_timeline_node_id: 'timeline-node-1',
        neighborhood_depth: 1,
        max_nodes: 25,
      },
    });
  });

  it('uses the desktop context influence projection command', async () => {
    const response = {
      version: 2,
      payload: {
        target_node_id: 'timeline-node-1',
        evaluation_id: 'evaluation-1',
        task_kind: 'generate_timeline_context',
        records: [],
      },
    };
    const invoke = installDesktopInvoke(response);

    await expect(
      getContextInfluenceProjection({
        target_node_id: 'timeline-node-1',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_context_influence', {
      query: { target_node_id: 'timeline-node-1' },
    });
  });

  it('uses the desktop context stack projection command', async () => {
    const response = {
      version: 2,
      payload: {
        target_node_id: 'timeline-node-1',
        layers: [],
      },
    };
    const invoke = installDesktopInvoke(response);

    await expect(
      getContextStackProjection({
        target_node_id: 'timeline-node-1',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_context_stack', {
      query: { target_node_id: 'timeline-node-1' },
    });
  });

  it('uses the desktop script document projection command', async () => {
    const response = { version: 1, payload: { document_id: 'script.main' } };
    const invoke = installDesktopInvoke(response);

    await expect(
      getScriptDocumentProjection({
        document_id: 'script.main',
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_script_document', {
      query: { document_id: 'script.main' },
    });
  });

  it('uses the desktop bible reference proposal list projection command', async () => {
    const response = { version: 1, payload: { proposals: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getBibleReferenceProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_bible_reference_proposals', undefined);
  });

  it('uses the desktop propagation proposal list projection command', async () => {
    const response = { version: 1, payload: { proposals: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getPropagationProposalListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_propagation_proposals', undefined);
  });

  it('uses the desktop story arc list projection command', async () => {
    const response = { version: 1, payload: { arcs: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getStoryArcListProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_story_arcs', undefined);
  });

  it('uses the desktop story arc progression projection command', async () => {
    const response = { version: 1, payload: { progressions: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getStoryArcProgressionProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_story_arc_progression', undefined);
  });

  it('uses the desktop timeline render projection command', async () => {
    const response = {
      version: 4,
      payload: {
        total_duration_ms: 120_000,
        tracks: [],
        clips: [],
        relationships: [],
      },
    };
    const invoke = installDesktopInvoke(response);

    await expect(getTimelineRenderProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_timeline_render', undefined);
  });

  it('uses the desktop selected node projection command with a node id', async () => {
    const response = { version: 5, payload: { node: null } };
    const invoke = installDesktopInvoke(response);

    await expect(
      getSelectedNodeEditorProjection({ node_id: 'node.scene/beach one' }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_selected_node', {
      query: { node_id: 'node.scene/beach one' },
    });
  });

  it('uses the desktop selected node projection command with a null node id', async () => {
    const response = { version: 5, payload: { node: null } };
    const invoke = installDesktopInvoke(response);

    await expect(getSelectedNodeEditorProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_selected_node', {
      query: { node_id: null },
    });
  });

  it('uses the desktop change review projection command', async () => {
    const response = { version: 1, payload: { changes: [] } };
    const invoke = installDesktopInvoke(response);

    await expect(getChangeReviewProjection()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('projection_change_review', undefined);
  });
});
