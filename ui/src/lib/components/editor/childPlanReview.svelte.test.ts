import { expect, it, vi } from 'vitest';
import type { ChildPlan } from '$lib/childPlanningTypes.js';
import { createChildPlanReview } from './childPlanReview.svelte.js';

const plan: ChildPlan = {
  id: 'plan',
  parent_node_id: 'scene',
  target_child_level: 'Beat',
  children: [{ name: 'Departure', outline: 'Exact proposed outline', weight: 1, beat_type: null }],
  script_context: [
    {
      document_id: 'main',
      segment_id: 'segment',
      block_id: 'manual',
      source_node_id: 'scene',
      revision_event_id: 'write',
      segment_revision_event_id: 'placement',
      start_ms: 0,
      end_ms: 1000,
      text: 'Exact human text\n\n',
    },
  ],
};

function fixture() {
  let selected = { nodeId: 'scene', session: 0 };
  let mounted = true;
  const generate = vi.fn().mockResolvedValue(plan);
  const load = vi.fn().mockResolvedValue({
    version: 1,
    payload: { plans: [{ plan, status: 'pending', created_at_ms: 0 }] },
  });
  const apply = vi.fn().mockResolvedValue({});
  const accepted = vi.fn().mockResolvedValue(undefined);
  const review = createChildPlanReview({
    owner: () => selected,
    mounted: () => mounted,
    generate,
    load,
    apply,
    accepted,
  });
  return {
    review,
    generate,
    load,
    apply,
    accepted,
    select(nodeId: string, session = 0) {
      selected = { nodeId, session };
      review.syncOwner();
    },
    unmount() {
      mounted = false;
    },
  };
}

it('generation presents a pending proposal and exact saved evidence without applying', async () => {
  const f = fixture();
  await f.review.generate();
  expect(f.review.state.plan).toEqual(plan);
  expect(f.apply).not.toHaveBeenCalled();
  expect(f.review.state.plan?.script_context?.[0]?.text).toBe('Exact human text\n\n');
});

it('only explicit acceptance submits the existing timeline command and then clears preview', async () => {
  const f = fixture();
  await f.review.generate();
  await f.review.accept();
  expect(f.apply).toHaveBeenCalledOnce();
  expect(f.apply.mock.calls[0]?.[0]).toEqual({
    parent_id: 'scene',
    child_plan_id: 'plan',
    children: [
      {
        name: 'Departure',
        outline: 'Exact proposed outline',
        weight: 1,
        beat_type: null,
        characters: [],
        props: [],
        location: null,
      },
    ],
  });
  expect(f.accepted).toHaveBeenCalledWith('scene');
  expect(f.review.state.plan).toBeNull();
});

it.each(['selection', 'selection-aba', 'session', 'unmount'])(
  'ignores a delayed generated proposal after %s',
  async (change) => {
    const f = fixture();
    let respond!: (p: ChildPlan) => void;
    f.generate.mockImplementation(
      () =>
        new Promise((resolve) => {
          respond = resolve;
        }),
    );
    const pending = f.review.generate();
    if (change === 'selection') f.select('other');
    if (change === 'selection-aba') {
      f.select('other');
      f.select('scene');
    }
    if (change === 'session') f.select('scene', 1);
    if (change === 'unmount') f.unmount();
    respond(plan);
    await pending;
    expect(f.review.state.plan).toBeNull();
    expect(f.apply).not.toHaveBeenCalled();
  },
);

it.each([
  'Child plan story context changed; generate and review a fresh plan before accepting',
  'Accepted children differ from the reviewed child plan',
])('allows Close and fresh generation after definite native refusal: %s', async (message) => {
  const f = fixture();
  await f.review.generate();
  f.apply.mockRejectedValue(new Error(message, { cause: { kind: 'conflict', message } }));
  await f.review.accept();
  expect(f.review.state.plan).toEqual(plan);
  expect(f.review.state.error).toBe(message);
  expect(f.review.state.uncertain).toBe(false);
  f.review.close();
  expect(f.review.state.plan).toBeNull();
  await f.review.generate();
  expect(f.generate).toHaveBeenCalledTimes(2);
  f.apply.mockResolvedValueOnce({});
  await f.review.accept();
  expect(f.apply.mock.calls[1]?.[1]).not.toBe(f.apply.mock.calls[0]?.[1]);
  expect(f.review.state.plan).toBeNull();
});

it.each([
  new Error('Accepted children differ from the reviewed child plan'),
  new Error('Child plan story context changed; generate and review a fresh plan before accepting'),
  new Error('Accepted children differ from the reviewed child plan', {
    cause: { kind: 'internal' },
  }),
  new Error('Different validation error', { cause: { kind: 'conflict' } }),
])('keeps lookalike or unproven completion errors uncertain: %s', async (error) => {
  const f = fixture();
  await f.review.generate();
  f.apply.mockRejectedValueOnce(error);
  await f.review.accept();
  expect(f.review.state.uncertain).toBe(true);
  f.review.close();
  await f.review.generate();
  expect(f.generate).toHaveBeenCalledOnce();
  await f.review.accept();
  expect(f.apply.mock.calls[1]).toEqual(f.apply.mock.calls[0]);
});

