import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  drainGraphRendererCommands,
  getGraphRendererStatus,
  getGraphRendererVisualSnapshot,
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
      running: true,
      native_panel_ready: true,
      node_count: 2,
      edge_count: 1,
      native_visual_node_count: 2,
      native_visual_edge_count: 1,
      native_panel_width_px: 1280,
      native_panel_height_px: 720,
      influence_count: 1,
      last_error: null,
    };
    const invoke = installDesktopInvoke(response);

    await expect(getGraphRendererStatus()).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('graph_renderer_status', undefined);
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
