import * as Y from 'yjs';
import type { ServerMessage } from './types.js';
import { ydoc } from './yjs.js';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type MessageHandler = (data: any) => void;

/**
 * WebSocket client with automatic reconnection and event dispatch.
 *
 * Multiplexes two frame types on a single connection:
 * - **Binary frames**: Y.Doc CRDT updates (forwarded to/from yjs)
 * - **Text frames**: JSON-encoded ServerEvent messages
 */
export class WsClient {
	private ws: WebSocket | null = null;
	private handlers = new Map<string, Set<MessageHandler>>();
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private url: string;
	private yjsSyncActive = false;

	constructor(url = `ws://${window.location.host}/ws`) {
		this.url = url;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) return;

		this.ws = new WebSocket(this.url);
		// Receive binary frames as ArrayBuffer (for CRDT updates).
		this.ws.binaryType = 'arraybuffer';

		this.ws.onopen = () => {
			console.log('[ws] connected');
			this.yjsSyncActive = true;
			// Subscribe to all channels.
			this.send({ type: 'subscribe', data: { channels: ['beats', 'generation', 'timeline', 'scenes'] } });
		};

		this.ws.onmessage = (event: MessageEvent) => {
			if (event.data instanceof ArrayBuffer) {
				// Binary frame → CRDT update from server.
				this.handleCrdtUpdate(new Uint8Array(event.data));
				return;
			}

			// Text frame → JSON ServerEvent.
			try {
				const msg = JSON.parse(event.data as string) as ServerMessage;
				const handlers = this.handlers.get(msg.type);
				if (handlers) {
					for (const handler of handlers) {
						handler(msg);
					}
				}
			} catch {
				// Ignore malformed messages.
			}
		};

		this.ws.onclose = () => {
			console.log('[ws] disconnected, reconnecting in 2s...');
			this.yjsSyncActive = false;
			this.scheduleReconnect();
		};

		this.ws.onerror = () => {
			this.ws?.close();
		};

		// Wire outgoing Y.Doc updates → server.
		this.setupYjsOutgoing();
	}

	disconnect(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		this.yjsSyncActive = false;
		this.ws?.close();
		this.ws = null;
	}

	on<T extends ServerMessage['type']>(
		type: T,
		handler: (data: Extract<ServerMessage, { type: T }>) => void,
	): () => void {
		if (!this.handlers.has(type)) {
			this.handlers.set(type, new Set());
		}
		this.handlers.get(type)!.add(handler);

		// Return unsubscribe function.
		return () => {
			this.handlers.get(type)?.delete(handler);
		};
	}

	send(data: unknown): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(JSON.stringify(data));
		}
	}

	/** Send a binary CRDT update to the server. */
	sendBinary(data: Uint8Array): void {
		if (this.ws?.readyState === WebSocket.OPEN) {
			this.ws.send(data);
		}
	}

	private scheduleReconnect(): void {
		if (this.reconnectTimer) return;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
		}, 2000);
	}

	/** Apply an incoming binary CRDT update to the local Y.Doc. */
	private handleCrdtUpdate(data: Uint8Array): void {
		try {
			Y.applyUpdate(ydoc, data, 'server');
		} catch (e) {
			console.warn('[ws] failed to apply CRDT update:', e);
		}
	}

	/** Listen for local Y.Doc changes and send them to the server. */
	private setupYjsOutgoing(): void {
		ydoc.on('update', (update: Uint8Array, origin: unknown) => {
			// Don't echo back updates that came from the server.
			if (origin === 'server') return;
			if (!this.yjsSyncActive) return;
			this.sendBinary(update);
		});
	}
}
