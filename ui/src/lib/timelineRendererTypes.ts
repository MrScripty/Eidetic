export interface TimelineRendererStatus {
  renderer_window_kind: TimelineRendererWindowKind;
  running: boolean;
  renderer_window_open: boolean;
  renderer_scene_ready: boolean;
  renderer_window_lifecycle: TimelineRendererWindowLifecycle;
  renderer_window_visible: boolean;
  renderer_window_ready: boolean;
  renderer_window_focus_supported: boolean;
  renderer_window_message: string;
  track_count: number;
  clip_count: number;
  relationship_count: number;
  affect_overlay_count: number;
  queued_command_count: number;
  last_error: string | null;
}

export type TimelineRendererWindowKind = 'timeline';

export type TimelineRendererWindowLifecycle =
  | 'closed'
  | 'scene_ready_pending_native_runner'
  | 'visible';
