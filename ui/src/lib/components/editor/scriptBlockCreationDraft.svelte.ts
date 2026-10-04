import { createCommandId } from '$lib/commandTransport.js';
import type { CreateScriptBlockCommand, ScriptBlockKind } from '$lib/scriptTypes.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';

interface CreationDraft {
  writing: boolean;
  documentId: string | null;
  target: Pick<SelectedNodeEditorNode, 'node_id' | 'name' | 'start_ms' | 'end_ms'> | null;
  commandId: string | null;
  text: string;
  kind: ScriptBlockKind;
  saving: boolean;
  error: string | null;
  uncertain: boolean;
  placementRefused: boolean;
}

export function createScriptBlockCreationDraft(options: {
  documentId: () => string;
  save: (payload: CreateScriptBlockCommand, commandId: string) => Promise<unknown>;
  newCommandId?: () => string;
}): {
  readonly state: CreationDraft;
  begin: (node: SelectedNodeEditorNode) => void;
  cancel: () => void;
  save: () => Promise<void>;
  useCurrentPlacement: (node: SelectedNodeEditorNode) => Promise<void>;
} {
  const state = $state<CreationDraft>({
    writing: false,
    documentId: null,
    target: null,
    commandId: null,
    text: '',
    kind: 'action',
    saving: false,
    error: null,
    uncertain: false,
    placementRefused: false,
  });
  let submitted: { payload: CreateScriptBlockCommand; commandId: string } | null = null;
  function begin(node: SelectedNodeEditorNode): void {
    if (state.saving || submitted) return;
    state.target = {
      node_id: node.node_id,
      name: node.name,
      start_ms: node.start_ms,
      end_ms: node.end_ms,
    };
    state.documentId = options.documentId();
    state.commandId = (options.newCommandId ?? createCommandId)();
    state.text = '';
    state.kind = 'action';
    state.error = null;
    state.uncertain = false;
    state.placementRefused = false;
    state.writing = true;
  }
  function cancel(): void {
    if (state.saving || submitted) return;
    state.writing = false;
    state.target = null;
    state.commandId = null;
    state.documentId = null;
    state.text = '';
    state.error = null;
    state.uncertain = false;
    state.placementRefused = false;
  }
  async function save(): Promise<void> {
    if (state.saving) return;
    if (!submitted) {
      if (!state.target || !state.commandId || !state.documentId || !state.text.trim()) return;
      submitted = {
        payload: {
          document_id: state.documentId,
          source_node_id: state.target.node_id,
          expected_start_ms: state.target.start_ms,
          expected_end_ms: state.target.end_ms,
          block_kind: state.kind,
          text: state.text,
        },
        commandId: state.commandId,
      };
    }
    state.saving = true;
    state.error = null;
    state.placementRefused = false;
    try {
      // Retry the immutable submission, independent of mutable draft fields.
      // A fresh copy also prevents a transport caller from changing that snapshot.
      await options.save({ ...submitted.payload }, submitted.commandId);
      submitted = null;
      state.uncertain = false;
      state.writing = false;
      state.target = null;
      state.commandId = null;
      state.documentId = null;
      state.text = '';
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not write screenplay block';
      state.placementRefused = isPlacementRefusal(failure);
      state.uncertain = !state.placementRefused;
      // This exact native refusal precedes recording (and replay is checked
      // first). Other errors cannot establish whether the original save committed.
      if (state.placementRefused) submitted = null;
    } finally {
      state.saving = false;
    }
  }
  async function useCurrentPlacement(node: SelectedNodeEditorNode): Promise<void> {
    if (
      state.saving ||
      submitted ||
      !state.placementRefused ||
      node.node_id !== state.target?.node_id
    )
      return;
    state.target = { ...state.target, start_ms: node.start_ms, end_ms: node.end_ms };
    await save();
  }
  return {
    get state() {
      return state;
    },
    begin,
    cancel,
    save,
    useCurrentPlacement,
  };
}

function isPlacementRefusal(failure: unknown): boolean {
  if (!(failure instanceof Error)) return false;
  const nativeError = failure.cause;
  return (
    typeof nativeError === 'object' &&
    nativeError !== null &&
    'kind' in nativeError &&
    nativeError.kind === 'bad_request' &&
    failure.message === 'timeline placement changed; use its current placement and try again'
  );
}
