<script lang="ts">
  import type { FieldValue } from '$lib/projectionTypes.js';
  import type { BibleGraphSnapshotProjection } from '$lib/bibleGraphTypes.js';

  let {
    snapshots,
  }: {
    snapshots: BibleGraphSnapshotProjection[];
  } = $props();

  const sortedSnapshots = $derived(
    [...snapshots].sort(
      (a, b) =>
        a.snapshot.at_ms - b.snapshot.at_ms ||
        a.snapshot.sort_order - b.snapshot.sort_order ||
        a.snapshot.label.localeCompare(b.snapshot.label) ||
        a.snapshot.id.localeCompare(b.snapshot.id),
    ),
  );

  function formatValue(value: FieldValue | null | undefined): string {
    if (!value) return '';
    switch (value.type) {
      case 'text':
        return value.value;
      case 'integer':
      case 'number':
        return value.value.toString();
      case 'bool':
        return value.value ? 'True' : 'False';
      case 'object_ref':
        return `${value.value.kind}: ${value.value.id}`;
      case 'asset_ref':
        return value.value;
    }
  }
</script>

<section class="snapshot-section">
  <h3>Snapshots</h3>
  {#if sortedSnapshots.length > 0}
    <ul class="snapshot-list">
      {#each sortedSnapshots as projection (projection.snapshot.id)}
        <li>
          <div class="snapshot-heading">
            <span class="snapshot-label">{projection.snapshot.label}</span>
            <span class="snapshot-time">{projection.snapshot.at_ms.toLocaleString()} ms</span>
          </div>
          {#if projection.fields.length > 0}
            <dl class="snapshot-fields">
              {#each projection.fields as field (field.id)}
                <div>
                  <dt>{field.part_name} / {field.field_key}</dt>
                  <dd>{formatValue(field.value) || 'Empty'}</dd>
                </div>
              {/each}
            </dl>
          {:else}
            <p class="muted">No fields</p>
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <p class="muted">No snapshots</p>
  {/if}
</section>

<style>
  .snapshot-section {
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-subtle);
  }

  h3,
  p,
  dl {
    margin: 0;
  }

  h3 {
    color: var(--color-text-primary);
    font-size: 0.85rem;
    font-weight: 600;
  }

  .snapshot-list {
    display: grid;
    gap: 12px;
    list-style: none;
    margin: 12px 0 0;
    padding: 0;
  }

  .snapshot-list li {
    display: grid;
    gap: 8px;
  }

  .snapshot-heading {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
  }

  .snapshot-label {
    color: var(--color-text-primary);
    font-size: 0.8rem;
    overflow-wrap: anywhere;
  }

  .snapshot-time {
    margin-left: auto;
    color: var(--color-text-muted);
    flex: none;
    font-size: 0.7rem;
  }

  .snapshot-fields {
    display: grid;
    gap: 6px;
  }

  .snapshot-fields div {
    display: grid;
    gap: 2px;
  }

  dt {
    color: var(--color-text-muted);
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  dd {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  .muted {
    color: var(--color-text-muted);
    font-size: 0.8rem;
  }
</style>
