import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';

export const timelineRendererWindowState = $state<{
  status: TimelineRendererStatus | null;
}>({
  status: null,
});

export function setTimelineRendererWindowStatus(status: TimelineRendererStatus): void {
  timelineRendererWindowState.status = status;
}

export function clearTimelineRendererWindowStatus(): void {
  timelineRendererWindowState.status = null;
}
