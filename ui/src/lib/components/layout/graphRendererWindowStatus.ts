import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';

export interface GraphRendererWindowStatusDisplay {
  label: string;
  active: boolean;
  message: string;
  primaryActionLabel: string;
  nativeWindowAvailable: boolean;
}

export function graphRendererWindowStatusDisplay(
  status: GraphRendererStatus | null,
): GraphRendererWindowStatusDisplay {
  if (!status) {
    return {
      label: 'Renderer closed',
      active: false,
      message: 'Graph renderer window is closed',
      primaryActionLabel: 'Prepare Bevy Renderer',
      nativeWindowAvailable: false,
    };
  }

  switch (status.renderer_window_lifecycle) {
    case 'visible':
      return {
        label: 'Renderer visible',
        active: true,
        message: status.renderer_window_message,
        primaryActionLabel: 'Focus Bevy Window',
        nativeWindowAvailable: true,
      };
    case 'scene_ready_pending_native_runner':
      if (!status.renderer_window_verified_support) {
        return {
          label: unavailableLabel(status.renderer_window_capability_reason),
          active: false,
          message: status.renderer_window_message,
          primaryActionLabel: status.renderer_window_open
            ? 'Renderer Prepared'
            : 'Prepare Bevy Renderer',
          nativeWindowAvailable: false,
        };
      }
      return {
        label: 'Renderer waiting',
        active: false,
        message: status.renderer_window_message,
        primaryActionLabel: 'Focus Bevy Window',
        nativeWindowAvailable: true,
      };
    case 'scene_starting':
      return {
        label: 'Renderer preparing',
        active: false,
        message: status.renderer_window_message,
        primaryActionLabel: 'Preparing',
        nativeWindowAvailable: status.renderer_window_verified_support,
      };
    case 'closed':
      return {
        label: 'Renderer closed',
        active: false,
        message: status.renderer_window_message,
        primaryActionLabel: 'Prepare Bevy Renderer',
        nativeWindowAvailable: false,
      };
  }
}

function unavailableLabel(
  reason: GraphRendererStatus['renderer_window_capability_reason'],
): string {
  switch (reason) {
    case 'pending_native_runner':
    case 'platform_unproven':
      return 'Renderer unavailable';
    case 'platform_unsupported':
      return 'Renderer unsupported';
    case 'runner_error':
      return 'Renderer error';
    case 'verified_support':
      return 'Renderer waiting';
  }
}
