import { describe, expect, it } from 'vitest';

import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';
import { graphRendererWindowStatusDisplay } from './graphRendererWindowStatus.js';

function statusFor(
  renderer_window_lifecycle: GraphRendererStatus['renderer_window_lifecycle'],
  renderer_window_visible_supported = renderer_window_lifecycle === 'visible',
  renderer_window_capability: GraphRendererStatus['renderer_window_capability'] = renderer_window_visible_supported
    ? 'verified_support'
    : 'platform_unproven',
  renderer_window_capability_reason: GraphRendererStatus['renderer_window_capability_reason'] = renderer_window_visible_supported
    ? 'verified_support'
    : 'platform_unproven',
  renderer_window_verified_support = renderer_window_capability === 'verified_support',
): GraphRendererStatus {
  return {
    renderer_window_kind: 'bible_graph',
    running: renderer_window_lifecycle !== 'closed',
    renderer_window_open: renderer_window_lifecycle !== 'closed',
    renderer_scene_ready:
      renderer_window_lifecycle === 'scene_ready_pending_native_runner' ||
      renderer_window_lifecycle === 'visible',
    renderer_window_visible: renderer_window_lifecycle === 'visible',
    renderer_window_strategy: 'bevy_winit_floating_window',
    renderer_window_platform: 'linux',
    renderer_runner_lifecycle: 'open_requested',
    renderer_supervisor_lifecycle: 'starting',
    renderer_runner_threading_model: 'worker_thread',
    renderer_window_capability,
    renderer_window_capability_reason,
    renderer_window_lifecycle,
    renderer_window_ready: renderer_window_lifecycle === 'visible',
    renderer_window_verified_support,
    renderer_window_visible_supported,
    renderer_window_focus_supported: renderer_window_lifecycle === 'visible',
    renderer_window_message: `message:${renderer_window_lifecycle}`,
    node_count: 0,
    edge_count: 0,
    native_visual_node_count: 0,
    native_visual_edge_count: 0,
    renderer_window_width_px: 0,
    renderer_window_height_px: 0,
    influence_count: 0,
    workspace_timeline_track_count: 0,
    workspace_timeline_clip_count: 0,
    workspace_timeline_relationship_count: 0,
    workspace_timeline_affect_overlay_count: 0,
    workspace_timeline_total_duration_ms: 0,
    last_error: null,
  };
}

describe('graph renderer window status display', () => {
  it('uses a closed display before backend status loads', () => {
    expect(graphRendererWindowStatusDisplay(null)).toEqual({
      label: 'Renderer closed',
      active: false,
      message: 'Graph renderer window is closed',
      primaryActionLabel: 'Open Graph Window',
      nativeWindowAvailable: false,
    });
  });

  it('derives display state from backend lifecycle status', () => {
    expect(graphRendererWindowStatusDisplay(statusFor('closed'))).toEqual({
      label: 'Renderer closed',
      active: false,
      message: 'message:closed',
      primaryActionLabel: 'Open Graph Window',
      nativeWindowAvailable: false,
    });
    expect(graphRendererWindowStatusDisplay(statusFor('scene_starting'))).toEqual({
      label: 'Renderer preparing',
      active: false,
      message: 'message:scene_starting',
      primaryActionLabel: 'Preparing',
      nativeWindowAvailable: false,
    });
    expect(
      graphRendererWindowStatusDisplay(statusFor('scene_ready_pending_native_runner')),
    ).toEqual({
      label: 'Renderer unavailable',
      active: false,
      message: 'message:scene_ready_pending_native_runner',
      primaryActionLabel: 'Renderer Prepared',
      nativeWindowAvailable: false,
    });
    expect(
      graphRendererWindowStatusDisplay(
        statusFor('scene_ready_pending_native_runner', true, 'verified_support'),
      ),
    ).toEqual({
      label: 'Renderer waiting',
      active: false,
      message: 'message:scene_ready_pending_native_runner',
      primaryActionLabel: 'Focus Bevy Window',
      nativeWindowAvailable: true,
    });
    expect(graphRendererWindowStatusDisplay(statusFor('visible'))).toEqual({
      label: 'Renderer visible',
      active: true,
      message: 'message:visible',
      primaryActionLabel: 'Focus Bevy Window',
      nativeWindowAvailable: true,
    });
  });

  it('uses backend capability instead of inferring unsupported state', () => {
    expect(
      graphRendererWindowStatusDisplay(
        statusFor(
          'scene_ready_pending_native_runner',
          false,
          'platform_unsupported',
          'platform_unsupported',
        ),
      ),
    ).toEqual({
      label: 'Renderer unsupported',
      active: false,
      message: 'message:scene_ready_pending_native_runner',
      primaryActionLabel: 'Renderer Prepared',
      nativeWindowAvailable: false,
    });
    expect(
      graphRendererWindowStatusDisplay(
        statusFor('scene_ready_pending_native_runner', true, 'runner_error', 'runner_error'),
      ),
    ).toEqual({
      label: 'Renderer error',
      active: false,
      message: 'message:scene_ready_pending_native_runner',
      primaryActionLabel: 'Renderer Prepared',
      nativeWindowAvailable: false,
    });
  });
});
