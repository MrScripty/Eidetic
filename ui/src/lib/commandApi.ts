import type {
  BibleGraphNodeCommandResponse,
  BibleGraphNodeListCommandResponse,
  BibleGraphRootsCommandResponse,
  CreateBibleGraphNodeCommand,
  DeleteBibleGraphEdgeCommand,
  DeleteBibleGraphNodeCommand,
  EnsureCanonicalBibleRootsCommand,
  SetBibleGraphEdgeCommand,
  SetBibleGraphFieldCommand,
  SetBibleGraphSnapshotFieldCommand,
} from './bibleGraphTypes.js';
import type {
  AcceptAffectProposalCommand,
  AffectCommandResponse,
  AffectProposalCommandResponse,
  CreateAffectProposalCommand,
  RejectAffectProposalCommand,
  SetAffectValueCommand,
  SetAffectValueInput,
} from './affectTypes.js';
import type {
  CommandEnvelope,
  ObjectFieldCommandResponse,
  ProjectionEnvelope,
  SetObjectFieldCommand,
} from './projectionTypes.js';
import type {
  ContextInfluenceProjection,
  RecordContextEvaluationCommand,
} from './contextInfluenceTypes.js';
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
import { invokeDesktop } from './desktopTransport.js';
import { createCommandId } from './commandTransport.js';

export { createCommandId } from './commandTransport.js';
export {
  applyTimelineChildren,
  createTimelineChildFromParent,
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

  return invokeDesktop<ObjectFieldCommandResponse>('command_object_field', { command });
}

export function setAffectValue(
  payload: SetAffectValueInput,
  commandId = createCommandId(),
): Promise<AffectCommandResponse> {
  const command: CommandEnvelope<SetAffectValueCommand> = {
    id: commandId,
    payload: {
      command_id: commandId,
      ...payload,
    },
  };

  return invokeDesktop<AffectCommandResponse>('command_affect_set', { command });
}

export function createAffectProposal(
  payload: CreateAffectProposalCommand,
  commandId = createCommandId(),
): Promise<AffectProposalCommandResponse> {
  const command: CommandEnvelope<CreateAffectProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<AffectProposalCommandResponse>('command_affect_proposal_create', {
    command,
  });
}

export function rejectAffectProposal(
  payload: RejectAffectProposalCommand,
  commandId = createCommandId(),
): Promise<AffectProposalCommandResponse> {
  const command: CommandEnvelope<RejectAffectProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<AffectProposalCommandResponse>('command_affect_proposal_reject', {
    command,
  });
}

export function acceptAffectProposal(
  payload: AcceptAffectProposalCommand,
  commandId = createCommandId(),
): Promise<AffectProposalCommandResponse> {
  const command: CommandEnvelope<AcceptAffectProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<AffectProposalCommandResponse>('command_affect_proposal_accept', {
    command,
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

  return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_node', { command });
}

export function deleteBibleGraphNode(
  payload: DeleteBibleGraphNodeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeListCommandResponse> {
  const command: CommandEnvelope<DeleteBibleGraphNodeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleGraphNodeListCommandResponse>('command_bible_graph_delete_node', {
    command,
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

  return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_field', { command });
}

export function setBibleGraphEdge(
  payload: SetBibleGraphEdgeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<SetBibleGraphEdgeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_edge', { command });
}

export function deleteBibleGraphEdge(
  payload: DeleteBibleGraphEdgeCommand,
  commandId = createCommandId(),
): Promise<BibleGraphNodeCommandResponse> {
  const command: CommandEnvelope<DeleteBibleGraphEdgeCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_delete_edge', {
    command,
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

  return invokeDesktop<BibleGraphNodeCommandResponse>('command_bible_graph_snapshot_field', {
    command,
  });
}

export function ensureCanonicalBibleRoots(
  commandId = createCommandId(),
): Promise<BibleGraphRootsCommandResponse> {
  const command: CommandEnvelope<EnsureCanonicalBibleRootsCommand> = {
    id: commandId,
    payload: {},
  };

  return invokeDesktop<BibleGraphRootsCommandResponse>('command_bible_graph_roots', {
    command,
  });
}

export function recordContextEvaluation(
  payload: RecordContextEvaluationCommand,
  commandId = createCommandId(),
): Promise<ProjectionEnvelope<ContextInfluenceProjection>> {
  const command: CommandEnvelope<RecordContextEvaluationCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<ProjectionEnvelope<ContextInfluenceProjection>>(
    'command_context_evaluation',
    { command },
  );
}

export function createBibleReferenceProposal(
  payload: CreateBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<CreateBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleReferenceProposalCommandResponse>(
    'command_bible_reference_proposal_create',
    { command },
  );
}

export function rejectBibleReferenceProposal(
  payload: RejectBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<RejectBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleReferenceProposalCommandResponse>(
    'command_bible_reference_proposal_reject',
    { command },
  );
}

export function acceptBibleReferenceProposal(
  payload: AcceptBibleReferenceProposalCommand,
  commandId = createCommandId(),
): Promise<BibleReferenceProposalCommandResponse> {
  const command: CommandEnvelope<AcceptBibleReferenceProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<BibleReferenceProposalCommandResponse>(
    'command_bible_reference_proposal_accept',
    { command },
  );
}

export function createPropagationProposal(
  payload: CreatePropagationProposalCommand,
  commandId = createCommandId(),
): Promise<PropagationProposalCommandResponse> {
  const command: CommandEnvelope<CreatePropagationProposalCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<PropagationProposalCommandResponse>('command_propagation_proposal_create', {
    command,
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

  return invokeDesktop<PropagationProposalCommandResponse>('command_propagation_proposal_reject', {
    command,
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

  return invokeDesktop<PropagationProposalCommandResponse>('command_propagation_proposal_update', {
    command,
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

  return invokeDesktop<PropagationProposalCommandResponse>('command_propagation_proposal_accept', {
    command,
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

  return invokeDesktop<ScriptDocumentCommandResponse>('command_script_block', { command });
}

export function setScriptLock(
  payload: SetScriptLockCommand,
  commandId = createCommandId(),
): Promise<ScriptDocumentCommandResponse> {
  const command: CommandEnvelope<SetScriptLockCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<ScriptDocumentCommandResponse>('command_script_lock', { command });
}

export function createStoryArc(
  payload: CreateStoryArcCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<CreateStoryArcCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<StoryArcCommandResponse>('command_story_create', { command });
}

export function setStoryArcMetadata(
  payload: SetStoryArcMetadataCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<SetStoryArcMetadataCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<StoryArcCommandResponse>('command_story_update', { command });
}

export function deleteStoryArc(
  payload: DeleteStoryArcCommand,
  commandId = createCommandId(),
): Promise<StoryArcCommandResponse> {
  const command: CommandEnvelope<DeleteStoryArcCommand> = {
    id: commandId,
    payload,
  };

  return invokeDesktop<StoryArcCommandResponse>('command_story_delete', { command });
}
