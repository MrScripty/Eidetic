type MessageHandler = (data: Record<string, unknown>) => void;

/**
 * WebSocket client with automatic reconnection and event dispatch.
 *
 * Subscribes to server-sent events and dispatches them to registered handlers.
 * In Sprint 1 this is a scaffold; Sprint 2 will add real event handling.
 */
export class WsClient {
	private ws: WebSocket | null = null;
	private handlers = new Map<string, Set<MessageHandler>>();
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private url: string;

	constructor(url = `ws://${window.location.host}/ws`) {
		this.url = url;
	}

	connect(): void {
		if (this.ws?.readyState === WebSocket.OPEN) return;

		this.ws = new WebSocket(this.url);

		this.ws.onopen = () => {
			console.log('[ws] connected');
			// Subscribe to all channels.
			this.send({ type: 'subscribe', data: { channels: ['beats', 'generation', 'timeline', 'scenes'] } });
		};

		this.ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data as string) as Record<string, unknown>;
				const type = msg.type as string;
				const handlers = this.handlers.get(type);
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
			this.scheduleReconnect();
		};

		this.ws.onerror = () => {
			this.ws?.close();
		};
	}

	disconnect(): void {
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
		this.ws?.close();
		this.ws = null;
	}

	on(type: string, handler: MessageHandler): () => void {
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

	private scheduleReconnect(): void {
		if (this.reconnectTimer) return;
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null;
			this.connect();
		}, 2000);
	}
}
