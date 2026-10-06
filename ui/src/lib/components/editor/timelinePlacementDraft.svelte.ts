import type {
  SetTimelineNodeRangeCommand,
  TimelineNodeRangeRead,
} from '$lib/timelineCommandTypes.js';

export function createTimelinePlacementDraft(options: {
  nodeId: string;
  read: () => Promise<TimelineNodeRangeRead | null>;
  apply: (payload: SetTimelineNodeRangeCommand, commandId: string) => Promise<unknown>;
  commandId: () => string;
}) {
  const state = $state({
    base: null as TimelineNodeRangeRead | null,
    start: '',
    end: '',
    busy: false,
    uncertain: false,
    error: null as string | null,
    saved: false,
  });
  let submission: { payload: SetTimelineNodeRangeCommand; id: string } | null = null;
  let initialized = false;
  function initialize(read: TimelineNodeRangeRead | null | undefined) {
    if (!read || initialized || state.busy || submission) return;
    initialized = true;
    useRead(read);
  }
  function useRead(read: TimelineNodeRangeRead) {
    state.base = { ...read };
    state.start = String(read.start_ms / 1000);
    state.end = String(read.end_ms / 1000);
    state.error = null;
    state.saved = false;
  }
  async function reload() {
    if (state.busy || submission) return;
    state.busy = true;
    try {
      const read = await options.read();
      if (!read)
        throw new Error('Current placement is unavailable; save the project and read again.');
      useRead(read);
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
    } finally {
      state.busy = false;
    }
  }
  async function apply() {
    if (state.busy || !state.base) return;
    if (!submission) {
      const start_ms = secondsToMs(state.start);
      const end_ms = secondsToMs(state.end);
      if (start_ms === null || end_ms === null || end_ms <= start_ms) {
        state.error =
          'Enter nonnegative seconds with up to three decimal places, and an end after the start.';
        return;
      }
      submission = {
        id: options.commandId(),
        payload: {
          node_id: options.nodeId,
          start_ms,
          end_ms,
          expected: { ...state.base },
        },
      };
    }
    state.busy = true;
    state.error = null;
    state.saved = false;
    try {
      await options.apply(submission.payload, submission.id);
      submission = null;
      state.uncertain = false;
      state.base = null;
      state.saved = true;
      // The acknowledged projection supplies the new receipt on the next
      // explicit read. Never guess a revision from requested placement.
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
      const native = error instanceof Error ? error.cause : null;
      const refused =
        typeof native === 'object' &&
        native !== null &&
        'kind' in native &&
        (native.kind === 'conflict' || native.kind === 'bad_request') &&
        state.error.startsWith('Placement edit refused: ');
      state.uncertain = !refused;
      if (refused) submission = null;
    } finally {
      state.busy = false;
    }
  }
  return { state, initialize, reload, apply };
}

function secondsToMs(value: string): number | null {
  const match = /^(\d+)(?:\.(\d{1,3}))?$/.exec(value.trim());
  if (!match) return null;
  const result = Number(match[1]) * 1000 + Number((match[2] ?? '').padEnd(3, '0'));
  return Number.isSafeInteger(result) ? result : null;
}
