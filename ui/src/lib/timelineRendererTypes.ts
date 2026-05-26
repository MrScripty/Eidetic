export interface TimelineRendererStatus {
  renderer_window_kind: TimelineRendererWindowKind;
  running: boolean;
  renderer_window_open: boolean;
  renderer_scene_ready: boolean;
  renderer_window_lifecycle: TimelineRendererWindowLifecycle;
  renderer_runner_lifecycle: TimelineRendererRunnerLifecycle;
  renderer_runner_threading_model: TimelineRendererRunnerThreadingModel;
  renderer_window_platform: TimelineRendererWindowPlatform;
  renderer_window_capability: TimelineRendererWindowCapability;
  renderer_window_capability_reason: TimelineRendererWindowCapabilityReason;
  renderer_window_visible: boolean;
  renderer_window_ready: boolean;
  renderer_window_verified_support: boolean;
  renderer_window_visible_supported: boolean;
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

export type TimelineRendererRunnerLifecycle = 'closed' | 'open_requested' | 'visible';

export type TimelineRendererRunnerThreadingModel = 'worker_thread' | 'main_thread' | 'unsupported';

export type TimelineRendererWindowPlatform = 'linux' | 'macos' | 'windows' | 'unsupported';

export type TimelineRendererWindowCapability =
  | 'pending_native_runner'
  | 'platform_unproven'
  | 'platform_unsupported'
  | 'runner_error'
  | 'verified_support';

export type TimelineRendererWindowCapabilityReason =
  | 'pending_native_runner'
  | 'platform_unproven'
  | 'platform_unsupported'
  | 'runner_error'
  | 'verified_support';
