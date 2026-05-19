<script lang="ts">
  import type { FieldDelta, ObjectRevision } from '$lib/changeReviewTypes.js';
  import type { FieldValue } from '$lib/projectionTypes.js';
  import {
    changeReviewProjectionState,
    refreshChangeReviewProjection,
  } from '$lib/stores/changeReviewProjection.svelte.js';
  import './changeReviewPanel.css';

  let changes = $derived(changeReviewProjectionState.projection?.payload.changes ?? []);

  $effect(() => {
    if (!changeReviewProjectionState.projection && !changeReviewProjectionState.pending) {
      refreshChangeReviewProjection().catch(() => {});
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
</script>

<section class="change-review-panel">
  <div class="panel-header">
    <h3>Change Review</h3>
    <button
      class="refresh-btn"
      onclick={() => refreshChangeReviewProjection().catch(() => {})}
      disabled={changeReviewProjectionState.pending}
    >
      {changeReviewProjectionState.pending ? '...' : 'Refresh'}
    </button>
  </div>

  {#if changeReviewProjectionState.error}
    <p class="status status-error">{changeReviewProjectionState.error}</p>
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
