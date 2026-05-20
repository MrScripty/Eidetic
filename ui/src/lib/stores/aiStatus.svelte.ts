import { getAiStatus } from '$lib/api.js';
import type { AiStatus } from '$lib/aiTypes.js';

const POLL_INTERVAL_MS = 30_000;

export const aiStatusState = $state<{
  status: AiStatus | null;
}>({
  status: null,
});

let pollTimer: ReturnType<typeof setInterval> | null = null;
let pollOwners = 0;
let latestRefreshId = 0;

export async function refreshAiStatus(): Promise<void> {
  const refreshId = latestRefreshId + 1;
  latestRefreshId = refreshId;
  try {
    const status = await getAiStatus();
    if (refreshId === latestRefreshId) {
      aiStatusState.status = status;
    }
  } catch {
    if (refreshId !== latestRefreshId) return;
    if (aiStatusState.status) {
      aiStatusState.status = {
        ...aiStatusState.status,
        connected: false,
        error: 'Failed to reach server',
      };
    } else {
      aiStatusState.status = {
        backend: 'llama_cpp',
        connected: false,
        error: 'Failed to reach server',
      };
    }
  }
}

export function resetAiStatusPollingForTests(): void {
  latestRefreshId += 1;
  pollOwners = 0;
  if (pollTimer) {
    clearInterval(pollTimer);
    pollTimer = null;
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
