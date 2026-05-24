import { beforeEach, describe, expect, it } from 'vitest';

import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';
import {
  clearGraphRendererWindowStatus,
  graphRendererWindowState,
  setGraphRendererWindowStatus,
} from './graphRendererWindow.svelte.js';

const baseStatus: GraphRendererStatus = {
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
  renderer_window_message: 'pending native runner',
  node_count: 0,
  edge_count: 0,
  native_visual_node_count: 0,
  native_visual_edge_count: 0,
  renderer_window_width_px: 0,
  renderer_window_height_px: 0,
  influence_count: 0,
  last_error: null,
};

beforeEach(() => {
  clearGraphRendererWindowStatus();
});

describe('graph renderer window status', () => {
  it('stores and clears backend-projected renderer status', () => {
    setGraphRendererWindowStatus(baseStatus);
    expect(graphRendererWindowState.status).toEqual(baseStatus);

    clearGraphRendererWindowStatus();
    expect(graphRendererWindowState.status).toBeNull();
  });
});
