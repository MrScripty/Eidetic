import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  getEmbeddedViewportStatus,
  mountEmbeddedViewport,
  setEmbeddedViewportFocus,
  unmountEmbeddedViewport,
  updateEmbeddedViewportBounds,
} from './embeddedViewportApi.js';
import type { EmbeddedViewportBounds } from './embeddedViewportTypes.js';

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

describe('embedded viewport api helpers', () => {
  const bounds: EmbeddedViewportBounds = {
    x: 10,
    y: 20,
    width: 640,
    height: 360,
    scale_factor: 1,
  };
  const surface = {
    attached: false,
    status: 'pending_attachment',
    message: 'native Bevy surface attachment is not implemented yet',
  };

  it('mounts a graph viewport through desktop IPC', async () => {
    const response = { viewport_id: 'graph-main', kind: 'graph', bounds, focused: false, surface };
    const invoke = installDesktopInvoke(response);

    await expect(
      mountEmbeddedViewport({
        viewport_id: 'graph-main',
        kind: 'graph',
        bounds,
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('viewport_mount', {
      request: {
        viewport_id: 'graph-main',
        kind: 'graph',
        bounds,
      },
    });
  });

  it('updates viewport bounds through desktop IPC', async () => {
    const response = { viewport_id: 'graph-main', kind: 'graph', bounds, focused: false, surface };
    const invoke = installDesktopInvoke(response);

    await expect(
      updateEmbeddedViewportBounds({
        viewport_id: 'graph-main',
        bounds,
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('viewport_update_bounds', {
      request: {
        viewport_id: 'graph-main',
        bounds,
      },
    });
  });

  it('sets viewport focus through desktop IPC', async () => {
    const response = { viewport_id: 'graph-main', kind: 'graph', bounds, focused: true, surface };
    const invoke = installDesktopInvoke(response);

    await expect(
      setEmbeddedViewportFocus({
        viewport_id: 'graph-main',
        focused: true,
      }),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenCalledWith('viewport_set_focus', {
      request: {
        viewport_id: 'graph-main',
        focused: true,
      },
    });
  });

  it('unmounts and reads viewport status through desktop IPC', async () => {
    const response = { viewports: [] };
    const invoke = installDesktopInvoke(response);

    await expect(unmountEmbeddedViewport('graph-main')).resolves.toEqual(response);
    await expect(getEmbeddedViewportStatus()).resolves.toEqual(response);

    expect(invoke).toHaveBeenNthCalledWith(1, 'viewport_unmount', {
      viewportId: 'graph-main',
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'viewport_status', undefined);
  });
});
