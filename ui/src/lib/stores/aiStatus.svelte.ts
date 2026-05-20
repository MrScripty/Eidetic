import { getAiStatus } from '$lib/api.js';
import { editorState } from './editor.svelte.js';

const POLL_INTERVAL_MS = 30_000;

let pollTimer: ReturnType<typeof setInterval> | null = null;
let pollOwners = 0;

export async function refreshAiStatus(): Promise<void> {
  try {
    editorState.aiStatus = await getAiStatus();
  } catch {
    if (editorState.aiStatus) {
      editorState.aiStatus = {
        ...editorState.aiStatus,
        connected: false,
        error: 'Failed to reach server',
      };
    } else {
      editorState.aiStatus = {
        backend: 'llama_cpp',
        connected: false,
        error: 'Failed to reach server',
      };
    }
  }
}

export function startAiStatusPolling(): () => void {
  pollOwners += 1;
  if (pollOwners === 1) {
    void refreshAiStatus();
    pollTimer = setInterval(() => {
      void refreshAiStatus();
    }, POLL_INTERVAL_MS);
  }

  return () => {
    pollOwners = Math.max(0, pollOwners - 1);
    if (pollOwners === 0 && pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  };
}