it('unlocks fresh generation after an exact uncertain retry receives a definite refusal', async () => {
  const f = fixture();
  const message = 'Accepted children differ from the reviewed child plan';
  await f.review.generate();
  f.apply
    .mockRejectedValueOnce(new Error('Acknowledgement lost'))
    .mockRejectedValueOnce(new Error(message, { cause: { kind: 'conflict', message } }));
  await f.review.accept();
  await f.review.accept();
  expect(f.apply.mock.calls[1]).toEqual(f.apply.mock.calls[0]);
  expect(f.review.state.uncertain).toBe(false);
  await f.review.generate();
  f.apply.mockResolvedValueOnce({});
  await f.review.accept();
  expect(f.apply.mock.calls[2]?.[1]).not.toBe(f.apply.mock.calls[0]?.[1]);
});

it('retries an uncertain acceptance using the exact payload and command identity', async () => {
  const f = fixture();
  await f.review.generate();
  f.apply.mockRejectedValueOnce(new Error('Acknowledgement lost'));
  await f.review.accept();
  expect(f.review.state.uncertain).toBe(true);
  f.review.close();
  await f.review.generate();
  expect(f.generate).toHaveBeenCalledOnce();
  expect(f.review.state.plan).not.toBeNull();
  await f.review.accept();
  expect(f.apply.mock.calls[1]).toEqual(f.apply.mock.calls[0]);
  expect(f.review.state.plan).toBeNull();
});

it('does not run selection follow-up after delayed acceptance in another editor lifetime', async () => {
  const f = fixture();
  await f.review.generate();
  let respond!: () => void;
  f.apply.mockImplementation(
    () =>
      new Promise<void>((resolve) => {
        respond = resolve;
      }),
  );
  const pending = f.review.accept();
  f.select('other', 1);
  respond();
  await pending;
  expect(f.accepted).not.toHaveBeenCalled();
  expect(f.review.state.plan).toBeNull();
});

it('keeps a successful acceptance acknowledged when projection refresh fails', async () => {
  const f = fixture();
  await f.review.generate();
  f.accepted.mockRejectedValueOnce(new Error('Refresh failed'));
  await f.review.accept();
  expect(f.review.state.plan).toBeNull();
  expect(f.review.state.uncertain).toBe(false);
  expect(f.review.state.error).toBe('Refresh failed');
});

it('lists all matching pending plans and reopens exact material only after choosing', async () => {
  const f = fixture();
  const second = { ...plan, id: 'second' };
  const records = [
    { plan, status: 'pending', created_at_ms: 0 },
    { plan: { ...plan, id: 'applied' }, status: 'applied', created_at_ms: 99 },
    { plan: { ...plan, id: 'rejected' }, status: 'rejected', created_at_ms: 99 },
    {
      plan: { ...plan, id: 'other', parent_node_id: 'other' },
      status: 'pending',
      created_at_ms: 99,
    },
    { plan: second, status: 'pending', created_at_ms: 0 },
  ];
  f.load.mockResolvedValue({ version: 5, payload: { plans: records } });
  await f.review.recover();
  expect(f.review.state.savedPlans).toEqual([plan, second]);
  expect(f.review.state.plan).toBeNull();
  expect(f.generate).not.toHaveBeenCalled();
  expect(f.apply).not.toHaveBeenCalled();
  records[0]!.plan = { ...plan, children: [] };
  f.review.reviewSaved('other');
  expect(f.review.state.plan).toBeNull();
  f.review.reviewSaved('plan');
  expect(f.review.state.plan).toEqual(plan);
  expect(f.review.state.savedPlans).toBeNull();
  expect(f.review.state.plan?.script_context?.[0]?.text).toBe('Exact human text\n\n');
  expect(f.apply).not.toHaveBeenCalled();
  await f.review.accept();
  expect(f.apply.mock.calls[0]?.[0].child_plan_id).toBe('plan');
  expect(f.apply.mock.calls[0]?.[0].children[0]?.outline).toBe(plan.children[0]?.outline);
});

it('retries read failures and distinguishes a successful empty pending list', async () => {
  const f = fixture();
  f.load.mockRejectedValueOnce(new Error('Read unavailable'));
  await f.review.recover();
  expect(f.review.state.error).toBe('Read unavailable');
  expect(f.review.state.savedPlans).toBeNull();
  expect(f.review.state.busy).toBe(false);
  expect(f.review.state.reading).toBe(false);
  f.load.mockResolvedValueOnce({ version: 2, payload: { plans: [] } });
  await f.review.recover();
  expect(f.review.state.error).toBeNull();
  expect(f.review.state.savedPlans).toEqual([]);
  expect(f.apply).not.toHaveBeenCalled();
});

