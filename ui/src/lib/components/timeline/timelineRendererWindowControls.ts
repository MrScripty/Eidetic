import type { TimelineRendererStatus } from '$lib/timelineRendererTypes.js';

export type TimelineRendererWindowControlAction = 'open' | 'refresh' | 'focus' | 'close';

export interface TimelineRendererWindowControlState {
  action: TimelineRendererWindowControlAction;
  label: string;
  disabled: boolean;
}

const CONTROL_LABELS: Record<TimelineRendererWindowControlAction, string> = {
  open: 'Open timeline renderer',
  refresh: 'Refresh timeline renderer status',
  focus: 'Focus timeline renderer',
  close: 'Close timeline renderer',
};

export function timelineRendererStatusLabel(status: TimelineRendererStatus | null): string {
  if (!status) return 'not checked';
  return (
    status.last_error ??
    (status.renderer_window_ready ? `${status.clip_count} clips` : status.renderer_window_message)
  );
}

export function timelineRendererWindowControlState(
  action: TimelineRendererWindowControlAction,
  status: TimelineRendererStatus | null,
  pending: boolean,
): TimelineRendererWindowControlState {
  const running = status?.running ?? false;
  const focusSupported = status?.renderer_window_focus_supported ?? false;
  const disabled =
    pending ||
    (action === 'focus' && (!running || !focusSupported)) ||
    (action === 'close' && !running);

  return {
    action,
    label: CONTROL_LABELS[action],
    disabled,
  };
}
