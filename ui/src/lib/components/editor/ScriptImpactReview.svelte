<script lang="ts">
  import ScriptArcEvidence from './ScriptArcEvidence.svelte';
  import ScriptTimelineNotesEvidence from './ScriptTimelineNotesEvidence.svelte';
  import ScriptTimelineTitleEvidence from './ScriptTimelineTitleEvidence.svelte';
  import ScriptBlockTypeEvidence from './ScriptBlockTypeEvidence.svelte';
  import ScriptRecallFacts from './ScriptRecallFacts.svelte';
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
  let recallFacts: ScriptRecallFacts | undefined = $state();
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
      const recallSelection = recallFacts?.getSelection() ?? null;
      await applyRequestScriptImpactProposalCommand({
        proposal_id: `script.review.${crypto.randomUUID()}`,
        document_id: documentId,
        segment_id: segment.segment.id,
        block_id: block.block.id,
        expected_block_revision_event_id: block.revision_event_id,
        generation_event_id: segment.impact.generation_event_id,
        dependency_id: cause.dependency_id,
        story_time_ms: null,
        ...(recallSelection ? { recall_selection: recallSelection } : {}),
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
      <ScriptRecallFacts
        bind:this={recallFacts}
        scope={JSON.stringify([
          documentId,
          segment.segment.id,
          block?.revision_event_id,
          segment.impact.generation_event_id,
        ])}
        disabled={pending}
      />
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
        <ScriptTimelineTitleEvidence
          previous={proposal.script_review_binding?.timeline_title_previous}
          current={proposal.script_review_binding?.timeline_title_current}
          absent={proposal.script_review_binding?.timeline_title_absence_revisions}
        />
        <ScriptBlockTypeEvidence
          previous={proposal.script_review_binding?.script_previous_inputs}
          current={proposal.script_review_binding?.script_inputs ?? []}
        />
        <ScriptTimelineNotesEvidence
          previous={proposal.script_review_binding?.timeline_notes_previous}
          current={proposal.script_review_binding?.timeline_notes_current}
        />
        {#each proposal.script_review_binding?.ancestor_notes_previous ?? [] as previous (previous.node_id)}
          <ScriptTimelineNotesEvidence
            ancestor={true}
            {previous}
            current={proposal.script_review_binding?.ancestor_notes_current?.find(
              (input) => input.node_id === previous.node_id,
            )}
            absentRevision={proposal.script_review_binding?.ancestor_notes_absence_revisions?.find(
              ([node]) => node === previous.node_id,
            )?.[1]}
          />
        {/each}
        <ScriptArcEvidence
          applicabilityPrevious={proposal.script_review_binding
            ?.arc_description_applicability_previous}
          applicabilityCurrent={proposal.script_review_binding
            ?.arc_description_applicability_current}
          previous={proposal.script_review_binding?.arc_previous_inputs}
          current={proposal.script_review_binding?.arc_inputs}
          absent={proposal.script_review_binding?.arc_absence_revisions}
        />
        <pre aria-label="Proposed text">{proposal.proposed_text}</pre>
        {#if proposal.script_review_binding?.request.recall_selection?.facts.length}
          <details>
            <summary>Author-selected recalled facts used for this preview</summary>
            <p>Unspecified fictional time · baseline facts · connecting paths are untimed.</p>
            {#each proposal.script_review_binding.request.recall_selection.facts as fact (fact.field_id)}
              {@const input = proposal.script_review_binding.bible_inputs?.find(
                (input) => input.field_id === fact.field_id,
              )}
              <p>
                {fact.node_id} · {fact.part_key}.{fact.field_key}: {JSON.stringify(
                  input?.value.value,
                )} · {fact.field_id} · revision {fact.revision_event_id}
              </p>
            {/each}
          </details>
        {/if}
        {#if proposal.script_review_binding?.bible_node_name_inputs?.length}
          <details aria-label="Recorded screenplay name evidence">
            <summary>Names used for this preview</summary>
            <ul>
              {#each proposal.script_review_binding.bible_node_name_inputs as input (input.node_id)}
                <li>
                  {input.name} ({input.node_id}) <small>Revision {input.revision_event_id}</small>
                </li>
              {/each}
            </ul>
          </details>
        {/if}
        {#if proposal.script_review_binding?.bible_relationship_inputs?.length}
          <details aria-label="Recorded screenplay relationship evidence">
            <summary>Relationships used for this preview</summary>
            <ul>
              {#each proposal.script_review_binding.bible_relationship_inputs as input (input.edge.edge_id)}
                <li>
                  {input.edge.from_node_id}
                  {input.edge.directed ? '→' : '↔'}
                  {input.edge.to_node_id}:
                  {input.edge.label}
                  ({typeof input.edge.edge_kind === 'string'
                    ? input.edge.edge_kind.replaceAll('_', ' ')
                    : input.edge.edge_kind.custom})
                  <small>Revision {input.revision_event_id}</small>
                </li>
              {/each}
            </ul>
          </details>
        {/if}
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
