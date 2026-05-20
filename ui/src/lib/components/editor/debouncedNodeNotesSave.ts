import type { NodeId } from '$lib/timelineTypes.js';

export interface DebouncedNodeNotesSaveOptions {
  delayMs: number;
  save: (nodeId: NodeId, notes: string) => Promise<void>;
}

export interface DebouncedNodeNotesSave {
  schedule: (nodeId: NodeId, notes: string) => void;
  dispose: () => void;
}

export function createDebouncedNodeNotesSave({
  delayMs,
  save,
}: DebouncedNodeNotesSaveOptions): DebouncedNodeNotesSave {
  let timer: ReturnType<typeof setTimeout> | null = null;

  function dispose(): void {
    if (!timer) return;
    clearTimeout(timer);
    timer = null;
  }

  return {
    schedule(nodeId, notes) {
      dispose();
      timer = setTimeout(() => {
        timer = null;
        void save(nodeId, notes);
      }, delayMs);
    },
    dispose,
  };
}
