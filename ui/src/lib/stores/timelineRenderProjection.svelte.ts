import { setTimelineNodeRange } from '$lib/commandApi.js';
import { getTimelineRenderProjection } from '$lib/projectionApi.js';
import type {
  CommandId,
  ProjectionEnvelope,
  SetTimelineNodeRangeCommand,
  TimelineCommandResponse,
  TimelineRenderProjection,
} from '../types.js';

export const timelineRenderProjectionState = $state<{
  projection: ProjectionEnvelope<TimelineRenderProjection> | null;
  pending: boolean;
  error?: string;
}>({
  projection: null,
  pending: false,
  error: undefined,
});

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function getCachedTimelineRenderProjection(): ProjectionEnvelope<TimelineRenderProjection> | null {
  return timelineRenderProjectionState.projection;
}

export async function refreshTimelineRenderProjection(): Promise<
  ProjectionEnvelope<TimelineRenderProjection>
> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const projection = await getTimelineRenderProjection();
    timelineRenderProjectionState.projection = projection;
    return projection;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to load timeline render projection',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyTimelineNodeRangeCommand(
  payload: SetTimelineNodeRangeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await setTimelineNodeRange(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline node range command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export function clearTimelineRenderProjection(): void {
  timelineRenderProjectionState.projection = null;
  timelineRenderProjectionState.pending = false;
  timelineRenderProjectionState.error = undefined;
}
