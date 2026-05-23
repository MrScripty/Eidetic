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
}

export interface EmbeddedViewportHostStatus {
  viewports: EmbeddedViewportState[];
}

export interface MountEmbeddedViewportRequest {
  viewport_id: string;
  kind: EmbeddedViewportKind;
  bounds: EmbeddedViewportBounds;
}

export interface UpdateEmbeddedViewportBoundsRequest {
  viewport_id: string;
  bounds: EmbeddedViewportBounds;
}

export interface SetEmbeddedViewportFocusRequest {
  viewport_id: string;
  focused: boolean;
}
