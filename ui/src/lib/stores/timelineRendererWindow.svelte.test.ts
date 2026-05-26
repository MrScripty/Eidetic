import { describe, expect, it } from 'vitest';
import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';
import {
  clearTimelineRendererWindowStatus,
  setTimelineRendererWindowStatus,
  timelineRendererWindowState,
} from './timelineRendererWindow.svelte.js';

const baseStatus: TimelineRendererStatus = {
  renderer_window_kind: 'timeline',
  running: true,
  renderer_window_open: true,
  renderer_scene_ready: true,
  renderer_window_lifecycle: 'scene_ready_pending_native_runner',
  renderer_runner_lifecycle: 'closed',
  renderer_runner_threading_model: 'unsupported',
  renderer_window_platform: 'linux',
  renderer_window_capability: 'pending_native_runner',
  renderer_window_capability_reason: 'pending_native_runner',
  renderer_window_visible: false,
  renderer_window_ready: false,
  renderer_window_verified_support: false,
  renderer_window_visible_supported: false,
  renderer_window_focus_supported: false,
  renderer_window_message: 'timeline renderer scene is ready; native window is not connected',
  track_count: 5,
  clip_count: 13,
  relationship_count: 2,
  affect_overlay_count: 3,
  queued_command_count: 0,
  last_error: null,
};

describe('timeline renderer window state', () => {
  it('stores only the latest backend status projection', () => {
    clearTimelineRendererWindowStatus();

    setTimelineRendererWindowStatus(baseStatus);
    expect(timelineRendererWindowState.status).toEqual(baseStatus);

    clearTimelineRendererWindowStatus();
    expect(timelineRendererWindowState.status).toBeNull();
  });
});
