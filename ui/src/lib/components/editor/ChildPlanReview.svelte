<script lang="ts">
  import type { ChildPlan } from '$lib/childPlanningTypes.js';
  let {
    plan,
    busy,
    uncertain,
    error,
    onaccept,
    onclose,
  }: {
    plan: ChildPlan | null;
    busy: boolean;
    uncertain: boolean;
    error: string | null;
    onaccept: () => void;
    onclose: () => void;
  } = $props();
</script>

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
