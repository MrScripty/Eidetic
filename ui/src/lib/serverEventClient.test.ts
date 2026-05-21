import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createServerEventClient,
  DesktopServerEventClient,
  type ServerEventClient,
} from './serverEventClient.js';

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('desktop server event client', () => {
  it('dispatches Tauri server events to typed handlers', async () => {
    let listener: ((event: { payload: unknown }) => void) | undefined;
    const unlisten = vi.fn();
    const listen = vi.fn((_event: string, handler: (event: { payload: unknown }) => void) => {
      listener = handler;
      return Promise.resolve(unlisten);
    });
    vi.stubGlobal('window', {
      __TAURI__: {
        event: { listen },
      },
    });
    const client = new DesktopServerEventClient();
    const handler = vi.fn();

    client.connect();
    client.on('timeline_changed', handler);
    await Promise.resolve();
    if (!listener) {
      throw new Error('desktop event listener was not registered');
    }
    listener({ payload: { event: { type: 'timeline_changed' } } });

    expect(listen).toHaveBeenCalledWith('eidetic://server-event', expect.any(Function));
    expect(handler).toHaveBeenCalledWith({ type: 'timeline_changed' });
  });

  it('unsubscribes from Tauri events on disconnect', async () => {
    const unlisten = vi.fn();
    let resolveListen: ((unlisten: () => void) => void) | undefined;
    vi.stubGlobal('window', {
      __TAURI__: {
        event: {
          listen: vi.fn(
            () =>
              new Promise<() => void>((resolve) => {
                resolveListen = resolve;
              }),
          ),
        },
      },
    });
    const client = new DesktopServerEventClient();

    client.connect();
    client.disconnect();
    resolveListen?.(unlisten);
    await flushPromises();

    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('uses the desktop event client when Tauri event transport exists', () => {
    vi.stubGlobal('window', {
      __TAURI__: {
        event: {
          listen: vi.fn(),
        },
      },
    });

    const client: ServerEventClient = createServerEventClient();

    expect(client).toBeInstanceOf(DesktopServerEventClient);
  });
});

function flushPromises(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}
