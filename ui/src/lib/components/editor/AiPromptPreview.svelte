<script lang="ts">
  let {
    notes,
    context,
    loading,
    onrefresh = () => {},
    label = 'Raw AI Prompt',
    refreshable = true,
  }: {
    notes: string;
    context: { system: string; user: string } | null;
    loading: boolean;
    onrefresh?: () => void;
    label?: string;
    refreshable?: boolean;
  } = $props();
</script>

{#if notes.trim()}
  <details class="context-panel">
    <summary class="context-panel-summary">
      {label}
      {#if refreshable}
        <button
          type="button"
          class="context-refresh-btn"
          onclick={(event) => {
            event.stopPropagation();
            onrefresh();
          }}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Refresh'}
        </button>
      {/if}
    </summary>
    {#if context}
      <div class="context-display">
        <details>
          <summary class="context-heading">System Prompt</summary>
          <pre class="context-text">{context.system}</pre>
        </details>
        <details open>
          <summary class="context-heading">User Prompt</summary>
          <pre class="context-text">{context.user}</pre>
        </details>
      </div>
    {:else if loading}
      <p class="context-loading">Loading context...</p>
    {:else}
      <p class="context-loading">No context available</p>
    {/if}
  </details>
{/if}
