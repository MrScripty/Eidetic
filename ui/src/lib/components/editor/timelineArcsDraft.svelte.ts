import type {
  TimelineArcMembershipInput,
  SetTimelineNodeArcsCommand,
} from '$lib/timelineCommandTypes.js';

const copy = (input: TimelineArcMembershipInput): TimelineArcMembershipInput => ({
  ...input,
  arc_ids: [...input.arc_ids],
});
export const sameArcIds = (a: string[], b: string[]) =>
  a.length === b.length && [...a].sort().every((id, i) => id === [...b].sort()[i]);
const same = (a: TimelineArcMembershipInput | null, b: TimelineArcMembershipInput | null) =>
  !!a &&
  !!b &&
  a.node_id === b.node_id &&
  sameArcIds(a.arc_ids, b.arc_ids) &&
  a.revision_event_id === b.revision_event_id;

/** Per-clip explicit intent; transport receives copies of privately retained retry custody. */
export function createTimelineArcsDraft(options: {
  nodeId: string;
  read: () => Promise<TimelineArcMembershipInput | null>;
  apply: (
    payload: SetTimelineNodeArcsCommand,
    id: string,
  ) => Promise<TimelineArcMembershipInput | null>;
  commandId: () => string;
}) {
  const state = $state({
    base: null as TimelineArcMembershipInput | null,
    current: null as TimelineArcMembershipInput | null,
    arcIds: [] as string[],
    busy: false,
    uncertain: false,
    saved: false,
    error: null as string | null,
  });
  let base: TimelineArcMembershipInput | null = null;
  let submission: { payload: SetTimelineNodeArcsCommand; id: string } | null = null;
  function useRead(read: TimelineArcMembershipInput) {
    base = copy(read);
    state.base = copy(read);
    state.current = copy(read);
    state.arcIds = [...read.arc_ids];
    state.saved = false;
    state.error = null;
  }
  function observe(read: TimelineArcMembershipInput | null | undefined) {
    if (!read || read.node_id !== options.nodeId) return;
    if (same(base, read)) state.current = copy(read);
    else if (!submission && !state.busy && (!base || sameArcIds(state.arcIds, base.arc_ids)))
      useRead(read);
    else state.current = copy(read);
  }
  async function reload() {
    if (state.busy || submission) return;
    const ids = [...state.arcIds];
    state.busy = true;
    state.error = null;
    try {
      const read = await options.read();
      if (!read || read.node_id !== options.nodeId)
        throw new Error('Current arc assignment is unavailable; save the project and read again.');
      if (!sameArcIds(state.arcIds, ids))
        throw new Error('The draft changed during the read; your assignment was kept.');
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
          'Saved arc assignment changed. Your draft is kept; discard it and read current arcs before saving.';
        return;
      }
      if (sameArcIds(state.arcIds, base.arc_ids)) return;
      submission = {
        id: options.commandId(),
        payload: {
          node_id: options.nodeId,
          arc_ids: [...state.arcIds].sort(),
          expected: copy(base),
        },
      };
    }
    const accepted = submission;
    state.busy = true;
    state.error = null;
    state.saved = false;
    try {
      const receipt = await options.apply(
        {
          ...accepted.payload,
          arc_ids: [...accepted.payload.arc_ids],
          expected: copy(accepted.payload.expected),
        },
        accepted.id,
      );
      if (
        !receipt ||
        receipt.node_id !== options.nodeId ||
        !sameArcIds(receipt.arc_ids, accepted.payload.arc_ids) ||
        !receipt.revision_event_id
      )
        throw new Error(
          'The save receipt is unavailable. Retry this exact save to recover its result.',
        );
      if (same(state.current, accepted.payload.expected)) state.current = copy(receipt);
      base = copy(receipt);
      state.base = copy(receipt);
      submission = null;
      state.uncertain = false;
      state.saved = true;
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
      const native = error instanceof Error ? error.cause : null;
      const refused =
        typeof native === 'object' &&
        native !== null &&
        'kind' in native &&
        (native.kind === 'conflict' || native.kind === 'bad_request') &&
        state.error.startsWith('Arc assignment refused: ');
      state.uncertain = !refused;
      if (refused) submission = null;
    } finally {
      state.busy = false;
    }
  }
  return { state, observe, reload, apply };
}
