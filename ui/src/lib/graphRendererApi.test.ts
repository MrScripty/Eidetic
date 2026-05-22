import { afterEach, describe, expect, it, vi } from 'vitest';

import { drainGraphRendererCommands, getGraphRendererStatus } from './graphRendererApi.js';

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
      node_count: 2,
      edge_count: 1,
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
});
