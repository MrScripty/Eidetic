import type { TimeRange } from '../timelineTypes.js';
import { TIMELINE } from '../timelineTypes.js';

export type TimelineTool = 'select' | 'blade';

/** Frontend-owned transient timeline interaction state. */
export const timelineState = $state<{
  zoom: number;
  scrollX: number;
  /** Measured width of the timeline viewport container. */
  viewportWidth: number;
  /** Playhead position in milliseconds. */
  playheadMs: number;
  /** Active editing tool. */
  activeTool: TimelineTool;
  /** Whether node snapping is enabled. */
  snapping: boolean;
}>({
  zoom: 1.0,
  scrollX: 0,
  viewportWidth: 0,
  playheadMs: 0,
  activeTool: 'select',
  snapping: true,
});

// --- Coordinate conversions ---

/** Convert a time range to pixel coordinates at current zoom. */
export function timeToX(ms: number): number {
  return ms * TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom;
}

/** Convert pixel X back to time in ms. */
export function xToTime(px: number): number {
  return px / (TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom);
}

/** Width in pixels for a time range at current zoom. */
export function rangeWidth(range: TimeRange): number {
  return (range.end_ms - range.start_ms) * TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom;
}

/** Total timeline width in pixels at current zoom. */
export function totalWidth(): number {
  return TIMELINE.DURATION_MS * TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom;
}

/** Zoom to a new level, keeping the playhead at the same viewport position. */
export function zoomTo(newZoom: number): void {
  newZoom = Math.max(0.005, Math.min(10, newZoom));
  const pxPerMs = TIMELINE.DEFAULT_PX_PER_MS;
  const playheadViewportX =
    timelineState.playheadMs * pxPerMs * timelineState.zoom - timelineState.scrollX;
  timelineState.zoom = newZoom;
  const newPlayheadAbsX = timelineState.playheadMs * pxPerMs * newZoom;
  const maxScroll = Math.max(0, totalWidth() - timelineState.viewportWidth);
  timelineState.scrollX = Math.max(0, Math.min(maxScroll, newPlayheadAbsX - playheadViewportX));
}

/** Set zoom so the entire timeline fits within the viewport. */
export function zoomToFit(): void {
  if (timelineState.viewportWidth <= 0) return;
  timelineState.zoom =
    timelineState.viewportWidth / (TIMELINE.DURATION_MS * TIMELINE.DEFAULT_PX_PER_MS);
  timelineState.scrollX = 0;
}

/** Zoom and scroll so a specific time range fills the viewport (with padding). */
export function zoomToRange(startMs: number, endMs: number): void {
  if (timelineState.viewportWidth <= 0) return;
  const durationMs = endMs - startMs;
  if (durationMs <= 0) return;
  const padding = durationMs * 0.1;
  const paddedStart = Math.max(0, startMs - padding);
  const paddedDuration = durationMs + padding * 2;
  timelineState.zoom = timelineState.viewportWidth / (paddedDuration * TIMELINE.DEFAULT_PX_PER_MS);
  timelineState.scrollX = paddedStart * TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom;
}

/** Scroll so a node is centered in the viewport, if it's not already fully visible. */
export function scrollToNode(startMs: number, endMs: number): void {
  const nodeLeft = timeToX(startMs);
  const nodeRight = timeToX(endMs);
  const vw = timelineState.viewportWidth;
  if (vw <= 0) return;

  if (nodeLeft >= timelineState.scrollX && nodeRight <= timelineState.scrollX + vw) {
    return;
  }

  const nodeCenter = (nodeLeft + nodeRight) / 2;
  const maxScroll = Math.max(0, totalWidth() - vw);
  timelineState.scrollX = Math.max(0, Math.min(maxScroll, nodeCenter - vw / 2));
}

/** Transient state for drag-to-connect relationship creation. */
export const connectionDrag = $state<{
  active: boolean;
  fromNodeId: string | null;
  fromX: number;
  fromY: number;
  currentX: number;
  currentY: number;
}>({
  active: false,
  fromNodeId: null,
  fromX: 0,
  fromY: 0,
  currentX: 0,
  currentY: 0,
});
