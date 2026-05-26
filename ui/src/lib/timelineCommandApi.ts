import type { CommandEnvelope } from './projectionTypes.js';
import type {
  ApplyTimelineChildrenCommand,
  CreateTimelineChildFromParentCommand,
  CreateTimelineNodeCommand,
  CreateTimelineRelationshipCommand,
  DeleteTimelineNodeCommand,
  DeleteTimelineRelationshipCommand,
  SetTimelineNodeLockCommand,
  SetTimelineNodeNotesCommand,
  SetTimelineNodeRangeCommand,
  SplitTimelineNodeCommand,
  TimelineCommandResponse,
  TimelinePlayheadCommandResponse,
} from './timelineCommandTypes.js';
import { invokeDesktop } from './desktopTransport.js';
import { createCommandId } from './commandTransport.js';

export function setTimelinePlayhead(positionMs: number): Promise<TimelinePlayheadCommandResponse> {
  return invokeDesktop<TimelinePlayheadCommandResponse>('command_timeline_playhead', {
    positionMs: Math.max(0, Math.trunc(positionMs)),
  });
}

export function setTimelineNodeRange(
  payload: SetTimelineNodeRangeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SetTimelineNodeRangeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_node_range', { command });
}

export function createTimelineNode(
  payload: CreateTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<CreateTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_create_node', { command });
}

export function createTimelineChildFromParent(
  payload: CreateTimelineChildFromParentCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<CreateTimelineChildFromParentCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_create_child_from_parent', {
    command,
  });
}

export function applyTimelineChildren(
  payload: ApplyTimelineChildrenCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<ApplyTimelineChildrenCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_apply_children', { command });
}

export function createTimelineRelationship(
  payload: CreateTimelineRelationshipCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<CreateTimelineRelationshipCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_create_relationship', {
    command,
  });
}

export function deleteTimelineRelationship(
  payload: DeleteTimelineRelationshipCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<DeleteTimelineRelationshipCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_delete_relationship', {
    command,
  });
}

export function setTimelineNodeLock(
  payload: SetTimelineNodeLockCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SetTimelineNodeLockCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_node_lock', { command });
}

export function setTimelineNodeNotes(
  payload: SetTimelineNodeNotesCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SetTimelineNodeNotesCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_node_notes', { command });
}

export function splitTimelineNode(
  payload: SplitTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SplitTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_split_node', { command });
}

export function deleteTimelineNode(
  payload: DeleteTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<DeleteTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<TimelineCommandResponse>('command_timeline_delete_node', { command });
}
