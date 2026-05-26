<script lang="ts">
  import type { FieldDelta, ObjectRevision } from '$lib/changeReviewTypes.js';
  import type { AffectProposal, AffectTarget } from '$lib/affectTypes.js';
  import type { FieldValue } from '$lib/projectionTypes.js';
  import {
    affectProposalProjectionState,
    applyAcceptAffectProposalCommand,
    applyRejectAffectProposalCommand,
    refreshAffectProposalListProjection,
  } from '$lib/stores/affectProposalProjection.svelte.js';
  import {
    changeReviewProjectionState,
    refreshChangeReviewProjection,
  } from '$lib/stores/changeReviewProjection.svelte.js';
  import './changeReviewPanel.css';

  let changes = $derived(changeReviewProjectionState.projection?.payload.changes ?? []);
  let affectProposals = $derived(affectProposalProjectionState.projection?.payload.proposals ?? []);

  $effect(() => {
    if (!changeReviewProjectionState.projection && !changeReviewProjectionState.pending) {
      refreshChangeReviewProjection().catch(() => {});
    }
    if (!affectProposalProjectionState.projection && !affectProposalProjectionState.pending) {
      refreshAffectProposalListProjection().catch(() => {});
    }
  });

  function formatDate(createdAtMs: number): string {
    return new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
    }).format(new Date(createdAtMs));
  }

  function formatKind(value: string): string {
    return value
      .split('_')
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(' ');
  }

  function revisionLabel(revision: ObjectRevision): string {
    return `${formatKind(revision.object_kind)} ${revision.operation}`;
  }

  function fieldValueLabel(value: FieldValue | null | undefined): string {
    if (!value) return 'empty';
    switch (value.type) {
      case 'text':
      case 'asset_ref':
        return value.value || 'empty';
      case 'integer':
      case 'number':
        return String(value.value);
      case 'bool':
        return value.value ? 'true' : 'false';
      case 'object_ref':
        return `${formatKind(value.value.kind)} ${value.value.id}`;
    }
  }

  function deltaChanged(delta: FieldDelta): boolean {
    return fieldValueLabel(delta.old_value) !== fieldValueLabel(delta.new_value);
  }

  function affectTargetLabel(target: AffectTarget): string {
    switch (target.type) {
      case 'project':
        return 'Project';
      case 'timeline_node':
        return `Timeline ${target.node_id}`;
      case 'script_segment':
        return `Script ${target.segment_id}`;
      case 'bible_node':
        return `Bible ${target.node_id}`;
      case 'bible_snapshot':
        return `Snapshot ${target.snapshot_id}`;
    }
  }

  function affectValueLabel(proposal: AffectProposal): string {
    const value = proposal.proposed_value;
    const moods = value.mood_labels.join(', ');
    return `${affectTargetLabel(value.target)}: ${moods} | V ${value.valence} / A ${value.arousal}`;
  }

  async function refreshAll(): Promise<void> {
    await Promise.all([refreshChangeReviewProjection(), refreshAffectProposalListProjection()]);
  }
</script>

<section class="change-review-panel">
  <div class="panel-header">
    <h3>Change Review</h3>
    <button
      class="refresh-btn"
      onclick={() => refreshAll().catch(() => {})}
      disabled={changeReviewProjectionState.pending || affectProposalProjectionState.pending}
    >
      {changeReviewProjectionState.pending || affectProposalProjectionState.pending
        ? '...'
        : 'Refresh'}
    </button>
  </div>

  {#if changeReviewProjectionState.error}
    <p class="status status-error">{changeReviewProjectionState.error}</p>
  {/if}
  {#if affectProposalProjectionState.error}
    <p class="status status-error">{affectProposalProjectionState.error}</p>
  {/if}

  {#if affectProposals.length > 0}
    <section class="affect-proposals" aria-label="Affect proposals">
      <h4>Affect Proposals</h4>
      <div class="change-list">
        {#each affectProposals as proposal (proposal.id)}
          <article class="change-card affect-proposal-card">
            <header class="change-header">
              <div class="change-title">
                <span class="change-kind">{formatKind(proposal.status)}</span>
                <span class="change-summary">{proposal.summary}</span>
              </div>
              <time datetime={new Date(proposal.created_at_ms).toISOString()}>
                {formatDate(proposal.created_at_ms)}
              </time>
            </header>
            <p class="affect-value">{affectValueLabel(proposal)}</p>
            {#if proposal.rationale}
              <p class="affect-rationale">{proposal.rationale}</p>
            {/if}
            {#if proposal.status === 'pending'}
              <div class="proposal-actions">
                <button
                  type="button"
                  onclick={() =>
                    applyRejectAffectProposalCommand({ proposal_id: proposal.id }).catch(() => {})}
                  disabled={affectProposalProjectionState.pending}
                >
                  Reject
                </button>
                <button
                  type="button"
                  class="primary-action"
                  onclick={() =>
                    applyAcceptAffectProposalCommand({ proposal_id: proposal.id }).catch(() => {})}
                  disabled={affectProposalProjectionState.pending}
                >
                  Accept
                </button>
              </div>
            {/if}
          </article>
        {/each}
      </div>
    </section>
  {/if}

  {#if changes.length === 0 && !changeReviewProjectionState.pending}
    <p class="status">No recorded changes.</p>
  {:else}
    <div class="change-list">
      {#each changes as change (change.event.id)}
        <article class="change-card">
          <header class="change-header">
            <div class="change-title">
              <span class="change-kind">{formatKind(change.event.kind)}</span>
              <span class="change-summary">{change.event.summary}</span>
            </div>
            <time datetime={new Date(change.event.created_at_ms).toISOString()}>
              {formatDate(change.event.created_at_ms)}
            </time>
          </header>

          <div class="revision-list">
            {#each change.revisions as revision (revision.id)}
              <details class="revision-item">
                <summary>
                  <span>{revisionLabel(revision)}</span>
                  <code>{revision.object_id}</code>
                </summary>
                {#if revision.fields.length === 0}
                  <p class="field-empty">No field deltas recorded.</p>
                {:else}
                  <dl class="field-list">
                    {#each revision.fields as field (`${revision.id}-${field.field_key}`)}
                      <div class:unchanged={!deltaChanged(field)} class="field-row">
                        <dt>{field.field_key}</dt>
                        <dd>
                          <span>{fieldValueLabel(field.old_value)}</span>
                          <span class="arrow">-></span>
                          <span>{fieldValueLabel(field.new_value)}</span>
                        </dd>
                      </div>
                    {/each}
                  </dl>
                {/if}
              </details>
            {/each}
          </div>
        </article>
      {/each}
    </div>
  {/if}
</section>
