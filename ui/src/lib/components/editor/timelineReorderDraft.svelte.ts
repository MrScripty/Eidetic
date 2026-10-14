import type {
  TimelineSiblingOrderRead,
  ReorderTimelineSiblingCommand,
} from '$lib/timelineCommandTypes.js';
const copy = (r: TimelineSiblingOrderRead): TimelineSiblingOrderRead => ({
  ...r,
  siblings: r.siblings.map((n) => ({ ...n })),
});
export const sameOrder = (a: TimelineSiblingOrderRead | null, b: TimelineSiblingOrderRead | null) =>
  !!a &&
  !!b &&
  a.node_id === b.node_id &&
  a.parent_id === b.parent_id &&
  a.level === b.level &&
  a.membership_revision_event_id === b.membership_revision_event_id &&
  a.siblings.length === b.siblings.length &&
  a.siblings.every((n, i) => {
    const other = b.siblings[i]!;
    return (
      n.node_id === other.node_id &&
      n.name === other.name &&
      n.start_ms === other.start_ms &&
      n.end_ms === other.end_ms &&
      n.sort_order === other.sort_order &&
      n.revision_event_id === other.revision_event_id
    );
  });
export function reordered(
  read: TimelineSiblingOrderRead,
  neighborId: string,
): TimelineSiblingOrderRead | null {
  const result = copy(read);
  const a = result.siblings.findIndex((n) => n.node_id === read.node_id);
  const b = result.siblings.findIndex((n) => n.node_id === neighborId);
  if (a < 0 || b < 0 || Math.abs(a - b) !== 1 || !read.parent_id) return null;
  const first = result.siblings[Math.min(a, b)]!;
  const second = result.siblings[Math.max(a, b)]!;
  if (first.end_ms > second.start_ms || first.sort_order > second.sort_order) return null;
  const hierarchy = [...result.siblings].sort(
    (a, b) =>
      a.sort_order - b.sort_order || a.start_ms - b.start_ms || a.node_id.localeCompare(b.node_id),
  );
  if (hierarchy[hierarchy.indexOf(first) + 1] !== second) return null;
  if (
    !result.siblings.every(
      (n) => [n.start_ms, n.end_ms].every(Number.isSafeInteger) && n.start_ms < n.end_ms,
    )
  )
    return null;
  const end = first.start_ms + (second.end_ms - second.start_ms);
  const start = end + (second.start_ms - first.end_ms);
  if (![start, end, first.start_ms, second.end_ms].every(Number.isSafeInteger)) return null;
  const outerStart = first.start_ms,
    outerEnd = second.end_ms;
  if (
    result.siblings.some(
      (n) => n !== first && n !== second && n.start_ms < outerEnd && n.end_ms > outerStart,
    )
  )
    return null;
  [first.sort_order, second.sort_order] = [second.sort_order, first.sort_order];
  first.end_ms = outerEnd;
  first.start_ms = start;
  second.start_ms = outerStart;
  second.end_ms = end;
  result.siblings.sort((a, b) => a.start_ms - b.start_ms || a.node_id.localeCompare(b.node_id));
  return result;
}
export function createTimelineReorderDraft(options: {
  nodeId: string;
  read: () => Promise<TimelineSiblingOrderRead | null>;
  apply: (p: ReorderTimelineSiblingCommand, id: string) => Promise<TimelineSiblingOrderRead | null>;
  commandId: () => string;
}) {
  const state = $state({
    base: null as TimelineSiblingOrderRead | null,
    current: null as TimelineSiblingOrderRead | null,
    neighborId: null as string | null,
    busy: false,
    uncertain: false,
    saved: false,
    error: null as string | null,
  });
  let base: TimelineSiblingOrderRead | null = null;
  let submission: { payload: ReorderTimelineSiblingCommand; id: string } | null = null;
  function useRead(read: TimelineSiblingOrderRead) {
    base = copy(read);
    state.base = copy(read);
    state.current = copy(read);
    state.neighborId = null;
    state.saved = false;
    state.error = null;
  }
  function observe(read: TimelineSiblingOrderRead | null | undefined) {
    if (!read || read.node_id !== options.nodeId) return;
    if (sameOrder(base, read)) state.current = copy(read);
    else if (!submission && !state.busy && (!base || !state.neighborId)) {
      const acknowledged = state.saved;
      useRead(read);
      state.saved = acknowledged;
    } else state.current = copy(read);
  }
  async function reload() {
    if (state.busy || submission) return;
    const intent = state.neighborId;
    state.busy = true;
    state.error = null;
    try {
      const read = await options.read();
      if (!read || read.node_id !== options.nodeId)
        throw new Error('Current siblings are unavailable; save the project and read again.');
      if (state.neighborId !== intent)
        throw new Error('Reorder draft changed during the read; it was kept.');
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
      if (!sameOrder(base, state.current)) {
        state.error =
          'Saved siblings changed. Your reorder draft is kept; discard it and read current siblings before applying.';
        return;
      }
      if (!state.neighborId || !reordered(base, state.neighborId)) return;
      submission = {
        id: options.commandId(),
        payload: { node_id: options.nodeId, neighbor_id: state.neighborId, expected: copy(base) },
      };
    }
    const accepted = submission;
    state.busy = true;
    state.error = null;
    state.saved = false;
    try {
      const receipt = await options.apply(
        { ...accepted.payload, expected: copy(accepted.payload.expected) },
        accepted.id,
      );
      const expected = reordered(accepted.payload.expected, accepted.payload.neighbor_id);
      if (!receipt || !expected)
        throw new Error(
          'Reorder receipt unavailable. Retry this exact reorder to recover its result.',
        );
      const pair = [accepted.payload.node_id, accepted.payload.neighbor_id];
      const clocks = receipt.siblings
        .filter((n) => pair.includes(n.node_id))
        .map((n) => n.revision_event_id);
      const normalized = copy(receipt);
      for (const n of normalized.siblings) {
        if (pair.includes(n.node_id))
          n.revision_event_id = expected.siblings.find(
            (v) => v.node_id === n.node_id,
          )!.revision_event_id;
      }
      if (
        clocks.length !== 2 ||
        !clocks[0] ||
        clocks[0] !== clocks[1] ||
        !sameOrder(normalized, expected)
      )
        throw new Error(
          'Reorder receipt differs from the submitted pair. Retry the exact reorder.',
        );
      if (sameOrder(state.current, accepted.payload.expected)) state.current = copy(receipt);
      base = copy(receipt);
      state.base = copy(receipt);
      if (state.neighborId === accepted.payload.neighbor_id) state.neighborId = null;
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
        state.error.startsWith('Reorder refused: ');
      state.uncertain = !refused;
      if (refused) submission = null;
    } finally {
      state.busy = false;
    }
  }
  return { state, observe, reload, apply };
}
