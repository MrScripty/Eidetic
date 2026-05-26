import { describe, expect, it } from 'vitest';

import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';
import {
  timelineRendererStatusLabel,
  timelineRendererWindowControlState,
} from './timelineRendererWindowControls.js';

function status(overrides: Partial<TimelineRendererStatus> = {}): TimelineRendererStatus {
  return {
    renderer_window_kind: 'timeline',
    running: false,
    renderer_window_open: false,
    renderer_scene_ready: false,
    renderer_window_lifecycle: 'closed',
    renderer_runner_lifecycle: 'closed',
    renderer_runner_threading_model: 'worker_thread',
    renderer_window_strategy: 'bevy_winit_floating_window',
    renderer_window_platform: 'linux',
    renderer_window_capability: 'pending_native_runner',
    renderer_window_capability_reason: 'pending_native_runner',
    renderer_window_visible: false,
    renderer_window_ready: false,
    renderer_window_verified_support: false,
    renderer_window_visible_supported: false,
    renderer_window_focus_supported: false,
    renderer_window_message: 'timeline renderer is closed',
    track_count: 0,
    clip_count: 0,
    relationship_count: 0,
    affect_overlay_count: 0,
    queued_command_count: 0,
    last_error: null,
    ...overrides,
  };
}

describe('timeline renderer window controls', () => {
  it('derives status labels from backend status without frontend renderer state', () => {
    expect(timelineRendererStatusLabel(null)).toBe('not checked');
    expect(timelineRendererStatusLabel(status({ last_error: 'renderer failed' }))).toBe(
      'renderer failed',
    );
    expect(
      timelineRendererStatusLabel(
        status({
          renderer_window_ready: true,
          clip_count: 4,
          renderer_window_message: 'ready',
        }),
      ),
    ).toBe('4 clips');
    expect(timelineRendererStatusLabel(status({ renderer_window_message: 'pending' }))).toBe(
      'pending',
    );
  });

  it('keeps icon controls accessible and disables only unavailable actions', () => {
    expect(timelineRendererWindowControlState('open', null, false)).toEqual({
      action: 'open',
      label: 'Open timeline renderer',
      disabled: false,
    });
    expect(timelineRendererWindowControlState('refresh', null, true).disabled).toBe(true);
    expect(timelineRendererWindowControlState('focus', status({ running: true }), false)).toEqual({
      action: 'focus',
      label: 'Focus timeline renderer',
      disabled: true,
    });
    expect(
      timelineRendererWindowControlState(
        'focus',
        status({
          running: true,
          renderer_window_focus_supported: true,
        }),
        false,
      ).disabled,
    ).toBe(false);
    expect(
      timelineRendererWindowControlState('close', status({ running: false }), false).disabled,
    ).toBe(true);
    expect(
      timelineRendererWindowControlState('close', status({ running: true }), false).disabled,
    ).toBe(false);
  });
});
