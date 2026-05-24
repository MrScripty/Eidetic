import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  closeGraphRenderer,
  drainGraphRendererCommands,
  focusGraphRenderer,
  getGraphRendererStatus,
  getGraphRendererVisualSnapshot,
  openGraphRenderer,
  updateGraphRendererProjectionRequest,
} from './graphRendererApi.js';

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

describe('graph renderer api helpers', () => {
  it('uses the desktop graph renderer status command', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: false,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_window_capability: 'pending_native_runner',
      renderer_window_capability_reason: 'pending_native_runner',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_visible_supported: false,
      renderer_window_focus_supported: false,
      renderer_window_message:
        'graph renderer scene is ready; visible native window is pending implementation',
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      renderer_window_width_px: 1280,
      renderer_window_height_px: 720,
      influence_count: 1,
      last_error: null,
    };
    const invoke = installDesktopInvoke(response);

    await expect(getGraphRendererStatus()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_status', undefined);
  });

  it('opens the desktop graph renderer window with a bounded projection request', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: false,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_window_capability: 'pending_native_runner',
      renderer_window_capability_reason: 'pending_native_runner',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_visible_supported: false,
      renderer_window_focus_supported: false,
      renderer_window_message:
        'graph renderer scene is ready; visible native window is pending implementation',
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      renderer_window_width_px: 0,
      renderer_window_height_px: 0,
      influence_count: 1,
      last_error: null,
    };
    const request = {
      graph_projection_request: {
        selected_timeline_node_id: 'node.scene.beach',
        neighborhood_depth: 1,
        max_nodes: 200,
      },
      renderer_window_size_hint: {
        width_px: 1280,
        height_px: 720,
      },
    };
    const invoke = installDesktopInvoke(response);

    await expect(openGraphRenderer(request)).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_open', { request });
  });

  it('updates the desktop graph renderer projection request', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: false,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_window_capability: 'pending_native_runner',
      renderer_window_capability_reason: 'pending_native_runner',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_visible_supported: false,
      renderer_window_focus_supported: false,
      renderer_window_message:
        'graph renderer scene is ready; visible native window is pending implementation',
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      renderer_window_width_px: 0,
      renderer_window_height_px: 0,
      influence_count: 1,
      last_error: null,
    };
    const request = {
      selected_timeline_node_id: 'node.scene.beach',
      selected_node_id: 'node.character.ada',
      neighborhood_depth: 1,
      max_nodes: 200,
    };
    const invoke = installDesktopInvoke(response);

    await expect(updateGraphRendererProjectionRequest(request)).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_update_projection_request', {
      request,
    });
  });

  it('focuses and closes the desktop graph renderer window', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: false,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_window_capability: 'pending_native_runner',
      renderer_window_capability_reason: 'pending_native_runner',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_visible_supported: false,
      renderer_window_focus_supported: false,
      renderer_window_message:
        'graph renderer scene is ready; visible native window is pending implementation',
      node_count: 0,
      edge_count: 0,
      native_visual_node_count: 0,
      native_visual_edge_count: 0,
      renderer_window_width_px: 0,
      renderer_window_height_px: 0,
      influence_count: 0,
      last_error: null,
    };
    const invoke = installDesktopInvoke(response);

    await expect(focusGraphRenderer()).resolves.toEqual(response);
    await expect(closeGraphRenderer()).resolves.toEqual(response);

    expect(invoke).toHaveBeenNthCalledWith(1, 'graph_renderer_focus', undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, 'graph_renderer_close', undefined);
  });

  it('uses the desktop graph renderer command drain', async () => {
    const response = [{ type: 'select_edge', edge_id: 'edge.ada.beach' }];
    const invoke = installDesktopInvoke(response);

    await expect(drainGraphRendererCommands()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_drain_commands', undefined);
  });

  it('uses the desktop graph renderer visual snapshot command', async () => {
    const response = {
      nodes: [
        {
          node_id: 'node.character.ada',
          label: 'Ada',
          position: { x: 0, y: 0, z: 0 },
          radius: 19,
          fill_color: '#1f6f78',
          outline_color: '#f2c94c',
          highlighted: true,
        },
      ],
      edges: [],
    };
    const invoke = installDesktopInvoke(response);

    await expect(getGraphRendererVisualSnapshot()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_visual_snapshot', undefined);
  });
});
