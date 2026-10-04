<script lang="ts">
  import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
  import {
    propagationProposalProjectionState,
    refreshPropagationProposalListProjection,
    applyRequestScriptImpactProposalCommand,
    applyAcceptPropagationProposalCommand,
    applyRejectPropagationProposalCommand,
  } from '$lib/stores/propagationProposalProjection.svelte.js';
  import { refreshScriptDocumentProjection } from '$lib/stores/scriptDocumentProjection.svelte.js';
  import { scriptImpactCauseLabel } from './scriptImpactNotice.js';
  let { documentId, segment }: { documentId: string; segment: ScriptSegmentProjection } = $props();
  let selectedCause = $state('');
  let error = $state('');
  let block = $derived(
    segment.blocks.find((item) => item.block.id === segment.impact?.output_block_id),
  );
  let cause = $derived(
    segment.impact?.causes.find((item) => item.dependency_id === selectedCause) ??
      segment.impact?.causes[0],
  );
  let pending = $derived(propagationProposalProjectionState.pending);
  let proposals = $derived(
    (propagationProposalProjectionState.projection?.payload.proposals ?? []).filter(
      (proposal) =>
        proposal.script_review_binding?.request.document_id === documentId &&
        proposal.script_review_binding.request.segment_id === segment.segment.id,
    ),
  );

  $effect(() => {
    if (
      !propagationProposalProjectionState.projection &&
      !pending &&
      !propagationProposalProjectionState.error
    ) {
      void refreshPropagationProposalListProjection().catch(() => {});
    }
  });

  async function preview() {
    if (!block?.revision_event_id || !cause || !segment.impact) return;
    error = '';
    try {
      await applyRequestScriptImpactProposalCommand({
        proposal_id: `script.review.${crypto.randomUUID()}`,
        document_id: documentId,
        segment_id: segment.segment.id,
        block_id: block.block.id,
        expected_block_revision_event_id: block.revision_event_id,
        generation_event_id: segment.impact.generation_event_id,
        dependency_id: cause.dependency_id,
        story_time_ms: null,
      });
    } catch (failure) {
      error = failure instanceof Error ? failure.message : 'Preview failed';
    }
  }

  async function decide(proposalId: string, accept: boolean) {
    error = '';
    try {
      if (accept) await applyAcceptPropagationProposalCommand({ proposal_id: proposalId });
      else await applyRejectPropagationProposalCommand({ proposal_id: proposalId });
      await refreshScriptDocumentProjection({ document_id: documentId });
    } catch (failure) {
      error = failure instanceof Error ? failure.message : 'Review failed';
    }
  }
</script>

{#if segment.impact?.needs_review || proposals.length > 0}
  <section class="review" aria-label="Screenplay update proposals">
    {#if segment.impact?.needs_review}
      <p>Preview an update to the affected generated block.</p>
      <label
        >Input change
        <select bind:value={selectedCause} disabled={pending}>
          <option value=""
            >{segment.impact.causes[0]
              ? scriptImpactCauseLabel(segment.impact.causes[0])
              : 'Input change'}</option
          >
          {#each segment.impact.causes.slice(1) as item (item.dependency_id)}
            <option value={item.dependency_id}
              >{scriptImpactCauseLabel(item)} {item.input_excerpt?.slice(0, 60) ?? ''}</option
            >
          {/each}
        </select>
      </label>
      <button
        type="button"
        onclick={preview}
        disabled={pending || !block?.revision_event_id || !cause}>Preview update</button
      >
    {/if}
    {#if error}<p role="alert">{error}</p>{/if}
    {#if propagationProposalProjectionState.error && !error}<p role="alert">
        {propagationProposalProjectionState.error}
      </p>{/if}
    {#each proposals as proposal (proposal.id)}
      <article>
        <strong
          >{proposal.status === 'pending'
            ? 'Proposed screenplay update'
            : `Update ${proposal.status}`}</strong
        >
        {#if proposal.status === 'pending'}
          <details>
            <summary>Current text</summary>
            <pre>{segment.blocks.find(
                (item) =>
                  proposal.target.kind === 'script_block' &&
                  item.block.id === proposal.target.block_id,
              )?.block.text ?? 'Block no longer available'}</pre>
          </details>
        {/if}
        <pre aria-label="Proposed text">{proposal.proposed_text}</pre>
        {#if proposal.status === 'pending'}
          <button type="button" onclick={() => decide(proposal.id, false)} disabled={pending}
            >Reject</button
          >
          <button type="button" onclick={() => decide(proposal.id, true)} disabled={pending}
            >Accept update</button
          >
        {/if}
      </article>
    {/each}
  </section>
{/if}

<style>
  .review {
    break-inside: avoid;
    margin-bottom: 12px;
    padding: 8px;
    border: 1px solid var(--color-border-subtle);
    font-size: 0.8rem;
  }
  label {
    display: block;
    margin-bottom: 6px;
  }
  select {
    display: block;
    width: 100%;
  }
  article {
    margin-top: 8px;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  button {
    margin-right: 6px;
  }
</style>
