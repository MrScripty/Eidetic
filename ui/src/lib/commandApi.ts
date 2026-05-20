import type {
  BibleGraphNodeCommandResponse,
  BibleGraphRootsCommandResponse,
  CreateBibleGraphNodeCommand,
  EnsureCanonicalBibleRootsCommand,
  SetBibleGraphEdgeCommand,
  SetBibleGraphFieldCommand,
  SetBibleGraphSnapshotFieldCommand,
} from './bibleGraphTypes.js';
import type {
  CommandEnvelope,
  ObjectFieldCommandResponse,
  SetObjectFieldCommand,
} from './projectionTypes.js';
import type {
  AcceptPropagationProposalCommand,
  CreatePropagationProposalCommand,
  PropagationProposalCommandResponse,
  RejectPropagationProposalCommand,
  UpdatePropagationProposalCommand,
} from './propagationProposalTypes.js';
import type {
  ScriptDocumentCommandResponse,
  SetScriptBlockCommand,
  SetScriptLockCommand,
} from './scriptTypes.js';
import type {
  AcceptBibleReferenceProposalCommand,
  BibleReferenceProposalCommandResponse,
  CreateBibleReferenceProposalCommand,
  RejectBibleReferenceProposalCommand,
} from './semanticProposalTypes.js';
import type {
  CreateStoryArcCommand,
  DeleteStoryArcCommand,
  SetStoryArcMetadataCommand,
  StoryArcCommandResponse,
} from './storyArcTypes.js';
import type {
  ApplyTimelineChildrenCommand,
  CreateTimelineRelationshipCommand,
  CreateTimelineNodeCommand,
  DeleteTimelineNodeCommand,
  DeleteTimelineRelationshipCommand,
  SetTimelineNodeLockCommand,
  SetTimelineNodeNotesCommand,
  SetTimelineNodeRangeCommand,
  SplitTimelineNodeCommand,
  TimelineCommandResponse,
} from './timelineCommandTypes.js';
import { hasDesktopTransport, invokeDesktop } from './desktopTransport.js';

const BASE = '/api';

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'Content-Type': 'application/json' },
    ...options,
  });
  const body = await res.json().catch(() => null);
  if (!res.ok) {
    throw new Error((body as Record<string, string> | null)?.error || `HTTP ${res.status}`);
  }
  if (body && typeof body === 'object' && 'error' in body && typeof body.error === 'string') {
    throw new Error(body.error);
  }
  return body as T;
}

export function createCommandId(): string {
  return crypto.randomUUID();
}

export function setObjectField(
  payload: SetObjectFieldCommand,
  commandId = createCommandId(),
): Promise<ObjectFieldCommandResponse> {
  const command: CommandEnvelope<SetObjectFieldCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<ObjectFieldCommandResponse>('command_object_field', { command });
  }

  return request('/commands/object-field', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createBibleGraphNode(
  payload: CreateBibleGraphNodeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<CreateBibleGraphNodeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_node', { command });
  }

  return request('/commands/bible-graph/node', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setBibleGraphField(
  payload: SetBibleGraphFieldCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<SetBibleGraphFieldCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_field', { command });
  }

  return request('/commands/bible-graph/field', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setBibleGraphEdge(
  payload: SetBibleGraphEdgeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<SetBibleGraphEdgeCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_edge', { command });
  }

  return request('/commands/bible-graph/edge', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setBibleGraphSnapshotField(
  payload: SetBibleGraphSnapshotFieldCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<SetBibleGraphSnapshotFieldCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_snapshot_field', {
      command,
    });
  }

  return request('/commands/bible-graph/snapshot-field', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function ensureCanonicalBibleRoots(
  commandId = createCommandId(),
): Promise<BibleGraphRootsCommandResponse> {
  const command: CommandEnvelope<EnsureCanonicalBibleRootsCommand> = {
    id: commandId,
    payload: {},
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleGraphRootsCommandResponse>('command_bible_graph_roots', {
      command,
    });
  }

  return request('/commands/bible-graph/canonical-roots', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createBibleReferenceProposal(
  payload: CreateBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<CreateBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleReferenceProposalCommandResponse>(
      'command_bible_reference_proposal_create',
      { command },
    );
  }

  return request('/commands/semantic/bible-reference-proposal', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function rejectBibleReferenceProposal(
  payload: RejectBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<RejectBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleReferenceProposalCommandResponse>(
      'command_bible_reference_proposal_reject',
      { command },
    );
  }

  return request('/commands/semantic/bible-reference-proposal/reject', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function acceptBibleReferenceProposal(
  payload: AcceptBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<AcceptBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<BibleReferenceProposalCommandResponse>(
      'command_bible_reference_proposal_accept',
      { command },
    );
  }

  return request('/commands/semantic/bible-reference-proposal/accept', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createPropagationProposal(
  payload: CreatePropagationProposalCommand,
  commandId = createCommandId(),
): Promise<PropagationProposalCommandResponse> {
  const command: CommandEnvelope<CreatePropagationProposalCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/semantic/propagation-proposal', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function rejectPropagationProposal(
  payload: RejectPropagationProposalCommand,
  commandId = createCommandId(),
): Promise<PropagationProposalCommandResponse> {
  const command: CommandEnvelope<RejectPropagationProposalCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/semantic/propagation-proposal/reject', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function updatePropagationProposal(
  payload: UpdatePropagationProposalCommand,
  commandId = createCommandId(),
): Promise<PropagationProposalCommandResponse> {
  const command: CommandEnvelope<UpdatePropagationProposalCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/semantic/propagation-proposal/update', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function acceptPropagationProposal(
  payload: AcceptPropagationProposalCommand,
  commandId = createCommandId(),
): Promise<PropagationProposalCommandResponse> {
  const command: CommandEnvelope<AcceptPropagationProposalCommand> = {
    id: commandId,
    payload,
  };

  return request('/commands/semantic/propagation-proposal/accept', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setScriptBlock(
  payload: SetScriptBlockCommand,
  commandId = createCommandId(),
): Promise<ScriptDocumentCommandResponse> {
  const command: CommandEnvelope<SetScriptBlockCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<ScriptDocumentCommandResponse>('command_script_block', { command });
  }

  return request('/commands/script/block', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setScriptLock(
  payload: SetScriptLockCommand,
  commandId = createCommandId(),
): Promise<ScriptDocumentCommandResponse> {
  const command: CommandEnvelope<SetScriptLockCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<ScriptDocumentCommandResponse>('command_script_lock', { command });
  }

  return request('/commands/script/lock', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function createStoryArc(
  payload: CreateStoryArcCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<CreateStoryArcCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<StoryArcCommandResponse>('command_story_create', { command });
  }

  return request('/commands/story/create-arc', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function setStoryArcMetadata(
  payload: SetStoryArcMetadataCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<SetStoryArcMetadataCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<StoryArcCommandResponse>('command_story_update', { command });
  }

  return request('/commands/story/update-arc', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}

export function deleteStoryArc(
  payload: DeleteStoryArcCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<DeleteStoryArcCommand> = {
    id: commandId,
    payload,
  };

  if (hasDesktopTransport()) {
    return invokeDesktop<StoryArcCommandResponse>('command_story_delete', { command });
  }

  return request('/commands/story/delete-arc', {
    method: 'POST',
    body: JSON.stringify(command),
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

  return request('/commands/timeline/delete-node', {
    method: 'POST',
    body: JSON.stringify(command),
  });
}