it('reopens the same proposal after Close and selection away/back without generation', async () => {
  const f = fixture();
  await f.review.generate();
  f.review.close();
  f.select('other');
  f.select('scene');
  await f.review.recover();
  f.review.reviewSaved('plan');
  expect(f.review.state.plan).toEqual(plan);
  expect(f.generate).toHaveBeenCalledOnce();
  expect(f.apply).not.toHaveBeenCalled();
});

it.each(['selection', 'selection-aba', 'session', 'unmount'])(
  'ignores a delayed recovery read after %s',
  async (change) => {
    const f = fixture();
    let respond!: (value: unknown) => void;
    f.load.mockImplementation(
      () =>
        new Promise((resolve) => {
          respond = resolve;
        }),
    );
    const pending = f.review.recover();
    if (change === 'selection') f.select('other');
    if (change === 'selection-aba') {
      f.select('other');
      f.select('scene');
    }
    if (change === 'session') f.select('scene', 1);
    if (change === 'unmount') f.unmount();
    respond({ version: 1, payload: { plans: [{ plan, status: 'pending', created_at_ms: 0 }] } });
    await pending;
    expect(f.review.state.savedPlans).toBeNull();
    expect(f.review.state.plan).toBeNull();
    expect(f.apply).not.toHaveBeenCalled();
  },
);

it('obsolete read failure/finalizer cannot change another selection read', async () => {
  const f = fixture();
  let rejectOld!: (error: Error) => void;
  let resolveNew!: (value: unknown) => void;
  f.load
    .mockImplementationOnce(
      () =>
        new Promise((_resolve, reject) => {
          rejectOld = reject;
        }),
    )
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          resolveNew = resolve;
        }),
    );
  const old = f.review.recover();
  f.select('other');
  const fresh = f.review.recover();
  rejectOld(new Error('Obsolete read error'));
  await old;
  expect(f.review.state.error).toBeNull();
  expect(f.review.state.reading).toBe(true);
  expect(f.review.state.busy).toBe(true);
  resolveNew({ version: 2, payload: { plans: [] } });
  await fresh;
  expect(f.review.state.savedPlans).toEqual([]);
  expect(f.review.state.busy).toBe(false);
});

it('mutually excludes generation and reads', async () => {
  const f = fixture();
  let respond!: (value: unknown) => void;
  f.load.mockImplementation(
    () =>
      new Promise((resolve) => {
        respond = resolve;
      }),
  );
  const pending = f.review.recover();
  await f.review.generate();
  await f.review.recover();
  f.review.reviewSaved('plan');
  expect(f.generate).not.toHaveBeenCalled();
  expect(f.load).toHaveBeenCalledOnce();
  respond({ version: 1, payload: { plans: [] } });
  await pending;
  let finish!: (value: ChildPlan) => void;
  f.generate.mockImplementation(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  const generating = f.review.generate();
  await f.review.recover();
  expect(f.load).toHaveBeenCalledOnce();
  finish(plan);
  await generating;
  expect(f.review.state.savedPlans).toBeNull();
});

it('cannot replace an open preview or its uncertain command retry through recovery', async () => {
  const f = fixture();
  await f.review.generate();
  await f.review.recover();
  expect(f.load).not.toHaveBeenCalled();
  f.apply.mockRejectedValueOnce(new Error('Acknowledgement lost'));
  await f.review.accept();
  await f.review.recover();
  f.review.reviewSaved('replacement');
  expect(f.review.state.plan).toEqual(plan);
  expect(f.review.state.uncertain).toBe(true);
  expect(f.load).not.toHaveBeenCalled();
  await f.review.accept();
  expect(f.apply.mock.calls[1]).toEqual(f.apply.mock.calls[0]);
});

it('preserves recovered material and definite stale refusal for fresh review', async () => {
  const f = fixture();
  await f.review.recover();
  f.review.reviewSaved('plan');
  const message =
    'Child plan story context changed; generate and review a fresh plan before accepting';
  f.apply.mockRejectedValueOnce(new Error(message, { cause: { kind: 'conflict', message } }));
  await f.review.accept();
  expect(f.review.state.plan).toEqual(plan);
  expect(f.review.state.uncertain).toBe(false);
  expect(f.review.state.error).toBe(message);
  f.review.close();
  await f.review.recover();
  expect(f.review.state.savedPlans).toEqual([plan]);
  expect(f.apply).toHaveBeenCalledOnce();
});

it('clears saved choices on Close or selection change and ignores unmounted actions', async () => {
  const f = fixture();
  await f.review.recover();
  f.review.close();
  f.review.reviewSaved('plan');
  expect(f.review.state.plan).toBeNull();
  await f.review.recover();
  f.select('other');
  f.review.reviewSaved('plan');
  expect(f.review.state.plan).toBeNull();
  f.select('scene');
  await f.review.recover();
  f.unmount();
  f.review.reviewSaved('plan');
  await f.review.recover();
  expect(f.review.state.plan).toBeNull();
  expect(f.load).toHaveBeenCalledTimes(3);
});
