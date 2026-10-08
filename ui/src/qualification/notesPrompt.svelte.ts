import { untrack } from 'svelte';
import * as production from '../lib/api.js';
export * from '../lib/api.js';

interface Read {
  id: number;
  node_id: string;
  stage: string;
  actual_user?: string;
  actual_system?: string;
}
export const notesPromptQA = $state({
  holdNext: false,
  held: false,
  calls: [] as Read[],
});
let release: (() => void) | undefined;
export function armContextHold() {
  notesPromptQA.holdNext = true;
}
export function releaseContext() {
  release?.();
  release = undefined;
}
export function contextReceipt() {
  return JSON.stringify({
    seam: 'QA held real public context return; no fabricated prompt',
    ...notesPromptQA,
  });
}
export async function getAiContext(nodeId: string, storyTimeMs?: number) {
  // Instrumentation must not become a dependency of the production owner effect.
  const { id, hold } = untrack(() => {
    const id = notesPromptQA.calls.length + 1;
    const hold = notesPromptQA.holdNext;
    notesPromptQA.holdNext = false;
    notesPromptQA.calls = [...notesPromptQA.calls, { id, node_id: nodeId, stage: 'started' }];
    return { id, hold };
  });
  const actual = await production.getAiContext(nodeId, storyTimeMs);
  untrack(() => {
    notesPromptQA.calls = notesPromptQA.calls.map((call) =>
      call.id === id
        ? {
            ...call,
            actual_user: actual.user,
            actual_system: actual.system,
            stage: hold ? 'held' : 'returned',
          }
        : call,
    );
    notesPromptQA.held = hold || notesPromptQA.held;
  });
  if (hold) {
    await new Promise<void>((resolve) => {
      release = resolve;
    });
    untrack(() => {
      notesPromptQA.held = false;
      notesPromptQA.calls = notesPromptQA.calls.map((call) =>
        call.id === id ? { ...call, stage: 'returned' } : call,
      );
    });
  }
  return actual;
}
