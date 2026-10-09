<script lang="ts">
  import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
  import {
    propagationProposalProjectionState,
    refreshPropagationProposalListProjection,
    getPropagationProposalSessionEpoch,
  } from '$lib/stores/propagationProposalProjection.svelte.js';
  import { createScriptFactReview } from './scriptFactReview.svelte.js';
  let { documentId, segment }: { documentId: string; segment: ScriptSegmentProjection } = $props();
  let selected = $state('');
  const evidence = $derived(segment.impact?.fact_edit);
  const fact = $derived(evidence?.facts.find((f) => f.dependency_id === selected));
  const owner = $derived(
    JSON.stringify([
      getPropagationProposalSessionEpoch(),
      documentId,
      segment.segment.id,
      evidence?.revision_event_id,
      evidence?.generation_event_id,
      selected,
      fact?.revision_event_id,
    ]),
  );
  const review = $derived.by(() => {
    void getPropagationProposalSessionEpoch();
    return createScriptFactReview(() => owner);
  });
  const proposals = $derived(
    (propagationProposalProjectionState.projection?.payload.proposals ?? []).filter(
      (p) =>
        p.script_fact_binding?.request.document_id === documentId &&
        p.script_fact_binding.request.segment_id === segment.segment.id,
    ),
  );
  const pending = $derived(review.state.pending || propagationProposalProjectionState.pending);
  $effect(() => {
    if (
      !propagationProposalProjectionState.projection &&
      !propagationProposalProjectionState.pending &&
      !propagationProposalProjectionState.error
    )
      void refreshPropagationProposalListProjection().catch(() => {});
  });
  $effect(() => {
    void owner;
    review.observe();
  });
  async function analyze() {
    if (!evidence || !fact) return;
    await review.analyze({
      document_id: documentId,
      segment_id: segment.segment.id,
      block_id: evidence.block_id,
      expected_block_revision_event_id: evidence.revision_event_id,
      expected_field_revision_event_id: fact.revision_event_id,
      generation_event_id: evidence.generation_event_id,
      dependency_id: fact.dependency_id,
    });
  }
</script>

{#if evidence || proposals.length}
  <section class="fact-review" aria-label="Saved edit and Bible fact reconciliation">
    {#if evidence}
      <p>
        Analyze saved screenplay against one Bible fact this scene consumed. Drafts remain separate.
      </p>
      <label
        >Consumed baseline fact
        <select bind:value={selected} disabled={pending}>
          <option value="">Choose a consumed fact</option>
          {#each evidence.facts as item (item.dependency_id)}<option value={item.dependency_id}
              >{item.node_id} · {item.part_key}.{item.field_key}</option
            >{/each}
        </select>
      </label>
      {#if fact}<p>Current fact</p>
        <pre>{fact.text}</pre>{/if}
      <details>
        <summary>Saved screenplay edit used for analysis</summary>
        <p>Before · revision {evidence.before_revision_event_id}</p>
        <pre aria-label="Previous saved screenplay">{evidence.before_text}</pre>
        <p>After · revision {evidence.revision_event_id}</p>
        <pre aria-label="Current saved screenplay">{evidence.text}</pre>
      </details>
      <button type="button" onclick={analyze} disabled={pending || !fact}
        >{review.state.pending
          ? 'Analyzing saved edit…'
          : review.state.retry
            ? 'Retry same analysis'
            : 'Analyze saved edit'}</button
      >
    {/if}
    {#if review.state.error}<p role="alert">{review.state.error}</p>{/if}
    {#each proposals as proposal (proposal.id)}
      {@const binding = proposal.script_fact_binding!}
      {@const used = binding.edit.facts[0]!}
      <article aria-label="Bible fact proposal">
        <strong>{used.node_id} · {used.part_key}.{used.field_key} · {proposal.status}</strong>
        <p>Before · fact revision {used.revision_event_id}</p>
        <pre aria-label="Fact before analysis">{used.text}</pre>
        <p>Proposed fact</p>
        <pre aria-label="Proposed Bible fact">{proposal.proposed_value?.type === 'text'
            ? proposal.proposed_value.value
            : ''}</pre>
        <p>{proposal.rationale}</p>
        <details>
          <summary>Saved edit provenance</summary>
          <p>Before · revision {binding.edit.before_revision_event_id}</p>
          <pre>{binding.edit.before_text}</pre>
          <p>After · revision {binding.edit.revision_event_id}</p>
          <pre>{binding.edit.text}</pre>
          <p>Consumed fact · revision {used.consumed_revision_event_id}</p>
          <pre>{used.consumed_text}</pre>
        </details>
        {#if proposal.status === 'pending'}
          <button type="button" onclick={() => review.decide(proposal.id, true)} disabled={pending}
            >Accept fact update</button
          >
          <button type="button" onclick={() => review.decide(proposal.id, false)} disabled={pending}
            >Reject fact update</button
          >
        {/if}
      </article>
    {/each}
  </section>
{/if}

<style>
  .fact-review {
    padding: 8px;
    margin: 8px 0;
    border: 1px solid var(--color-border-subtle);
    font-size: 0.8rem;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  select {
    max-width: 100%;
  }
  article {
    margin-top: 10px;
    border-top: 1px solid var(--color-border-subtle);
    padding-top: 8px;
  }
</style>
