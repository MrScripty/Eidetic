<script lang="ts">
  import type { ScriptImpactProjection } from '$lib/scriptTypes.js';
  import { scriptImpactCauseLabel } from './scriptImpactNotice.js';
  let { impact }: { impact: ScriptImpactProjection } = $props();
</script>

{#if impact.needs_review}
  <aside class="impact" aria-label="Screenplay needs review">
    <strong>Needs review</strong>
    <p>Story inputs have changed since this text was generated.</p>
    <details>
      <summary>What changed</summary>
      <ul>
        {#each impact.causes as cause (cause.dependency_id)}
          <li>
            {scriptImpactCauseLabel(cause)}
            {#if cause.input_excerpt}<blockquote>{cause.input_excerpt}</blockquote>{/if}
          </li>
        {/each}
      </ul>
    </details>
  </aside>
{:else if !impact.lineage_available}
  <p class="unavailable">Generation input history is unavailable for this segment.</p>
{/if}

<style>
  .impact {
    break-inside: avoid;
    padding: 8px;
    margin-bottom: 8px;
    border: 1px solid var(--color-border-subtle);
    font-size: 0.8rem;
  }
  .impact p {
    margin: 4px 0;
  }
  blockquote {
    margin: 4px 0;
    font-family: monospace;
    white-space: pre-wrap;
  }
  .unavailable {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }
</style>
