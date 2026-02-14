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

/** Set zoom so the entire timeline fits within the viewport. */
export function zoomToFit(): void {
	if (timelineState.viewportWidth <= 0) return;
	timelineState.zoom = timelineState.viewportWidth / (TIMELINE.DURATION_MS * TIMELINE.DEFAULT_PX_PER_MS);
	timelineState.scrollX = 0;
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
