import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  applyGraphRendererCameraCommand,
  applyGraphRendererTextEditorSettings,
  closeGraphRenderer,
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
      renderer_runner_lifecycle: 'open_requested',
      renderer_supervisor_lifecycle: 'starting',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'platform_unproven',
      renderer_window_capability_reason: 'platform_unproven',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_verified_support: false,
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
      renderer_runner_lifecycle: 'open_requested',
      renderer_supervisor_lifecycle: 'starting',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'platform_unproven',
      renderer_window_capability_reason: 'platform_unproven',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_verified_support: false,
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
      renderer_runner_lifecycle: 'open_requested',
      renderer_supervisor_lifecycle: 'starting',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'platform_unproven',
      renderer_window_capability_reason: 'platform_unproven',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_verified_support: false,
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
      renderer_runner_lifecycle: 'open_requested',
      renderer_supervisor_lifecycle: 'starting',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'platform_unproven',
      renderer_window_capability_reason: 'platform_unproven',
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_ready: false,
      renderer_window_verified_support: false,
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

  it('sends backend-owned camera commands to the desktop graph renderer', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: true,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_runner_lifecycle: 'visible',
      renderer_supervisor_lifecycle: 'running',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'verified_support',
      renderer_window_capability_reason: 'verified_support',
      renderer_window_lifecycle: 'visible',
      renderer_window_ready: true,
      renderer_window_verified_support: true,
      renderer_window_visible_supported: true,
      renderer_window_focus_supported: false,
      renderer_window_message: 'graph renderer native window is ready',
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      renderer_window_width_px: 1280,
      renderer_window_height_px: 720,
      influence_count: 1,
      last_error: null,
    };
    const command = { type: 'frame_node' as const, node_id: 'node.character.ada' };
    const invoke = installDesktopInvoke(response);

    await expect(applyGraphRendererCameraCommand(command)).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_camera_command', { command });
  });

  it('sends text editor settings to the desktop graph renderer', async () => {
    const response = {
      renderer_window_kind: 'bible_graph',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_visible: true,
      renderer_window_strategy: 'bevy_winit_floating_window',
      renderer_window_platform: 'linux',
      renderer_runner_lifecycle: 'visible',
      renderer_supervisor_lifecycle: 'running',
      renderer_runner_threading_model: 'worker_thread',
      renderer_window_capability: 'verified_support',
      renderer_window_capability_reason: 'verified_support',
      renderer_window_lifecycle: 'visible',
      renderer_window_ready: true,
      renderer_window_verified_support: true,
      renderer_window_visible_supported: true,
      renderer_window_focus_supported: false,
      renderer_window_message: 'graph renderer native window is ready',
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      renderer_window_width_px: 1280,
      renderer_window_height_px: 720,
      influence_count: 1,
      last_error: null,
    };
    const settings = {
      padding_px: 20,
      corner_radius_px: 8,
      outline_width_px: 2,
      outline_brightness: 0.75,
    };
    const invoke = installDesktopInvoke(response);

    await expect(applyGraphRendererTextEditorSettings(settings)).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_text_editor_settings', { settings });
  });
});
