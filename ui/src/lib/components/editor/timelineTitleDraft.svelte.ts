import type {
  SetTimelineNodeNameCommand,
  TimelineNodeNameRead,
} from '$lib/timelineCommandTypes.js';

export function createTimelineTitleDraft(options: {
  nodeId: string;
  read: () => Promise<TimelineNodeNameRead | null>;
  apply: (payload: SetTimelineNodeNameCommand, commandId: string) => Promise<unknown>;
  commandId: () => string;
}) {
  const state = $state({
    base: null as TimelineNodeNameRead | null,
    name: '',
    busy: false,
    uncertain: false,
    error: null as string | null,
    saved: false,
  });
  let submission: { payload: SetTimelineNodeNameCommand; id: string } | null = null;
  let initialized = false;
  function initialize(read: TimelineNodeNameRead | null | undefined) {
    if (!read || initialized || state.busy || submission) return;
    initialized = true;
    useRead(read);
  }
  function useRead(read: TimelineNodeNameRead) {
    state.base = { ...read };
    state.name = read.name;
    state.error = null;
    state.saved = false;
  }
  async function reload() {
    if (state.busy || submission) return;
    state.busy = true;
    try {
      const read = await options.read();
      if (!read) throw new Error('Current title is unavailable; save the project and read again.');
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
      if (
        !state.name.trim() ||
        [...state.name].length > 1024 ||
        [...state.name].some((character) => {
          const code = character.codePointAt(0)!;
          return code < 32 || (code >= 127 && code <= 159);
        })
      ) {
        state.error = 'Enter a title of 1–1024 characters without control characters.';
        return;
      }
      submission = {
        id: options.commandId(),
        payload: {
          node_id: options.nodeId,
          name: state.name,
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
      // The acknowledged projection supplies the new title receipt on the next
      // explicit read. Never guess a revision from requested title.
    } catch (error) {
      state.error = error instanceof Error ? error.message : String(error);
      const native = error instanceof Error ? error.cause : null;
      const refused =
        typeof native === 'object' &&
        native !== null &&
        'kind' in native &&
        (native.kind === 'conflict' || native.kind === 'bad_request') &&
        state.error.startsWith('Title edit refused: ');
      state.uncertain = !refused;
      if (refused) submission = null;
    } finally {
      state.busy = false;
    }
  }
  return { state, initialize, reload, apply };
}
