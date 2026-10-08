import type { TimelineNotesInput } from '$lib/propagationProposalTypes.js';
import type { SetTimelineNodeNotesCommand } from '$lib/timelineCommandTypes.js';

/** Session intent, separate from committed Notes and their existing owned clock. */
export function createTimelineNotesDraft(options: {
  nodeId: string;
  read: () => Promise<TimelineNotesInput | null>;
  apply: (payload: SetTimelineNodeNotesCommand, id: string) => Promise<TimelineNotesInput | null>;
  commandId: () => string;
}) {
  const state = $state({
    base: null as TimelineNotesInput | null,
    current: null as TimelineNotesInput | null,
    notes: '',
    busy: false,
    uncertain: false,
    saved: false,
    error: null as string | null,
  });
  // Rendered state is not the authority for either the guard or a retry payload.
  let base: TimelineNotesInput | null = null;
  let submission: { payload: SetTimelineNodeNotesCommand; id: string } | null = null;
  const same = (a: TimelineNotesInput | null, b: TimelineNotesInput | null) =>
    !!a &&
    !!b &&
    a.node_id === b.node_id &&
    a.notes === b.notes &&
    a.revision_event_id === b.revision_event_id;
  function useRead(read: TimelineNotesInput) {
    base = { ...read };
    state.base = { ...read };
    state.current = { ...read };
    state.notes = read.notes;
    state.saved = false;
    state.error = null;
  }
  function observe(read: TimelineNotesInput | null | undefined) {
    if (!read || read.node_id !== options.nodeId) return;
    if (same(base, read)) state.current = { ...read };
    else if (!submission && !state.busy && (!base || state.notes === base.notes)) useRead(read);
    else state.current = { ...read };
  }
  async function reload() {
    if (state.busy || submission) return;
    const text = state.notes;
    state.busy = true;
    state.error = null;
    try {
      const read = await options.read();
      if (!read || read.node_id !== options.nodeId)
        throw new Error('Current Notes are unavailable; save the project and read again.');
      if (state.notes !== text)
        throw new Error('The draft changed during the read; your text was kept.');
      useRead(read);
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
    } finally {
      state.busy = false;
    }
  }
  async function apply() {
    if (state.busy || !base) return;
    if (!submission) {
      if (!same(base, state.current)) {
        state.error =
          'Saved Notes changed. Your draft is kept; discard it and read current Notes before saving.';
        return;
      }
      if (state.notes === base.notes) return;
      submission = {
        id: options.commandId(),
        payload: { node_id: options.nodeId, notes: state.notes, expected: { ...base } },
      };
    }
    const accepted = submission;
    state.busy = true;
    state.error = null;
    state.saved = false;
    try {
      // Give the transport a copy: retry custody remains private and immutable.
      const receipt = await options.apply(
        { ...accepted.payload, expected: { ...accepted.payload.expected! } },
        accepted.id,
      );
      if (
        !receipt ||
        receipt.node_id !== options.nodeId ||
        receipt.notes !== accepted.payload.notes ||
        !receipt.revision_event_id
      )
        throw new Error(
          'The save receipt is unavailable. Retry this exact save to recover its result.',
        );
      if (same(state.current, accepted.payload.expected ?? null)) state.current = { ...receipt };
      base = { ...receipt };
      state.base = { ...receipt };
      submission = null;
      state.uncertain = false;
      state.saved = true;
      // A later edit to state.notes remains a separate, unsaved intent.
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
      const native = error instanceof Error ? error.cause : null;
      const refused =
        typeof native === 'object' &&
        native !== null &&
        'kind' in native &&
        (native.kind === 'conflict' || native.kind === 'bad_request') &&
        state.error.startsWith('Notes edit refused: ');
      state.uncertain = !refused;
      if (refused) submission = null;
    } finally {
      state.busy = false;
    }
  }
  return { state, observe, reload, apply };
}
