import {
  hasDesktopEventTransport,
  listenDesktop,
  type DesktopUnlisten,
} from './desktopTransport.js';
import type { ServerMessage } from './wsTypes.js';

const SERVER_EVENT_TOPIC = 'eidetic://server-event';

export interface ServerEventClient {
  connect(): void;
  disconnect(): void;
  on<T extends ServerMessage['type']>(
    type: T,
    handler: (data: Extract<ServerMessage, { type: T }>) => void,
  ): () => void;
}

type MessageHandler = (data: ServerMessage) => void;

interface DesktopServerEventPayload {
  event?: unknown;
}

export class DesktopServerEventClient implements ServerEventClient {
  private handlers = new Map<string, Set<MessageHandler>>();
  private connected = false;
  private unlisten: DesktopUnlisten | null = null;
  private connectPromise: Promise<void> | null = null;

  connect(): void {
    if (this.connected || this.connectPromise) return;
    this.connected = true;
    this.connectPromise = listenDesktop<DesktopServerEventPayload>(
      SERVER_EVENT_TOPIC,
      (payload) => {
        if (!isServerMessage(payload.event)) return;
        this.dispatch(payload.event);
      },
    )
      .then((unlisten) => {
        if (!this.connected) {
          unlisten();
          return;
        }
        this.unlisten = unlisten;
      })
      .finally(() => {
        this.connectPromise = null;
      });
  }

  disconnect(): void {
    this.connected = false;
    this.unlisten?.();
    this.unlisten = null;
  }

  on<T extends ServerMessage['type']>(
    type: T,
    handler: (data: Extract<ServerMessage, { type: T }>) => void,
  ): () => void {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler as MessageHandler);

    return () => {
      this.handlers.get(type)?.delete(handler as MessageHandler);
    };
  }

  private dispatch(message: ServerMessage): void {
    const handlers = this.handlers.get(message.type);
    if (!handlers) return;
    for (const handler of handlers) {
      handler(message);
    }
  }
}

export function createServerEventClient(): ServerEventClient {
  if (hasDesktopEventTransport()) {
    return new DesktopServerEventClient();
  }
  throw new Error('Tauri event transport is required for backend events');
}

function isServerMessage(value: unknown): value is ServerMessage {
  return Boolean(
    value &&
    typeof value === 'object' &&
    'type' in value &&
    typeof (value as { type: unknown }).type === 'string',
  );
}
