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
import { hasDesktopTransport, invokeDesktop } from './desktopTransport.js';
import { createCommandId, request } from './commandTransport.js';

export { createCommandId } from './commandTransport.js';
export {
  applyTimelineChildren,
  createTimelineNode,
  createTimelineRelationship,
  deleteTimelineNode,
  deleteTimelineRelationship,
  setTimelineNodeLock,
  setTimelineNodeNotes,
  setTimelineNodeRange,
  splitTimelineNode,
} from './timelineCommandApi.js';

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

  if (hasDesktopTransport()) {
    return invokeDesktop<PropagationProposalCommandResponse>(
      'command_propagation_proposal_create',
      { command },
    );
  }

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

  if (hasDesktopTransport()) {
    return invokeDesktop<PropagationProposalCommandResponse>(
      'command_propagation_proposal_reject',
      { command },
    );
  }

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

  if (hasDesktopTransport()) {
    return invokeDesktop<PropagationProposalCommandResponse>(
      'command_propagation_proposal_update',
      { command },
    );
  }

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

  if (hasDesktopTransport()) {
    return invokeDesktop<PropagationProposalCommandResponse>(
      'command_propagation_proposal_accept',
      { command },
    );
  }

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
