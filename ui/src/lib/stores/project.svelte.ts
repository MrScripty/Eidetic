import type { Project } from '../types.js';

/** Reactive project state. Backend-owned; frontend displays only. */
export const projectState = $state<{ current: Project | null }>({ current: null });
