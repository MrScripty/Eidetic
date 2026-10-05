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
  const apply = vi.fn().mockResolvedValue({});
  const accepted = vi.fn().mockResolvedValue(undefined);
  const review = createChildPlanReview({
    owner: () => selected,
    mounted: () => mounted,
    generate,
    apply,
    accepted,
  });
  return {
    review,
    generate,
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

it('retains the proposal and visible error after definite native context refusal', async () => {
  const f = fixture();
  await f.review.generate();
  f.apply.mockRejectedValue(
    new Error(
      'Child plan story context changed; generate and review a fresh plan before accepting',
    ),
  );
  await f.review.accept();
  expect(f.review.state.plan).toEqual(plan);
  expect(f.review.state.error).toContain('fresh plan');
  expect(f.review.state.uncertain).toBe(false);
  f.review.close();
  expect(f.review.state.plan).toBeNull();
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
