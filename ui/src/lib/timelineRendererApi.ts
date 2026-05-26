import { invokeDesktop } from './desktopTransport.js';
import type { TimelineRendererStatus } from './timelineRendererTypes.js';

export function openTimelineRenderer(): Promise<TimelineRendererStatus> {
  return invokeDesktop<TimelineRendererStatus>('timeline_renderer_open');
}

export function focusTimelineRenderer(): Promise<TimelineRendererStatus> {
  return invokeDesktop<TimelineRendererStatus>('timeline_renderer_focus');
}

export function getTimelineRendererStatus(): Promise<TimelineRendererStatus> {
  return invokeDesktop<TimelineRendererStatus>('timeline_renderer_status');
}

export function closeTimelineRenderer(): Promise<TimelineRendererStatus> {
  return invokeDesktop<TimelineRendererStatus>('timeline_renderer_close');
}
