import {
  applyTimelineChildren,
  createTimelineNode,
  createTimelineChildFromParent,
  createTimelineRelationship,
  deleteTimelineNode,
  deleteTimelineRelationship,
  setTimelineNodeLock,
  setTimelineNodeArcs,
  setTimelineNodeNotes,
  setTimelineNodeName,
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
  CreateTimelineChildFromParentCommand,
  CreateTimelineRelationshipCommand,
  DeleteTimelineNodeCommand,
  DeleteTimelineRelationshipCommand,
  SetTimelineNodeLockCommand,
  SetTimelineNodeArcsCommand,
  SetTimelineNodeNotesCommand,
  SetTimelineNodeNameCommand,
  SetTimelineNodeRangeCommand,
  SplitTimelineNodeCommand,
  TimelineCommandResponse,
} from '../timelineCommandTypes.js';
import type { TimelineRenderProjection } from '../timelineRenderTypes.js';
import type { CommandId, ProjectionEnvelope } from '../projectionTypes.js';
import { shouldReplaceProjection } from './projectionCacheGuards.js';

export const timelineRenderProjectionState = $state<{
  projection: ProjectionEnvelope<TimelineRenderProjection> | null;
  pending: boolean;
  error?: string;
}>({
  projection: null,
  pending: false,
  error: undefined,
});

// A clear starts a new cache lifetime even when the same project is reopened.
let cacheGeneration = 0;
let activeRequests = 0;
let latestRequestId = 0;

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

function replaceTimelineRenderProjectionIfFresh(
  projection: ProjectionEnvelope<TimelineRenderProjection>,
): void {
  if (shouldReplaceProjection(timelineRenderProjectionState.projection, projection)) {
    timelineRenderProjectionState.projection = projection;
  }
}

export function getCachedTimelineRenderProjection(): ProjectionEnvelope<TimelineRenderProjection> | null {
  return timelineRenderProjectionState.projection;
}

export function getCachedTimelineRenderModel(): TimelineRenderModel | null {
  const projection = timelineRenderProjectionState.projection;
  return projection ? timelineRenderModelFromProjection(projection.payload) : null;
}

async function runTimelineProjectionRequest<T>(
  request: () => Promise<T>,
  projectionOf: (result: T) => ProjectionEnvelope<TimelineRenderProjection>,
  failureMessage: string,
): Promise<T> {
  const generation = cacheGeneration;
  const requestId = ++latestRequestId;
  activeRequests += 1;
  timelineRenderProjectionState.pending = true;
  timelineRenderProjectionState.error = undefined;

  try {
    const result = await request();
    if (generation === cacheGeneration) {
      replaceTimelineRenderProjectionIfFresh(projectionOf(result));
    }
    return result;
  } catch (error) {
    if (generation === cacheGeneration && requestId === latestRequestId) {
      timelineRenderProjectionState.error = errorMessage(error, failureMessage);
    }
    throw error;
  } finally {
    if (generation === cacheGeneration) {
      activeRequests -= 1;
      timelineRenderProjectionState.pending = activeRequests > 0;
    }
  }
}

export async function refreshTimelineRenderProjection(): Promise<
  ProjectionEnvelope<TimelineRenderProjection>
> {
  return runTimelineProjectionRequest(
    () => getTimelineRenderProjection(),
    (result) => result,
    'Failed to load timeline render projection',
  );
}

export async function applyTimelineNodeNameCommand(
  payload: SetTimelineNodeNameCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => setTimelineNodeName(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline node name command',
  );
}
export async function applyTimelineNodeRangeCommand(
  payload: SetTimelineNodeRangeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => setTimelineNodeRange(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline node range command',
  );
}

export async function applyCreateTimelineNodeCommand(
  payload: CreateTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => createTimelineNode(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline create node command',
  );
}

export async function applyCreateTimelineChildFromParentCommand(
  payload: CreateTimelineChildFromParentCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => createTimelineChildFromParent(payload, commandId),
    (result) => result.projection,
    'Failed to create timeline child',
  );
}

export async function applyTimelineChildrenCommand(
  payload: ApplyTimelineChildrenCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => applyTimelineChildren(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline children command',
  );
}

export async function applyCreateTimelineRelationshipCommand(
  payload: CreateTimelineRelationshipCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => createTimelineRelationship(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline create relationship command',
  );
}

export async function applyDeleteTimelineRelationshipCommand(
  payload: DeleteTimelineRelationshipCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => deleteTimelineRelationship(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline delete relationship command',
  );
}

export async function applyTimelineNodeLockCommand(
  payload: SetTimelineNodeLockCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => setTimelineNodeLock(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline node lock command',
  );
}

export async function applyTimelineNodeArcsCommand(
  payload: SetTimelineNodeArcsCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => setTimelineNodeArcs(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline arc assignment command',
  );
}

export async function applyTimelineNodeNotesCommand(
  payload: SetTimelineNodeNotesCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => setTimelineNodeNotes(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline node notes command',
  );
}

export async function applySplitTimelineNodeCommand(
  payload: SplitTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => splitTimelineNode(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline split node command',
  );
}

export async function applyDeleteTimelineNodeCommand(
  payload: DeleteTimelineNodeCommand,
  commandId?: CommandId,
): Promise<TimelineCommandResponse> {
  return runTimelineProjectionRequest(
    () => deleteTimelineNode(payload, commandId),
    (result) => result.projection,
    'Failed to apply timeline delete node command',
  );
}

export function clearTimelineRenderProjection(): void {
  cacheGeneration += 1;
  activeRequests = 0;
  timelineRenderProjectionState.projection = null;
  timelineRenderProjectionState.pending = false;
  timelineRenderProjectionState.error = undefined;
}
