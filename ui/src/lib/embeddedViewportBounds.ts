import type { EmbeddedViewportBounds } from './embeddedViewportTypes.js';

export interface ClientRectLike {
  left: number;
  top: number;
  width: number;
  height: number;
}

export function embeddedViewportBoundsFromRect(
  rect: ClientRectLike,
  scaleFactor: number,
): EmbeddedViewportBounds | null {
  if (
    !Number.isFinite(rect.left) ||
    !Number.isFinite(rect.top) ||
    !Number.isFinite(rect.width) ||
    !Number.isFinite(rect.height) ||
    !Number.isFinite(scaleFactor) ||
    rect.width <= 0 ||
    rect.height <= 0 ||
    scaleFactor <= 0
  ) {
    return null;
  }

  return {
    x: Math.max(0, rect.left),
    y: Math.max(0, rect.top),
    width: rect.width,
    height: rect.height,
    scale_factor: scaleFactor,
  };
}

export function embeddedViewportBoundsChanged(
  previous: EmbeddedViewportBounds | null,
  next: EmbeddedViewportBounds,
): boolean {
  if (!previous) {
    return true;
  }

  return (
    Math.abs(previous.x - next.x) >= 0.5 ||
    Math.abs(previous.y - next.y) >= 0.5 ||
    Math.abs(previous.width - next.width) >= 0.5 ||
    Math.abs(previous.height - next.height) >= 0.5 ||
    Math.abs(previous.scale_factor - next.scale_factor) >= 0.001
  );
}
