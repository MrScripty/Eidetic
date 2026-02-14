export interface Shortcut {
	key: string;
	ctrl?: boolean;
	shift?: boolean;
	alt?: boolean;
	description: string;
	action: () => void;
	/** Skip this shortcut when focus is on an input/textarea/contenteditable. */
	skipInInput?: boolean;
}

const registry: Shortcut[] = [];

/** Register a shortcut. Returns an unsubscribe function. */
export function registerShortcut(shortcut: Shortcut): () => void {
	registry.push(shortcut);
	return () => {
		const idx = registry.indexOf(shortcut);
		if (idx >= 0) registry.splice(idx, 1);
	};
}

/** Handle a keydown event. Returns true if a shortcut matched. */
export function handleKeydown(e: KeyboardEvent): boolean {
	const target = e.target as HTMLElement;
	const isInput =
		target.tagName === 'INPUT' ||
		target.tagName === 'TEXTAREA' ||
		target.isContentEditable;

	for (const s of registry) {
		if (s.skipInInput && isInput) continue;
		if (s.key.toLowerCase() !== e.key.toLowerCase()) continue;
		if (!!s.ctrl !== (e.ctrlKey || e.metaKey)) continue;
		if (!!s.shift !== e.shiftKey) continue;
		if (!!s.alt !== e.altKey) continue;

		e.preventDefault();
		s.action();
		return true;
	}
	return false;
}

/** List all registered shortcuts (for help display). */
export function listShortcuts(): Shortcut[] {
	return [...registry];
}
