import { editorState } from './editor.svelte.js';
import { timelineState } from './timeline.svelte.js';
import {
  applyDeleteTimelineNodeCommand,
  applySplitTimelineNodeCommand,
  applyTimelineNodeRangeCommand,
  getCachedTimelineRenderProjection,
} from './timelineRenderProjection.svelte.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import type { TimelineRenderClip } from '../timelineRenderTypes.js';

export const TIMELINE_KEYBOARD_STEP_MS = 1_000;

export type TimelineKeyboardCommandResult = 'applied' | 'unavailable';

function selectedTimelineClip(): TimelineRenderClip | null {
  const selectedNodeId = editorState.selectedNodeId;
  const projection = getCachedTimelineRenderProjection();
  if (!selectedNodeId || !projection) return null;
  return projection.payload.clips.find((clip) => clip.node_id === selectedNodeId) ?? null;
}

function projectionDurationMs(): number {
  return getCachedTimelineRenderProjection()?.payload.total_duration_ms ?? 0;
}

export async function deleteSelectedTimelineNodeFromKeyboard(): Promise<TimelineKeyboardCommandResult> {
  const clip = selectedTimelineClip();
  if (!clip) return 'unavailable';

  await applyDeleteTimelineNodeCommand({ node_id: clip.node_id });
  editorState.selectedNodeId = null;
  editorState.selectedLevel = null;
  void refreshSelectedNodeEditorProjection(null).catch(() => {});
  return 'applied';
}

export async function splitSelectedTimelineNodeAtPlayhead(): Promise<TimelineKeyboardCommandResult> {
  const clip = selectedTimelineClip();
  if (!clip) return 'unavailable';
  const atMs = Math.round(timelineState.playheadMs);
  if (atMs <= clip.start_ms || atMs >= clip.end_ms) return 'unavailable';

  await applySplitTimelineNodeCommand({
    node_id: clip.node_id,
    at_ms: atMs,
  });
  editorState.selectedNodeId = null;
  editorState.selectedLevel = null;
  void refreshSelectedNodeEditorProjection(null).catch(() => {});
  return 'applied';
}

export async function nudgeSelectedTimelineNode(
  deltaMs: number,
): Promise<TimelineKeyboardCommandResult> {
  const clip = selectedTimelineClip();
  const durationMs = projectionDurationMs();
  if (!clip || durationMs <= 0) return 'unavailable';

  const widthMs = clip.end_ms - clip.start_ms;
  const nextStartMs = Math.max(0, Math.min(durationMs - widthMs, clip.start_ms + deltaMs));
  const nextEndMs = nextStartMs + widthMs;
  if (nextStartMs === clip.start_ms && nextEndMs === clip.end_ms) return 'unavailable';

  await applyTimelineNodeRangeCommand({
    node_id: clip.node_id,
    start_ms: nextStartMs,
    end_ms: nextEndMs,
  });
  return 'applied';
}

export async function resizeSelectedTimelineNodeStart(
  deltaMs: number,
): Promise<TimelineKeyboardCommandResult> {
  const clip = selectedTimelineClip();
  if (!clip) return 'unavailable';

  const nextStartMs = Math.max(0, Math.min(clip.end_ms - 1, clip.start_ms + deltaMs));
  if (nextStartMs === clip.start_ms) return 'unavailable';

  await applyTimelineNodeRangeCommand({
    node_id: clip.node_id,
    start_ms: nextStartMs,
    end_ms: clip.end_ms,
  });
  return 'applied';
}

export async function resizeSelectedTimelineNodeEnd(
  deltaMs: number,
): Promise<TimelineKeyboardCommandResult> {
  const clip = selectedTimelineClip();
  const durationMs = projectionDurationMs();
  if (!clip || durationMs <= 0) return 'unavailable';

  const nextEndMs = Math.max(clip.start_ms + 1, Math.min(durationMs, clip.end_ms + deltaMs));
  if (nextEndMs === clip.end_ms) return 'unavailable';

  await applyTimelineNodeRangeCommand({
    node_id: clip.node_id,
    start_ms: clip.start_ms,
    end_ms: nextEndMs,
  });
  return 'applied';
}
