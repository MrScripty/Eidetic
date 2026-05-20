import type { SetStoryArcMetadataCommand } from '$lib/storyArcTypes.js';

export interface DebouncedStoryArcMetadataSaveOptions {
  delayMs: number;
  save: (command: SetStoryArcMetadataCommand) => Promise<unknown>;
}

export interface DebouncedStoryArcMetadataSave {
  schedule: (arcId: string, field: 'name' | 'description', value: string) => void;
  dispose: () => void;
}

export function createDebouncedStoryArcMetadataSave({
  delayMs,
  save,
}: DebouncedStoryArcMetadataSaveOptions): DebouncedStoryArcMetadataSave {
  let timer: ReturnType<typeof setTimeout> | null = null;

  function dispose(): void {
    if (!timer) return;
    clearTimeout(timer);
    timer = null;
  }

  return {
    schedule(arcId, field, value) {
      dispose();
      timer = setTimeout(() => {
        timer = null;
        void save({ arc_id: arcId, [field]: value });
      }, delayMs);
    },
    dispose,
  };
}
