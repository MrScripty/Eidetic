import { expect, it, vi } from 'vitest';
import { createTimelineTitleDraft } from './timelineTitleDraft.svelte.js';
const read = { name: 'SCENE A', revision_event_id: null };
function fixture() {
  const apply = vi.fn().mockResolvedValue({});
  const load = vi.fn().mockResolvedValue({ name: 'Current', revision_event_id: 'new' });
  const draft = createTimelineTitleDraft({
    nodeId: 'A',
    read: load,
    apply,
    commandId: () => 'rename',
  });
  draft.initialize(read);
  return { draft, apply, load };
}
it('saves exact Unicode/whitespace only explicitly, with its original read', async () => {
  const { draft, apply } = fixture();
  draft.state.name = '  Station departure — 雨.  ';
  draft.initialize({ name: 'Concurrent', revision_event_id: 'later' });
  expect(apply).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply).toHaveBeenCalledWith(
    { node_id: 'A', name: '  Station departure — 雨.  ', expected: read },
    'rename',
  );
  expect(draft.state.saved).toBe(true);
  expect(draft.state.base).toBeNull();
  draft.initialize(read);
  expect(draft.state.base).toBeNull();
});
it('retains immutable retry and blocks reload after unknown acknowledgement', async () => {
  const { draft, apply, load } = fixture();
  draft.state.name = 'Saved';
  apply.mockRejectedValueOnce(new Error('Lost acknowledgement'));
  await draft.apply();
  const first = structuredClone(apply.mock.calls[0]);
  expect(draft.state.uncertain).toBe(true);
  draft.state.name = 'Amended by stale caller';
  await draft.reload();
  expect(load).not.toHaveBeenCalled();
  await draft.apply();
  expect(apply.mock.calls[1]).toEqual(first);
});
it('preserves refused draft and failed reload; uses new receipt only after explicit reload', async () => {
  const { draft, apply, load } = fixture();
  draft.state.name = 'My draft';
  const message = 'Title edit refused: title changed';
  apply.mockRejectedValueOnce(new Error(message, { cause: { kind: 'conflict', message } }));
  await draft.apply();
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.name).toBe('My draft');
  load.mockRejectedValueOnce(new Error('Unavailable'));
  await draft.reload();
  expect(draft.state.name).toBe('My draft');
  expect(draft.state.base).toEqual(read);
  await draft.reload();
  expect(draft.state.name).toBe('Current');
  expect(draft.state.base?.revision_event_id).toBe('new');
});
it('does not treat an unproven message as a native refusal', async () => {
  const { draft, apply } = fixture();
  apply.mockRejectedValueOnce(new Error('Title edit refused: invented'));
  await draft.apply();
  expect(draft.state.uncertain).toBe(true);
});
it('rejects blank/control/oversize titles without invoking a command', async () => {
  const { draft, apply } = fixture();
  for (const name of ['   ', 'line\nbreak', '\u0000', 'a'.repeat(1025)]) {
    draft.state.name = name;
    await draft.apply();
    expect(draft.state.error).toContain('1–1024');
  }
  expect(apply).not.toHaveBeenCalled();
});
