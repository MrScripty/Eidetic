import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { WsClient } from './ws.js';
import { ydoc } from './yjs.js';

class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  static instances: MockWebSocket[] = [];

  binaryType = 'blob';
  readyState = MockWebSocket.CONNECTING;
  onopen: (() => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: (() => void) | null = null;
  readonly send = vi.fn();

  constructor(readonly url: string) {
    MockWebSocket.instances.push(this);
  }

  open(): void {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.();
  }

  receive(data: unknown): void {
    this.onmessage?.({ data } as MessageEvent);
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.();
  }
}

const OriginalWebSocket = globalThis.WebSocket;

beforeEach(() => {
  vi.useFakeTimers();
  MockWebSocket.instances = [];
  vi.stubGlobal('WebSocket', MockWebSocket);
});

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  globalThis.WebSocket = OriginalWebSocket;
});

describe('websocket client lifecycle', () => {
  it('disconnect clears pending reconnect timers and prevents reconnect', async () => {
    const client = new WsClient('ws://localhost/ws');

    client.connect();
    MockWebSocket.instances[0]!.close();

    client.disconnect();
    await vi.advanceTimersByTimeAsync(2_000);

    expect(MockWebSocket.instances).toHaveLength(1);
  });

  it('manual disconnect prevents close-triggered reconnects', async () => {
    const client = new WsClient('ws://localhost/ws');

    client.connect();
    client.disconnect();
    await vi.advanceTimersByTimeAsync(2_000);

    expect(MockWebSocket.instances).toHaveLength(1);
  });

  it('unsubscribes message handlers', () => {
    const client = new WsClient('ws://localhost/ws');
    const handler = vi.fn();

    client.connect();
    const unsubscribe = client.on('timeline_changed', handler);
    unsubscribe();
    MockWebSocket.instances[0]!.receive(JSON.stringify({ type: 'timeline_changed' }));

    expect(handler).not.toHaveBeenCalled();
  });

  it('detaches outgoing yjs updates on disconnect', () => {
    const client = new WsClient('ws://localhost/ws');

    client.connect();
    MockWebSocket.instances[0]!.open();
    client.disconnect();
    ydoc.transact(() => {
      ydoc.getMap('ws-test').set('value', 'after-disconnect');
    });

    const socket = MockWebSocket.instances[0]!;
    expect(socket.send).toHaveBeenCalledTimes(1);
    expect(socket.send).toHaveBeenCalledWith(
      JSON.stringify({
        type: 'subscribe',
        data: { channels: ['beats', 'generation', 'timeline', 'scenes'] },
      }),
    );
  });
});
