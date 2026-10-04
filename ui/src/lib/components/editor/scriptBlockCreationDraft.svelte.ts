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
  });
  function begin(node: SelectedNodeEditorNode): void {
    if (state.saving) return;
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
    state.writing = true;
  }
  function cancel(): void {
    if (state.saving) return;
    state.writing = false;
    state.target = null;
    state.commandId = null;
    state.documentId = null;
    state.text = '';
    state.error = null;
  }
  async function save(): Promise<void> {
    if (
      !state.target ||
      !state.commandId ||
      !state.documentId ||
      state.saving ||
      !state.text.trim()
    )
      return;
    state.saving = true;
    state.error = null;
    try {
      await options.save(
        {
          document_id: state.documentId,
          source_node_id: state.target.node_id,
          expected_start_ms: state.target.start_ms,
          expected_end_ms: state.target.end_ms,
          block_kind: state.kind,
          text: state.text,
        },
        state.commandId,
      );
      state.writing = false;
      state.target = null;
      state.commandId = null;
      state.documentId = null;
      state.text = '';
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not write screenplay block';
    } finally {
      state.saving = false;
    }
  }
  async function useCurrentPlacement(node: SelectedNodeEditorNode): Promise<void> {
    if (state.saving || node.node_id !== state.target?.node_id) return;
    state.target = { ...state.target, start_ms: node.start_ms, end_ms: node.end_ms };
    // Keep the command identity: an ambiguous earlier acknowledgement must
    // never create a second block. Committed payload changes are refused.
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
