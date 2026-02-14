export type NotificationType = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
	id: string;
	type: NotificationType;
	message: string;
	duration: number;
}

export const notifications = $state<{ items: Notification[] }>({ items: [] });

let counter = 0;

export function notify(type: NotificationType, message: string, duration = 3000) {
	const id = `n-${++counter}`;
	notifications.items = [...notifications.items, { id, type, message, duration }];
	if (duration > 0) {
		setTimeout(() => dismiss(id), duration);
	}
}

export function dismiss(id: string) {
	notifications.items = notifications.items.filter((n) => n.id !== id);
}
