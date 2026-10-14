import { expect, it, vi } from 'vitest';
import { createTimelineArcsDraft } from './timelineArcsDraft.svelte.js';
import type {
  SetTimelineNodeArcsCommand,
  TimelineArcMembershipInput,
} from '$lib/timelineCommandTypes.js';
const initial: TimelineArcMembershipInput = {
  node_id: 'clip',
  arc_ids: [],
  revision_event_id: null,
};
function fixture() {
  const apply = vi.fn().mockImplementation(async (payload: SetTimelineNodeArcsCommand) => ({
    ...initial,
    arc_ids: [...payload.arc_ids],
    revision_event_id: 'ack',
  }));
  const read = vi
    .fn()
    .mockResolvedValue({ ...initial, arc_ids: ['current'], revision_event_id: 'new' });
  const commandId = vi.fn().mockReturnValueOnce('save-1').mockReturnValueOnce('save-2');
  const draft = createTimelineArcsDraft({ nodeId: 'clip', apply, read, commandId });
  draft.observe(initial);
  return { draft, apply, read, commandId };
}
it('assigns and clears only on explicit saves; reordered sets produce no write', async () => {
  const { draft, apply } = fixture();
  await draft.apply();
  expect(apply).not.toHaveBeenCalled();
  draft.state.arcIds = ['b', 'a'];
  draft.observe(initial);
  expect(apply).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply).toHaveBeenCalledWith(
    { node_id: 'clip', arc_ids: ['a', 'b'], expected: initial },
    'save-1',
  );
  draft.state.arcIds = ['a', 'b'];
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(1);
  draft.state.arcIds = [];
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(2);
  expect(apply.mock.calls[1]![0].expected.arc_ids).toEqual(['a', 'b']);
});
it('retains dirty intent for changed saved membership and same-set ABA', async () => {
  for (const current of [
    { ...initial, arc_ids: ['other'], revision_event_id: 'other' },
    { ...initial, revision_event_id: 'restored' },
  ]) {
    const { draft, apply } = fixture();
    draft.state.arcIds = ['mine'];
    draft.observe(current);
    await draft.apply();
    expect(apply).not.toHaveBeenCalled();
    expect(draft.state.arcIds).toEqual(['mine']);
    expect(draft.state.base).toEqual(initial);
  }
});
it('discard requires successful explicit same-clip read', async () => {
  const { draft, read, apply } = fixture();
  draft.state.arcIds = ['mine'];
  read.mockRejectedValueOnce(new Error('Unavailable'));
  await draft.reload();
  expect(draft.state.arcIds).toEqual(['mine']);
  read.mockResolvedValueOnce({ ...initial, node_id: 'other' });
  await draft.reload();
  expect(draft.state.arcIds).toEqual(['mine']);
  await draft.reload();
  expect(draft.state.arcIds).toEqual(['current']);
  expect(apply).not.toHaveBeenCalled();
});
it('retains immutable arrays and original command across uncertain transport and state mutation', async () => {
  const { draft, apply, read, commandId } = fixture();
  draft.state.arcIds = ['mine'];
  draft.state.base!.arc_ids.push('forged');
  apply.mockImplementationOnce(async (p: SetTimelineNodeArcsCommand) => {
    p.arc_ids.push('mutated');
    p.expected.arc_ids.push('mutated');
    throw new Error('Lost ack');
  });
  await draft.apply();
  expect(draft.state.uncertain).toBe(true);
  draft.state.arcIds = ['later'];
  await draft.reload();
  expect(read).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply.mock.calls[1]).toEqual([
    { node_id: 'clip', arc_ids: ['mine'], expected: initial },
    'save-1',
  ]);
  expect(commandId).toHaveBeenCalledTimes(1);
  expect(draft.state.arcIds).toEqual(['later']);
  expect(draft.state.base?.arc_ids).toEqual(['mine']);
});
it('acknowledges original receipt without adopting newer canonical membership', async () => {
  const { draft, apply } = fixture();
  let resolve!: (r: TimelineArcMembershipInput) => void;
  apply.mockReturnValueOnce(new Promise((done) => (resolve = done)));
  draft.state.arcIds = ['submitted'];
  const pending = draft.apply();
  draft.observe({ ...initial, arc_ids: ['concurrent'], revision_event_id: 'newer' });
  draft.state.arcIds = ['later'];
  resolve({ ...initial, arc_ids: ['submitted'], revision_event_id: 'ack' });
  await pending;
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(1);
  expect(draft.state.arcIds).toEqual(['later']);
  expect(draft.state.current?.arc_ids).toEqual(['concurrent']);
});
it('mismatched receipts remain uncertain and exact retries retain identity', async () => {
  for (const receipt of [
    null,
    { ...initial, node_id: 'other', arc_ids: ['mine'], revision_event_id: 'bad' },
    { ...initial, arc_ids: ['wrong'], revision_event_id: 'bad' },
    { ...initial, arc_ids: ['mine'] },
  ]) {
    const { draft, apply } = fixture();
    draft.state.arcIds = ['mine'];
    apply.mockResolvedValueOnce(receipt);
    await draft.apply();
    expect(draft.state.uncertain).toBe(true);
    await draft.apply();
    expect(apply.mock.calls[1]).toEqual(apply.mock.calls[0]);
  }
});
it('only typed precommit refusals release retry custody', async () => {
  for (const proven of [true, false]) {
    const { draft, apply, read } = fixture();
    draft.state.arcIds = ['mine'];
    const message = 'Arc assignment refused: stale';
    apply.mockRejectedValueOnce(
      new Error(message, proven ? { cause: { kind: 'conflict', message } } : undefined),
    );
    await draft.apply();
    expect(draft.state.uncertain).toBe(!proven);
    await draft.reload();
    expect(read).toHaveBeenCalledTimes(proven ? 1 : 0);
  }
});
