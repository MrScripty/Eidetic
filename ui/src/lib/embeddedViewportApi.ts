import { invokeDesktop } from './desktopTransport.js';
import type {
  EmbeddedViewportHostStatus,
  EmbeddedViewportState,
  MountEmbeddedViewportRequest,
  SetEmbeddedViewportFocusRequest,
  UpdateEmbeddedViewportBoundsRequest,
} from './embeddedViewportTypes.js';

export function mountEmbeddedViewport(
  request: MountEmbeddedViewportRequest,
): Promise<EmbeddedViewportState> {
  return invokeDesktop<EmbeddedViewportState>('viewport_mount', { request });
}

export function updateEmbeddedViewportBounds(
  request: UpdateEmbeddedViewportBoundsRequest,
): Promise<EmbeddedViewportState> {
  return invokeDesktop<EmbeddedViewportState>('viewport_update_bounds', { request });
}

export function setEmbeddedViewportFocus(
  request: SetEmbeddedViewportFocusRequest,
): Promise<EmbeddedViewportState> {
  return invokeDesktop<EmbeddedViewportState>('viewport_set_focus', { request });
}

export function unmountEmbeddedViewport(viewportId: string): Promise<EmbeddedViewportHostStatus> {
  return invokeDesktop<EmbeddedViewportHostStatus>('viewport_unmount', { viewportId });
}

export function getEmbeddedViewportStatus(): Promise<EmbeddedViewportHostStatus> {
  return invokeDesktop<EmbeddedViewportHostStatus>('viewport_status');
}
