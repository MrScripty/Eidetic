import type { CommandEnvelope } from './projectionTypes.js';
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
} from './timelineCommandTypes.js';
import { hasDesktopTransport, invokeDesktop } from './desktopTransport.js';
import { createCommandId, request } from './commandTransport.js';

export function setTimelineNodeRange(
  payload: SetTimelineNodeRangeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SetTimelineNodeRangeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_node_range', { command });
  }

  return request('/commands/timeline/node-range', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createTimelineNode(
  payload: CreateTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<CreateTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_create_node', { command });
  }

  return request('/commands/timeline/create-node', {
    method: 'POST',
    body: JSON.stringify(command),
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

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_apply_children', { command });
  }

  return request('/commands/timeline/apply-children', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createTimelineRelationship(
  payload: CreateTimelineRelationshipCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<CreateTimelineRelationshipCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_create_relationship', {
      command,
    });
  }

  return request('/commands/timeline/create-relationship', {
    method: 'POST',
    body: JSON.stringify(command),
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

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_delete_relationship', {
      command,
    });
  }

  return request('/commands/timeline/delete-relationship', {
    method: 'POST',
    body: JSON.stringify(command),
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

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_node_lock', { command });
  }

  return request('/commands/timeline/node-lock', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setTimelineNodeNotes(
  payload: SetTimelineNodeNotesCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SetTimelineNodeNotesCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_node_notes', { command });
  }

  return request('/commands/timeline/node-notes', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function splitTimelineNode(
  payload: SplitTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<SplitTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_split_node', { command });
  }

  return request('/commands/timeline/split-node', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function deleteTimelineNode(
  payload: DeleteTimelineNodeCommand,
  commandId = createCommandId(),
): Promise<TimelineCommandResponse> {
  const command: CommandEnvelope<DeleteTimelineNodeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<TimelineCommandResponse>('command_timeline_delete_node', { command });
  }

  return request('/commands/timeline/delete-node', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}
