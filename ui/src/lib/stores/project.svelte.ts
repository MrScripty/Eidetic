import type { Project } from '../projectTypes.js';

/** Reactive project state. Backend-owned; frontend displays only. */
export const projectState = $state<{ current: Project | null }>({ current: null });
