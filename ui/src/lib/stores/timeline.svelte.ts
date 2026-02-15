import type { Timeline, InferredScene, TimeRange } from '../types.js';
import { TIMELINE } from '../types.js';

export type TimelineTool = 'select' | 'blade';

/** Reactive timeline state. Populated from the server after project creation. */
export const timelineState = $state<{
	timeline: Timeline | null;
	scenes: InferredScene[];
	zoom: number;
	scrollX: number;
	/** Measured width of the timeline viewport container. */
	viewportWidth: number;
	/** Playhead position in milliseconds. */
	playheadMs: number;
	/** Active editing tool. */
	activeTool: TimelineTool;
	/** Whether clip snapping is enabled. */
	snapping: boolean;
}>({
	timeline: null,
	scenes: [],
	zoom: 1.0,
	scrollX: 0,
	viewportWidth: 0,
	playheadMs: 0,
	activeTool: 'select',
	snapping: true,
});

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
	// Playhead pixel position relative to viewport before zoom.
	const playheadViewportX = timelineState.playheadMs * pxPerMs * timelineState.zoom - timelineState.scrollX;
	timelineState.zoom = newZoom;
	// Adjust scroll so the playhead stays at the same viewport position.
	const newPlayheadAbsX = timelineState.playheadMs * pxPerMs * newZoom;
	const maxScroll = Math.max(0, totalWidth() - timelineState.viewportWidth);
	timelineState.scrollX = Math.max(0, Math.min(maxScroll, newPlayheadAbsX - playheadViewportX));
}

/** Set zoom so the entire timeline fits within the viewport. */
export function zoomToFit(): void {
	if (timelineState.viewportWidth <= 0) return;
	timelineState.zoom = timelineState.viewportWidth / (TIMELINE.DURATION_MS * TIMELINE.DEFAULT_PX_PER_MS);
	timelineState.scrollX = 0;
}

/** Zoom and scroll so a specific time range fills the viewport (with padding). */
export function zoomToRange(startMs: number, endMs: number): void {
	if (timelineState.viewportWidth <= 0) return;
	const durationMs = endMs - startMs;
	if (durationMs <= 0) return;
	const padding = durationMs * 0.1; // 10% padding on each side
	const paddedStart = Math.max(0, startMs - padding);
	const paddedDuration = durationMs + padding * 2;
	timelineState.zoom = timelineState.viewportWidth / (paddedDuration * TIMELINE.DEFAULT_PX_PER_MS);
	timelineState.scrollX = paddedStart * TIMELINE.DEFAULT_PX_PER_MS * timelineState.zoom;
}

/** Transient state for drag-to-connect relationship creation. */
export const connectionDrag = $state<{
	active: boolean;
	fromClipId: string | null;
	fromX: number;
	fromY: number;
	currentX: number;
	currentY: number;
}>({
	active: false,
	fromClipId: null,
	fromX: 0,
	fromY: 0,
	currentX: 0,
	currentY: 0,
});
