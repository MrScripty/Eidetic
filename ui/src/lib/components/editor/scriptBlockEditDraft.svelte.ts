import { createCommandId } from '$lib/commandTransport.js';
import type {
  EditScriptBlockCommand,
  RemoveScriptBlockCommand,
  ScriptBlockProjection,
  ScriptBlockKind,
} from '$lib/scriptTypes.js';

interface SavedComparison {
  text: string;
  revisionEventId: string;
}

export function createScriptBlockEditDraft(options: {
  documentId: string;
  blockId: string;
  save: (payload: EditScriptBlockCommand, commandId: string) => Promise<unknown>;
  remove?: (payload: RemoveScriptBlockCommand, commandId: string) => Promise<unknown>;
  reload: () => Promise<unknown>;
  readCurrent: () => Promise<ScriptBlockProjection | null>;
}) {
  const state = $state({
    removal: {
      active: false,
      text: '',
      saving: false,
      uncertain: false,
      error: null as string | null,
    },
    editing: false,
    text: '',
    baseRevision: null as string | null,
    saving: false,
    comparing: false,
    comparison: null as SavedComparison | null,
    error: null as string | null,
    uncertain: false,
  });
  let originalKind: ScriptBlockKind = 'action';
  let compared: SavedComparison | null = null;
  let submitted: { payload: EditScriptBlockCommand; commandId: string } | null = null;

  let removalCapture: RemoveScriptBlockCommand | null = null;
  let submittedRemoval: { payload: RemoveScriptBlockCommand; commandId: string } | null = null;

  function beginRemoval(block: ScriptBlockProjection): void {
    if (
      !options.remove ||
      state.editing ||
      state.saving ||
      state.comparing ||
      submitted ||
      state.removal.active ||
      block.block.id !== options.blockId ||
      !block.revision_event_id ||
      block.locks.length
    )
      return;
    removalCapture = {
      document_id: options.documentId,
      block_id: options.blockId,
      expected_revision_event_id: block.revision_event_id,
    };
    state.removal.text = block.block.text;
    state.removal.error = null;
    state.removal.active = true;
  }
  function cancelRemoval(): void {
    if (state.removal.saving || submittedRemoval) return;
    removalCapture = null;
    state.removal.active = false;
    state.removal.text = '';
    state.removal.error = null;
  }
  async function remove(): Promise<void> {
    if (!options.remove || !state.removal.active || !removalCapture || state.removal.saving) return;
    if (!submittedRemoval)
      submittedRemoval = { payload: { ...removalCapture }, commandId: createCommandId() };
    state.removal.saving = true;
    state.removal.error = null;
    try {
      await options.remove({ ...submittedRemoval.payload }, submittedRemoval.commandId);
      submittedRemoval = null;
      state.removal.uncertain = false;
      removalCapture = null;
      state.removal.active = false;
      state.removal.text = '';
    } catch (failure) {
      state.removal.error =
        failure instanceof Error ? failure.message : 'Could not remove script block';
      state.removal.uncertain = !isRemovalRefusal(failure);
      if (!state.removal.uncertain) submittedRemoval = null;
    } finally {
      state.removal.saving = false;
    }
  }

  function begin(block: ScriptBlockProjection): void {
    if (
      state.removal.active ||
      state.editing ||
      state.saving ||
      state.comparing ||
      submitted ||
      block.block.id !== options.blockId
    )
      return;
    originalKind = block.block.block_kind;
    state.text = block.block.text;
    state.baseRevision = block.revision_event_id ?? null;
    state.error = null;
    compared = null;
    state.comparison = null;
    state.editing = true;
  }
  function cancel(): void {
    if (state.removal.active || state.saving || state.comparing || submitted) return;
    compared = null;
    state.comparison = null;
    state.editing = false;
    state.text = '';
    state.baseRevision = null;
    state.error = null;
  }
  async function save(): Promise<void> {
    if (!state.editing || !state.baseRevision || state.saving || state.comparing) return;
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
      compared = null;
      state.comparison = null;
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
    if (state.removal.active || state.saving || state.comparing || submitted) return;
    state.saving = true;
    state.error = null;
    try {
      await options.reload();
      compared = null;
      state.comparison = null;
      state.editing = false;
      state.text = '';
      state.baseRevision = null;
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not reload script block';
    } finally {
      state.saving = false;
    }
  }
  async function compare(): Promise<void> {
    if (!state.editing || state.saving || state.comparing || submitted) return;
    state.comparing = true;
    state.error = null;
    compared = null;
    state.comparison = null;
    try {
      const current = await options.readCurrent();
      if (!current?.revision_event_id || current.block.id !== options.blockId) {
        throw new Error('Saved block is unavailable.');
      }
      compared = { text: current.block.text, revisionEventId: current.revision_event_id };
      // Display state cannot change the read snapshot used by explicit continuation.
      state.comparison = { ...compared };
    } catch (failure) {
      state.error = failure instanceof Error ? failure.message : 'Could not compare saved text';
    } finally {
      state.comparing = false;
    }
  }
  function useComparedRevision(current: ScriptBlockProjection): void {
    if (!state.editing || state.saving || state.comparing || submitted || !compared) return;
    if (
      current.block.id !== options.blockId ||
      current.revision_event_id !== compared.revisionEventId
    ) {
      state.error = 'Saved text changed again. Compare it before continuing.';
      compared = null;
      state.comparison = null;
      return;
    }
    state.baseRevision = compared.revisionEventId;
    state.error = null;
    compared = null;
    state.comparison = null;
  }
  return {
    documentId: options.documentId,
    blockId: options.blockId,
    get originalKind() {
      return originalKind;
    },
    state,
    begin,
    cancel,
    save,
    reload,
    compare,
    useComparedRevision,
    beginRemoval,
    cancelRemoval,
    remove,
  };
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

function isRemovalRefusal(failure: unknown): boolean {
  if (!(failure instanceof Error)) return false;
  const native = failure.cause;
  if (typeof native !== 'object' || native === null || !('kind' in native)) return false;
  return (
    (native.kind === 'bad_request' || native.kind === 'conflict') &&
    [
      'script block changed; reload before removing',
      'script block not found',
      'script document not found',
      'cannot remove a locked script block',
    ].includes(failure.message)
  );
}
