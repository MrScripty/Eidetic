<script lang="ts">
  import type { ChildPlan } from '$lib/childPlanningTypes.js';
  let {
    recoverable = false,
    savedPlans = null,
    reading = false,
    onrecover = () => {},
    onreviewsaved = () => {},
    plan,
    busy,
    uncertain,
    error,
    onaccept,
    onclose,
  }: {
    recoverable?: boolean;
    savedPlans?: ChildPlan[] | null;
    reading?: boolean;
    onrecover?: () => void;
    onreviewsaved?: (planId: string) => void;
    plan: ChildPlan | null;
    busy: boolean;
    uncertain: boolean;
    error: string | null;
    onaccept: () => void;
    onclose: () => void;
  } = $props();
</script>

{#if recoverable}
  <button type="button" disabled={busy || uncertain || plan != null} onclick={onrecover}>
    {reading ? 'Loading saved timeline plans…' : 'Review saved timeline plans'}
  </button>
{/if}
{#if savedPlans != null && !plan}
  <section aria-label="Saved timeline plans" class="child-plan-review">
    <h3>Saved timeline plans</h3>
    <p>Choose a pending plan to review its original proposal and screenplay evidence.</p>
    {#if savedPlans.length === 0}
      <p>No pending timeline plans for this clip.</p>
    {:else}
      <ol>
        {#each savedPlans as saved, index (saved.id)}
          <li>
            {#each saved.children as child}
              <strong>{child.name}</strong>
              <p>{child.outline}</p>
            {/each}
            <button
              type="button"
              disabled={busy || uncertain}
              onclick={() => onreviewsaved(saved.id)}>Review plan {index + 1}</button
            >
          </li>
        {/each}
      </ol>
    {/if}
    <button type="button" disabled={busy || uncertain} onclick={onclose}>Close saved plans</button>
  </section>
{/if}
{#if error}<p role="alert">{error}</p>{/if}
{#if plan}
  <section aria-label="Review proposed timeline children" class="child-plan-review">
    <h3>Proposed timeline children</h3>
    <p>Accept replaces the selected clip's child clips. Saved screenplay text stays unchanged.</p>
    <ol>
      {#each plan.children as child}
        <li>
          <strong>{child.name}</strong>
          <p>{child.outline}</p>
          <p>Duration weight: {child.weight}</p>
          {#if child.characters?.length}<p>Characters: {child.characters.join(', ')}</p>{/if}
          {#if child.location}<p>Location: {child.location}</p>{/if}
          {#if child.props?.length}<p>Props: {child.props.join(', ')}</p>{/if}
        </li>
      {/each}
    </ol>
    <details>
      <summary>Saved screenplay used for this plan</summary>
      {#if plan.script_context == null}
        <p>Screenplay evidence was not recorded for this plan.</p>
      {:else if plan.script_context.length === 0}
        <p>No saved screenplay was selected for this plan.</p>
      {:else}
        {#each plan.script_context as input}
          <pre
            data-source-block={input.block_id}
            data-source-revision={input.revision_event_id}>{input.text}</pre>
        {/each}
      {/if}
    </details>
    <button type="button" disabled={busy} onclick={onaccept}
      >{busy ? 'Accepting…' : uncertain ? 'Retry acceptance' : 'Accept timeline plan'}</button
    >
    <button type="button" disabled={busy || uncertain} onclick={onclose}>Close preview</button>
  </section>
{/if}

<style>
  .child-plan-review {
    border: 1px solid var(--color-border);
    padding: 12px;
    margin-bottom: 12px;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
