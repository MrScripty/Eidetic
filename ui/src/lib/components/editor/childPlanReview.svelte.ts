import type { ChildPlan } from '$lib/childPlanningTypes.js';
import type { ApplyTimelineChildrenCommand } from '$lib/timelineCommandTypes.js';

type Owner = { nodeId: string | null; session: number };

/** Transient review of the existing durable plan; generation never applies it. */
export function createChildPlanReview(options: {
  owner: () => Owner;
  mounted: () => boolean;
  generate: (parent: string) => Promise<ChildPlan>;
  apply: (payload: ApplyTimelineChildrenCommand, commandId: string) => Promise<unknown>;
  accepted: (parent: string) => Promise<void>;
}) {
  const state = $state({
    plan: null as ChildPlan | null,
    busy: false,
    error: null as string | null,
    uncertain: false,
  });
  let owner = options.owner();
  let sequence = 0;
  let submission: { payload: ApplyTimelineChildrenCommand; id: string } | null = null;
  function syncOwner() {
    const current = options.owner();
    if (current.nodeId === owner.nodeId && current.session === owner.session) return;
    owner = current;
    sequence += 1;
    state.plan = null;
    state.error = null;
    state.busy = false;
    state.uncertain = false;
    submission = null;
  }
  function current(token: number) {
    const selected = options.owner();
    return (
      options.mounted() &&
      token === sequence &&
      selected.nodeId === owner.nodeId &&
      selected.session === owner.session
    );
  }
  async function generate() {
    syncOwner();
    if (!owner.nodeId || state.busy || state.uncertain) return;
    const parent = owner.nodeId;
    const token = ++sequence;
    state.busy = true;
    state.error = null;
    try {
      const plan = await options.generate(parent);
      if (!current(token)) return;
      if (plan.parent_node_id !== parent)
        throw new Error('Generated child plan parent did not match the selected node');
      state.plan = plan;
      submission = null;
    } catch (error) {
      if (current(token)) state.error = error instanceof Error ? error.message : String(error);
    } finally {
      if (current(token)) state.busy = false;
    }
  }
  async function accept() {
    syncOwner();
    if (!state.plan || state.busy) return;
    const token = sequence;
    const parent = state.plan.parent_node_id;
    submission ??= {
      id: crypto.randomUUID(),
      payload: {
        parent_id: parent,
        child_plan_id: state.plan.id,
        children: state.plan.children.map((child) => ({
          name: child.name,
          outline: child.outline,
          weight: child.weight,
          beat_type: child.beat_type,
          characters: [...(child.characters ?? [])],
          props: [...(child.props ?? [])],
          location: child.location ?? null,
        })),
      },
    };
    state.busy = true;
    state.error = null;
    try {
      await options.apply(structuredClone(submission.payload), submission.id);
      if (!current(token)) return;
      state.plan = null;
      submission = null;
      state.uncertain = false;
      await options.accepted(parent).catch((error) => {
        if (current(token)) state.error = error instanceof Error ? error.message : String(error);
      });
    } catch (error) {
      if (!current(token)) return;
      state.error = error instanceof Error ? error.message : String(error);
      state.uncertain = !isChildPlanRefusal(error);
      if (!state.uncertain) submission = null;
    } finally {
      if (current(token)) state.busy = false;
    }
  }
  function close() {
    if (state.busy || state.uncertain) return;
    sequence += 1;
    state.plan = null;
    state.error = null;
    submission = null;
  }
  return { state, syncOwner, generate, accept, close };
}

function isChildPlanRefusal(failure: unknown): boolean {
  if (!(failure instanceof Error)) return false;
  const native = failure.cause;
  if (typeof native !== 'object' || native === null || !('kind' in native)) return false;
  // These exact native validation failures roll back the writer transaction.
  // Replay precedes validation. Error text or broad kinds alone cannot prove
  // that completion was refused rather than its acknowledgement being lost.
  return (
    native.kind === 'conflict' &&
    [
      'Child plan story context changed; generate and review a fresh plan before accepting',
      'Accepted children differ from the reviewed child plan',
    ].includes(failure.message)
  );
}
