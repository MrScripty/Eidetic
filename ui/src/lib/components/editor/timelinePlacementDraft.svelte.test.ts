import { describe, expect, it, vi } from 'vitest';
import { createTimelinePlacementDraft } from './timelinePlacementDraft.svelte.js';

const read = { start_ms: 1000, end_ms: 2000, node_revision_event_id: 'original' };
function fixture() {
  const apply = vi.fn<(...args: unknown[]) => Promise<unknown>>().mockResolvedValue({});
  const load = vi.fn().mockResolvedValue({ ...read, node_revision_event_id: 'fresh' });
  const draft = createTimelinePlacementDraft({
    nodeId: 'A',
    read: load,
    apply,
    commandId: () => 'placement-command',
  });
  draft.initialize(read);
  return { draft, apply, load };
}

describe('explicit placement intent', () => {
  it('submits exact milliseconds and the captured revision only after Apply', async () => {
    const { draft, apply } = fixture();
    draft.state.start = '6.125';
    draft.state.end = '7.001';
    draft.initialize({ ...read, start_ms: 4000, node_revision_event_id: 'later' });
    expect(apply).not.toHaveBeenCalled();
    await draft.apply();
    expect(apply).toHaveBeenCalledWith(
      { node_id: 'A', start_ms: 6125, end_ms: 7001, expected: read },
      'placement-command',
    );
    expect(draft.state.saved).toBe(true);
    expect(draft.state.base).toBeNull();
    draft.initialize(read);
    expect(draft.state.base).toBeNull();
  });
  it('retains an immutable retry after an unknown acknowledgement', async () => {
    const { draft, apply, load } = fixture();
    draft.state.start = '6';
    draft.state.end = '7';
    apply.mockRejectedValueOnce(new Error('Connection closed'));
    await draft.apply();
    expect(draft.state.uncertain).toBe(true);
    const submitted = structuredClone(apply.mock.calls[0]);
    draft.state.start = '9'; // Even a stale external caller cannot amend retry.
    await draft.reload();
    expect(load).not.toHaveBeenCalled();
    await draft.apply();
    expect(apply.mock.calls[1]).toEqual(submitted);
    expect(draft.state.uncertain).toBe(false);
  });
  it('keeps exact entered times on a proven native refusal and reloads only explicitly', async () => {
    const { draft, apply, load } = fixture();
    draft.state.start = '6';
    draft.state.end = '7';
    const message =
      'Placement edit refused: placement changed; read current placement before applying';
    apply.mockRejectedValueOnce(new Error(message, { cause: { kind: 'conflict', message } }));
    await draft.apply();
    expect(draft.state.uncertain).toBe(false);
    expect([draft.state.start, draft.state.end]).toEqual(['6', '7']);
    expect(load).not.toHaveBeenCalled();
    await draft.reload();
    expect(draft.state.base?.node_revision_event_id).toBe('fresh');
    expect([draft.state.start, draft.state.end]).toEqual(['1', '2']);
  });
  it('does not trust a refusal-looking unproven error or a failed reload', async () => {
    const { draft, apply } = fixture();
    apply.mockRejectedValueOnce(new Error('Placement edit refused: invented'));
    await draft.apply();
    expect(draft.state.uncertain).toBe(true);
    const other = fixture();
    other.draft.state.start = '6';
    other.draft.state.end = '7';
    other.load.mockRejectedValueOnce(new Error('Read unavailable'));
    await other.draft.reload();
    expect(other.draft.state.base).toEqual(read);
    expect([other.draft.state.start, other.draft.state.end]).toEqual(['6', '7']);
  });
  it('refuses invalid and imprecise input without a command', async () => {
    const { draft, apply } = fixture();
    for (const [start, end] of [
      ['-1', '2'],
      ['1.0001', '2'],
      ['1e3', '2000'],
      ['2', '1'],
      ['', '2'],
      ['9007199254740992', '9007199254740993'],
    ] as const) {
      draft.state.start = start;
      draft.state.end = end;
      await draft.apply();
      expect(draft.state.error).toContain('Enter nonnegative seconds');
    }
    expect(apply).not.toHaveBeenCalled();
  });
});
