import { createCommandId } from '$lib/commandTransport.js';
import type { EditScriptBlockCommand, ScriptBlockProjection } from '$lib/scriptTypes.js';

export function createScriptBlockEditDraft(options: {
  documentId: string;
  blockId: string;
  save: (payload: EditScriptBlockCommand, commandId: string) => Promise<unknown>;
  reload: () => Promise<unknown>;
}) {
  const state = $state({
    editing: false,
    text: '',
    baseRevision: null as string | null,
    saving: false,
    error: null as string | null,
    uncertain: false,
  });
  let submitted: { payload: EditScriptBlockCommand; commandId: string } | null = null;

  function begin(block: ScriptBlockProjection): void {
    if (state.editing || state.saving || submitted || block.block.id !== options.blockId) return;
    state.text = block.block.text;
    state.baseRevision = block.revision_event_id ?? null;
    state.error = null;
    state.editing = true;
  }
  function cancel(): void {
    if (state.saving || submitted) return;
    state.editing = false;
    state.text = '';
    state.baseRevision = null;
    state.error = null;
  }
  async function save(): Promise<void> {
    if (!state.editing || !state.baseRevision || state.saving) return;
    if (!submitted) {
      submitted = {
        payload: {
          document_id: options.documentId,
          block_id: options.blockId,
          expected_revision_event_id: state.baseRevision,
          text: state.text,
        },
        commandId: createCommandId(),
      };
    }
    state.saving = true;
    state.error = null;
    try {
      await options.save({ ...submitted.payload }, submitted.commandId);
      submitted = null;
      state.uncertain = false;
      state.editing = false;
      state.text = '';
      state.baseRevision = null;
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not save script block';
      state.uncertain = !isEditRefusal(failure);
      if (!state.uncertain) submitted = null;
    } finally {
      state.saving = false;
    }
  }
  async function reload(): Promise<void> {
    if (state.saving || submitted) return;
    state.saving = true;
    state.error = null;
    try {
      await options.reload();
      state.editing = false;
      state.text = '';
      state.baseRevision = null;
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not reload script block';
    } finally {
      state.saving = false;
    }
  }
  return { state, begin, cancel, save, reload };
}

function isEditRefusal(failure: unknown): boolean {
  if (!(failure instanceof Error)) return false;
  const native = failure.cause;
  if (typeof native !== 'object' || native === null || !('kind' in native)) return false;
  // These exact edit-path failures precede recording or roll back its writer
  // transaction. Replay is checked first; broad conflict/error labels alone
  // cannot establish whether an acknowledgement was lost after a commit.
  return (
    (native.kind === 'bad_request' || native.kind === 'conflict') &&
    [
      'script block changed; reload before saving',
      'script block not found',
      'script document not found',
      'invalid command: script block update would remove locked span text',
      'invalid command: script block update would modify locked span text',
    ].includes(failure.message)
  );
}
