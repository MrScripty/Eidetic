import { expect, it, vi } from 'vitest';
import { createTimelineReorderDraft, reordered } from './timelineReorderDraft.svelte.js';
import type {
  ReorderTimelineSiblingCommand,
  TimelineSiblingOrderRead,
} from '$lib/timelineCommandTypes.js';
const initial: TimelineSiblingOrderRead = {
  node_id: 'a',
  parent_id: 'parent',
  level: 'Scene',
  membership_revision_event_id: null,
  siblings: [
    { node_id: 'a', name: 'A', start_ms: 100, end_ms: 200, sort_order: 1, revision_event_id: null },
    { node_id: 'b', name: 'B', start_ms: 300, end_ms: 500, sort_order: 2, revision_event_id: null },
  ],
};
function receipt(p: ReorderTimelineSiblingCommand) {
  const r = reordered(p.expected, p.neighbor_id)!;
  for (const n of r.siblings) n.revision_event_id = 'ack';
  return r;
}
function fixture() {
  const apply = vi.fn().mockImplementation(async (p: ReorderTimelineSiblingCommand) => receipt(p));
  const read = vi.fn().mockResolvedValue(initial);
  const commandId = vi.fn().mockReturnValue('swap-1');
  const draft = createTimelineReorderDraft({ nodeId: 'a', apply, read, commandId });
  draft.observe(initial);
  return { draft, apply, read, commandId };
}
it('choice is unwritten until explicit atomic save preserves duration/gap', async () => {
  const { draft, apply } = fixture();
  draft.state.neighborId = 'b';
  expect(apply).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply).toHaveBeenCalledWith(
    { node_id: 'a', neighbor_id: 'b', expected: initial },
    'swap-1',
  );
  expect(draft.state.base?.siblings.map((n) => [n.node_id, n.start_ms, n.end_ms])).toEqual([
    ['b', 100, 300],
    ['a', 400, 500],
  ]);
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(1);
});
it('membership and owned-clock ABA keep draft and block stale write', async () => {
  for (const current of [
    { ...initial, membership_revision_event_id: 'aba' },
    {
      ...initial,
      siblings: initial.siblings.map((n) => ({ ...n, revision_event_id: 'restored' })),
    },
  ]) {
    const { draft, apply } = fixture();
    draft.state.neighborId = 'b';
    draft.observe(current);
    await draft.apply();
    expect(apply).not.toHaveBeenCalled();
    expect(draft.state.neighborId).toBe('b');
    expect(draft.state.base).toEqual(initial);
  }
});
it('exact private pair survives transport/state mutation and newer observations', async () => {
  const { draft, apply, read, commandId } = fixture();
  draft.state.neighborId = 'b';
  draft.state.base!.siblings[0]!.name = 'forged';
  apply.mockImplementationOnce(async (p: ReorderTimelineSiblingCommand) => {
    p.expected.siblings[0]!.name = 'transport-mutated';
    p.neighbor_id = 'mutated';
    throw new Error('Lost ACK');
  });
  await draft.apply();
  expect(draft.state.uncertain).toBe(true);
  draft.observe({ ...initial, membership_revision_event_id: 'newer' });
  await draft.reload();
  expect(read).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply.mock.calls[1]).toEqual([
    { node_id: 'a', neighbor_id: 'b', expected: initial },
    'swap-1',
  ]);
  expect(commandId).toHaveBeenCalledTimes(1);
  expect(draft.state.current?.membership_revision_event_id).toBe('newer');
  expect(draft.state.saved).toBe(true);
});
it('only typed precommit refusal releases retry custody', async () => {
  const { draft, apply } = fixture();
  draft.state.neighborId = 'b';
  apply.mockRejectedValueOnce(new Error('Reorder refused: stale', { cause: { kind: 'conflict' } }));
  await draft.apply();
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.neighborId).toBe('b');
  apply.mockRejectedValueOnce(new Error('Publication failed', { cause: { kind: 'conflict' } }));
  await draft.apply();
  expect(draft.state.uncertain).toBe(true);
});
it('discard requires successful same-node explicit read', async () => {
  const { draft, read, apply } = fixture();
  draft.state.neighborId = 'b';
  read.mockRejectedValueOnce(new Error('offline'));
  await draft.reload();
  expect(draft.state.neighborId).toBe('b');
  read.mockResolvedValueOnce({ ...initial, node_id: 'other' });
  await draft.reload();
  expect(draft.state.neighborId).toBe('b');
  await draft.reload();
  expect(draft.state.neighborId).toBe(null);
  expect(apply).not.toHaveBeenCalled();
});
it('malformed ACK retains exact retry', async () => {
  const { draft, apply } = fixture();
  draft.state.neighborId = 'b';
  apply.mockResolvedValueOnce(initial);
  await draft.apply();
  expect(draft.state.uncertain).toBe(true);
  expect(draft.state.saved).toBe(false);
  await draft.apply();
  expect(draft.state.saved).toBe(true);
  expect(apply.mock.calls[0]![1]).toEqual(apply.mock.calls[1]![1]);
});
it('overlap and absent pair cannot be submitted', () => {
  expect(
    reordered(
      { ...initial, siblings: initial.siblings.map((n) => ({ ...n, start_ms: 100 })) },
      'b',
    ),
  ).toBeNull();
  expect(reordered(initial, 'missing')).toBeNull();
});
it('mirrors hierarchy adjacency rather than offering chronologically adjacent mismatched order', () => {
  const r = {
    ...initial,
    siblings: [
      { ...initial.siblings[0]! },
      { ...initial.siblings[1]!, sort_order: 3 },
      {
        node_id: 'c',
        name: 'C',
        start_ms: 600,
        end_ms: 700,
        sort_order: 2,
        revision_event_id: null,
      },
    ],
  };
  expect(reordered(r, 'b')).toBeNull();
});
it('near-max safe ranges preserve exact millisecond arithmetic without unsafe intermediate sums', () => {
  const r = {
    ...initial,
    siblings: [
      { ...initial.siblings[0]!, start_ms: 9007199254740881, end_ms: 9007199254740901 },
      { ...initial.siblings[1]!, start_ms: 9007199254740931, end_ms: 9007199254740950 },
    ],
  };
  expect(reordered(r, 'b')?.siblings.map((n) => [n.start_ms, n.end_ms])).toEqual([
    [9007199254740881, 9007199254740900],
    [9007199254740930, 9007199254740950],
  ]);
});

it('confirmed reorder remains acknowledged when a post-save projection adopts a newer lock clock', async () => {
  const { draft } = fixture();
  draft.state.neighborId = 'b';
  await draft.apply();
  draft.observe({
    ...draft.state.base!,
    siblings: draft.state.base!.siblings.map((n) => ({
      ...n,
      revision_event_id: n.node_id === 'a' ? 'later-lock' : n.revision_event_id,
    })),
  });
  expect(draft.state.saved).toBe(true);
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.base?.siblings.find((n) => n.node_id === 'a')?.revision_event_id).toBe(
    'later-lock',
  );
  await draft.reload();
  expect(draft.state.saved).toBe(false);
});
