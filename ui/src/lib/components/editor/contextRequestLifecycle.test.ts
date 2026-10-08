import { describe, expect, it, vi } from 'vitest';
import { createContextRequestLifecycle } from './contextRequestLifecycle.js';

function fixture() {
  const state = { selected: 'A' as string | null, context: null as unknown, loading: false };
  const pending: {
    resolve: (value: { system: string; user: string }) => void;
    reject: (error: Error) => void;
  }[] = [];
  const fetchContext = vi.fn(
    () =>
      new Promise<{ system: string; user: string }>((resolve, reject) =>
        pending.push({ resolve, reject }),
      ),
  );
  const lifecycle = createContextRequestLifecycle({
    selectedNodeId: () => state.selected,
    fetchContext,
    setContext: (context) => (state.context = context),
    setLoading: (loading) => (state.loading = loading),
  });
  function request(index: number) {
    const request = pending[index];
    if (!request) throw new Error(`Context request ${index} was not started`);
    return request;
  }
  return { state, pending: request, lifecycle, fetchContext };
}

async function settle() {
  await Promise.resolve();
  await Promise.resolve();
}

describe('context request lifecycle', () => {
  it.each([
    ['Mara waits.', '  Mara reveals the witness — 雨.\n\n  '],
    ['Mara waits.', '  Mara waits.\n\n  '],
  ])(
    'refreshes exact same-node Notes %j -> %j without a script revision',
    async (before, after) => {
      const { state, pending, lifecycle, fetchContext } = fixture();
      lifecycle.update('A', before, 1);
      pending(0).resolve({ system: 'same instructions', user: before });
      await settle();
      lifecycle.update('A', after, 1);
      expect(fetchContext).toHaveBeenCalledTimes(2);
      expect(state.context).toBeNull();
      expect(state.loading).toBe(true);
      pending(1).resolve({ system: 'same instructions', user: after });
      await settle();
      expect(state.context).toEqual({ system: 'same instructions', user: after });
      expect(state.loading).toBe(false);
      lifecycle.update('A', after, 1);
      expect(fetchContext).toHaveBeenCalledTimes(2);
    },
  );

  it.each([false, true])(
    'refuses old same-node Notes ABA completion (failure=%j)',
    async (reject) => {
      const { state, pending, lifecycle, fetchContext } = fixture();
      lifecycle.update('A', '  Original — 雨.\n\n', 1);
      lifecycle.update('A', 'Edited Notes', 1);
      lifecycle.update('A', '  Original — 雨.\n\n', 1);
      expect(fetchContext).toHaveBeenCalledTimes(3);
      if (reject) pending(0).reject(new Error('late original failure'));
      else pending(0).resolve({ system: 'old', user: 'old original' });
      pending(1).resolve({ system: 'old', user: 'intervening Notes' });
      await settle();
      expect(state.context).toBeNull();
      expect(state.loading).toBe(true);
      pending(2).resolve({ system: 'fresh', user: '  Original — 雨.\n\n' });
      await settle();
      expect(state.context).toEqual({ system: 'fresh', user: '  Original — 雨.\n\n' });
      expect(state.loading).toBe(false);
    },
  );

  it('a manual Refresh pending before Notes edit cannot replace the newer prompt', async () => {
    const { state, pending, lifecycle } = fixture();
    lifecycle.update('A', 'Original Notes', 1);
    pending(0).resolve({ system: 'first', user: 'Original Notes' });
    await settle();
    const refresh = lifecycle.load('A');
    lifecycle.update('A', 'New Notes', 1);
    pending(2).resolve({ system: 'fresh', user: 'New Notes' });
    await settle();
    pending(1).resolve({ system: 'old refresh', user: 'Original Notes' });
    await refresh;
    expect(state.context).toEqual({ system: 'fresh', user: 'New Notes' });
    expect(state.loading).toBe(false);
  });

  it('a failed fresh Notes read keeps old context unavailable even after an older success', async () => {
    const { state, pending, lifecycle } = fixture();
    lifecycle.update('A', 'Original Notes', 1);
    lifecycle.update('A', 'New Notes', 1);
    pending(1).reject(new Error('current read unavailable'));
    await settle();
    expect(state.context).toBeNull();
    expect(state.loading).toBe(false);
    pending(0).resolve({ system: 'old', user: 'Original Notes' });
    await settle();
    expect(state.context).toBeNull();
    expect(state.loading).toBe(false);
  });

  it.each(['', '   '])(
    'clearing notes to %j clears loading and rejects stale same-node completion',
    async (notes) => {
      const { state, pending, lifecycle } = fixture();
      lifecycle.update('A', 'notes', 1);
      expect(state.loading).toBe(true);
      lifecycle.update('A', notes, 1);
      expect(state.loading).toBe(false);
      pending(0).resolve({ system: 'old', user: 'old' });
      await settle();
      expect(state.context).toBeNull();
      expect(state.loading).toBe(false);
    },
  );

  it.each(['B', null])(
    'selecting %j without notes invalidates success and failure callbacks',
    async (selected) => {
      for (const reject of [false, true]) {
        const { state, pending, lifecycle } = fixture();
        lifecycle.update('A', 'notes', 1);
        state.selected = selected;
        lifecycle.update(selected, undefined, 1);
        expect(state.loading).toBe(false);
        if (reject) pending(0).reject(new Error('old failure'));
        else pending(0).resolve({ system: 'old', user: 'old' });
        await settle();
        expect(state.context).toBeNull();
        expect(state.loading).toBe(false);
      }
    },
  );

  it('stale completion cannot clear loading or context for a newer request', async () => {
    const { state, pending, lifecycle } = fixture();
    lifecycle.update('A', 'notes', 1);
    lifecycle.update('A', '', 1);
    lifecycle.update('A', 'restored notes', 2);
    pending(0).resolve({ system: 'old', user: 'old' });
    await settle();
    expect(state.loading).toBe(true);
    expect(state.context).toBeNull();
    pending(1).resolve({ system: 'fresh', user: 'fresh' });
    await settle();
    expect(state.context).toEqual({ system: 'fresh', user: 'fresh' });
    expect(state.loading).toBe(false);
  });

  it('selection change rejects completion even before the next update effect', async () => {
    const { state, pending, lifecycle } = fixture();
    lifecycle.update('A', 'notes', 1);
    state.selected = 'B';
    pending(0).resolve({ system: 'old', user: 'old' });
    await settle();
    expect(state.context).toBeNull();
    lifecycle.update('B', '', 1);
    expect(state.loading).toBe(false);
  });

  it('reuses an unchanged context but refreshes a revision and clears current failures', async () => {
    const { state, pending, lifecycle, fetchContext } = fixture();
    lifecycle.update('A', 'notes', 1);
    pending(0).resolve({ system: 'first', user: 'first' });
    await settle();
    lifecycle.update('A', 'notes', 1);
    expect(fetchContext).toHaveBeenCalledTimes(1);
    lifecycle.update('A', 'notes', 2);
    expect(state.context).toBeNull();
    expect(state.loading).toBe(true);
    pending(1).reject(new Error('current failure'));
    await settle();
    expect(state.context).toBeNull();
    expect(state.loading).toBe(false);
  });
});
