<script lang="ts">
  let {
    onsave,
    onexport,
  }: {
    onsave: () => void;
    onexport: () => void;
  } = $props();
</script>

<div class="toolbar">
  <button class="toolbar-btn" title="Save (Ctrl+S)" onclick={onsave}>&#128190;</button>
  <button class="toolbar-btn" title="Export PDF (Ctrl+Shift+E)" onclick={onexport}>PDF</button>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px 8px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .toolbar-btn {
    background: none;
    border: 1px solid transparent;
    color: var(--color-text-secondary);
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85rem;
    line-height: 1;
  }

  .toolbar-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-subtle);
  }

  .toolbar-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
