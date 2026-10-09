import { expect, it, vi } from 'vitest';
import { createTimelineNotesDraft } from './timelineNotesDraft.svelte.js';
import type { SetTimelineNodeNotesCommand } from '$lib/timelineCommandTypes.js';
import type { TimelineNotesInput } from '$lib/propagationProposalTypes.js';
const initial: TimelineNotesInput = { node_id: 'A', notes: 'Saved A', revision_event_id: null };
function fixture() {
  const apply = vi.fn().mockImplementation(async (p: SetTimelineNodeNotesCommand, _id: string) => ({
    ...initial,
    notes: p.notes,
    revision_event_id: 'ack',
  }));
  const read = vi
    .fn()
    .mockResolvedValue({ ...initial, notes: 'Current', revision_event_id: 'new' });
  const commandId = vi.fn().mockReturnValueOnce('save-1').mockReturnValueOnce('save-2');
  const draft = createTimelineNotesDraft({ nodeId: 'A', read, apply, commandId });
  draft.observe(initial);
  return { draft, apply, read, commandId };
}
it('persists only on explicit save, retaining exact whitespace and Unicode', async () => {
  const { draft, apply } = fixture();
  await draft.apply();
  expect(apply).not.toHaveBeenCalled();
  draft.state.notes = '  Mara reveals the witness — 雨.\n\n  ';
  draft.observe(initial);
  expect(apply).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply).toHaveBeenCalledWith(
    { node_id: 'A', notes: draft.state.notes, expected: initial },
    'save-1',
  );
  expect(draft.state.saved).toBe(true);
  expect(draft.state.base?.revision_event_id).toBe('ack');
});
it('keeps dirty drafts when canonical text or a same-value ABA clock changes', async () => {
  for (const current of [
    { ...initial, notes: 'Other', revision_event_id: 'other' },
    { ...initial, revision_event_id: 'restored' },
  ]) {
    const { draft, apply } = fixture();
    draft.state.notes = 'My draft';
    draft.observe(current);
    await draft.apply();
    expect(apply).not.toHaveBeenCalled();
    expect(draft.state.notes).toBe('My draft');
    expect(draft.state.base).toEqual(initial);
    expect(draft.state.current).toEqual(current);
  }
});
it('discards only after an explicit successful read; failed and wrong-node reads keep the text', async () => {
  const { draft, apply, read } = fixture();
  draft.state.notes = 'Unsaved';
  read.mockRejectedValueOnce(new Error('Unavailable'));
  await draft.reload();
  expect(draft.state.notes).toBe('Unsaved');
  read.mockResolvedValueOnce({ ...initial, node_id: 'B' });
  await draft.reload();
  expect(draft.state.notes).toBe('Unsaved');
  await draft.reload();
  expect(draft.state.notes).toBe('Current');
  expect(apply).not.toHaveBeenCalled();
});
it('retries the immutable original command after unknown acknowledgement and forbids discard', async () => {
  const { draft, apply, read, commandId } = fixture();
  draft.state.notes = 'First';
  apply.mockRejectedValueOnce(new Error('Lost acknowledgement'));
  await draft.apply();
  const captured = structuredClone(apply.mock.calls[0]);
  expect(draft.state.uncertain).toBe(true);
  draft.state.notes = 'Later';
  draft.state.base!.notes = 'Untrusted';
  await draft.reload();
  expect(read).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply.mock.calls[1]).toEqual(captured);
  expect(commandId).toHaveBeenCalledTimes(1);
  expect(draft.state.notes).toBe('Later');
  expect(draft.state.base?.notes).toBe('First');
  expect(draft.state.uncertain).toBe(false);
  await draft.apply();
  expect(apply.mock.calls[2]![1]).toBe('save-2');
  expect(apply.mock.calls[2]![0].expected!.notes).toBe('First');
});
it('keeps later input while saving and submits it only with another explicit save', async () => {
  const { draft, apply } = fixture();
  let resolve!: (r: TimelineNotesInput) => void;
  apply.mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  draft.state.notes = 'Submitted';
  const pending = draft.apply();
  draft.state.notes = 'Queued';
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(1);
  resolve({ ...initial, notes: 'Submitted', revision_event_id: 'ack' });
  await pending;
  expect(draft.state.notes).toBe('Queued');
  expect(draft.state.base?.notes).toBe('Submitted');
  expect(apply).toHaveBeenCalledTimes(1);
});
it('acknowledges its own receipt without adopting a newer observed canonical revision', async () => {
  const { draft, apply } = fixture();
  let resolve!: (r: TimelineNotesInput) => void;
  apply.mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  draft.state.notes = 'Submitted';
  const pending = draft.apply();
  draft.observe({ ...initial, notes: 'Concurrent', revision_event_id: 'newer' });
  draft.state.notes = 'Later';
  resolve({ ...initial, notes: 'Submitted', revision_event_id: 'ack' });
  await pending;
  await draft.apply();
  expect(apply).toHaveBeenCalledTimes(1);
  expect(draft.state.notes).toBe('Later');
  expect(draft.state.current?.notes).toBe('Concurrent');
});
it('recovers missing or wrong receipts by exact retry instead of guessing a revision', async () => {
  for (const receipt of [
    null,
    { ...initial, node_id: 'B', revision_event_id: 'bad' },
    { ...initial, notes: 'Submitted' },
  ]) {
    const { draft, apply } = fixture();
    draft.state.notes = 'Submitted';
    apply.mockResolvedValueOnce(receipt);
    await draft.apply();
    expect(draft.state.uncertain).toBe(true);
    expect(draft.state.base).toEqual(initial);
    await draft.apply();
    expect(apply.mock.calls[1]).toEqual(apply.mock.calls[0]);
  }
});
it('protects original guards and retry text from rendered-state and transport argument mutation', async () => {
  const { draft, apply } = fixture();
  draft.state.notes = 'Submitted';
  draft.state.base!.revision_event_id = 'forged';
  apply.mockImplementationOnce(async (p) => {
    p.notes = 'mutated';
    p.expected!.notes = 'mutated';
    throw new Error('Disconnected');
  });
  await draft.apply();
  await draft.apply();
  expect(apply.mock.calls[1]![0]).toEqual({ node_id: 'A', notes: 'Submitted', expected: initial });
});
it('only a proven precommit refusal releases retry custody', async () => {
  const { draft, apply } = fixture();
  draft.state.notes = 'Submitted';
  const message = 'Notes edit refused: Notes changed';
  apply.mockRejectedValueOnce(new Error(message, { cause: { kind: 'conflict', message } }));
  await draft.apply();
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.notes).toBe('Submitted');
  await draft.reload();
  expect(draft.state.notes).toBe('Current');
  const second = fixture();
  second.draft.state.notes = 'Submitted';
  second.apply.mockRejectedValueOnce(new Error(message));
  await second.draft.apply();
  expect(second.draft.state.uncertain).toBe(true);
});
it('preserves edits made during a pending discard/read', async () => {
  const { draft, read } = fixture();
  let resolve!: (r: TimelineNotesInput) => void;
  read.mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  draft.state.notes = 'Before';
  const pending = draft.reload();
  draft.state.notes = 'Typed during read';
  resolve(initial);
  await pending;
  expect(draft.state.notes).toBe('Typed during read');
});

it('retains acknowledged save status when its identical canonical projection arrives', async () => {
  const { draft } = fixture();
  draft.state.notes = 'Submitted';
  await draft.apply();
  draft.observe({ ...initial, notes: 'Submitted', revision_event_id: 'ack' });
  expect(draft.state.saved).toBe(true);
  expect(draft.state.notes).toBe('Submitted');
});
