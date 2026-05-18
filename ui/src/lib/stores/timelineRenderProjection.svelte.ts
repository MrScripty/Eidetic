import {
  applyTimelineChildren,
  createTimelineNode,
  createTimelineRelationship,
  deleteTimelineNode,
  deleteTimelineRelationship,
  setTimelineNodeLock,
  setTimelineNodeNotes,
  setTimelineNodeRange,
  splitTimelineNode,
} from '$lib/commandApi.js';
import { getTimelineRenderProjection } from '$lib/projectionApi.js';
import {
  timelineRenderModelFromProjection,
  type TimelineRenderModel,
} from '$lib/timelineRenderModel.js';
import type {
  ApplyTimelineChildrenCommand,
  CreateTimelineNodeCommand,
  CreateTimelineRelationshipCommand,
  DeleteTimelineNodeCommand,
  DeleteTimelineRelationshipCommand,
  SetTimelineNodeLockCommand,
  SetTimelineNodeNotesCommand,
  SetTimelineNodeRangeCommand,
  SplitTimelineNodeCommand,
  TimelineCommandResponse,
} from '../timelineCommandTypes.js';
import type { TimelineRenderProjection } from '../timelineRenderTypes.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';

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

export function getCachedTimelineRenderModel(): TimelineRenderModel | null {
  const projection = timelineRenderProjectionState.projection;
  return projection ? timelineRenderModelFromProjection(projection.payload) : null;
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

export async function applyCreateTimelineNodeCommand(
  payload: CreateTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await createTimelineNode(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline create node command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyTimelineChildrenCommand(
  payload: ApplyTimelineChildrenCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await applyTimelineChildren(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline children command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyCreateTimelineRelationshipCommand(
  payload: CreateTimelineRelationshipCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await createTimelineRelationship(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline create relationship command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyDeleteTimelineRelationshipCommand(
  payload: DeleteTimelineRelationshipCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await deleteTimelineRelationship(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline delete relationship command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyTimelineNodeLockCommand(
  payload: SetTimelineNodeLockCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await setTimelineNodeLock(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline node lock command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyTimelineNodeNotesCommand(
  payload: SetTimelineNodeNotesCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await setTimelineNodeNotes(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline node notes command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applySplitTimelineNodeCommand(
  payload: SplitTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await splitTimelineNode(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline split node command',
    );
    throw error;
  } finally {
    timelineRenderProjectionState.pending = false;
  }
}

export async function applyDeleteTimelineNodeCommand(
  payload: DeleteTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const response = await deleteTimelineNode(payload, commandId);
    timelineRenderProjectionState.projection = response.projection;
    return response;
  } catch (error) {
    timelineRenderProjectionState.error = errorMessage(
      error,
      'Failed to apply timeline delete node command',
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
