import type { BibleRenderGraphProjectionRequest } from './bibleGraphTypes.js';

export type EmbeddedViewportKind = 'graph' | 'timeline';

export interface EmbeddedViewportBounds {
  x: number;
  y: number;
  width: number;
  height: number;
  scale_factor: number;
}

export interface EmbeddedViewportState {
  viewport_id: string;
  kind: EmbeddedViewportKind;
  bounds: EmbeddedViewportBounds;
  focused: boolean;
  surface: EmbeddedViewportSurfaceState;
}

export interface EmbeddedViewportSurfaceState {
  attached: boolean;
  status: EmbeddedViewportSurfaceStatus;
  strategy: EmbeddedViewportSurfaceStrategy;
  message: string;
  renderer_window: EmbeddedViewportRendererWindowState;
}

export interface EmbeddedViewportHostStatus {
  viewports: EmbeddedViewportState[];
}

export type EmbeddedViewportSurfaceStatus =
  | 'pending_attachment'
  | 'attachment_unsupported'
  | 'attached';

export type EmbeddedViewportSurfaceStrategy =
  | 'unsupported'
  | 'x11_child_window'
  | 'wayland_external_surface'
  | 'win32_child_window'
  | 'app_kit_subview';

export interface EmbeddedViewportRendererWindowState {
  status: EmbeddedViewportRendererWindowStatus;
  message: string;
  parent_window_id: string | null;
}

export type EmbeddedViewportRendererWindowStatus =
  | 'not_started'
  | 'pending_creation'
  | 'creation_unsupported'
  | 'attached';

export interface MountEmbeddedViewportRequest {
  viewport_id: string;
  kind: EmbeddedViewportKind;
  bounds: EmbeddedViewportBounds;
  graph_projection_request?: BibleRenderGraphProjectionRequest;
}

export interface UpdateEmbeddedViewportBoundsRequest {
  viewport_id: string;
  bounds: EmbeddedViewportBounds;
}

export interface SetEmbeddedViewportFocusRequest {
  viewport_id: string;
  focused: boolean;
}
