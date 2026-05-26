import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  closeTimelineRenderer,
  focusTimelineRenderer,
  getTimelineRendererStatus,
  openTimelineRenderer,
} from './timelineRendererApi.js';

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

describe('timeline renderer api helpers', () => {
  it('uses desktop timeline renderer lifecycle commands', async () => {
    const response = {
      renderer_window_kind: 'timeline',
      running: true,
      renderer_window_open: true,
      renderer_scene_ready: true,
      renderer_window_lifecycle: 'scene_ready_pending_native_runner',
      renderer_window_visible: false,
      renderer_window_ready: false,
      renderer_window_focus_supported: false,
      renderer_window_message: 'timeline renderer scene is ready; native window is not connected',
      track_count: 5,
      clip_count: 18,
      relationship_count: 3,
      affect_overlay_count: 4,
      queued_command_count: 0,
      last_error: null,
    };
    const invoke = installDesktopInvoke(response);

    await expect(openTimelineRenderer()).resolves.toEqual(response);
    await expect(focusTimelineRenderer()).resolves.toEqual(response);
    await expect(getTimelineRendererStatus()).resolves.toEqual(response);
    await expect(closeTimelineRenderer()).resolves.toEqual(response);

    expect(invoke).toHaveBeenNthCalledWith(1, 'timeline_renderer_open', undefined);
    expect(invoke).toHaveBeenNthCalledWith(2, 'timeline_renderer_focus', undefined);
    expect(invoke).toHaveBeenNthCalledWith(3, 'timeline_renderer_status', undefined);
    expect(invoke).toHaveBeenNthCalledWith(4, 'timeline_renderer_close', undefined);
  });
});
