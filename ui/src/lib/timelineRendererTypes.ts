export interface TimelineRendererStatus {
  renderer_window_kind: TimelineRendererWindowKind;
  running: boolean;
  renderer_scene_ready: boolean;
  track_count: number;
  clip_count: number;
  relationship_count: number;
  affect_overlay_count: number;
  queued_command_count: number;
  last_error: string | null;
}

export type TimelineRendererWindowKind = 'timeline';
