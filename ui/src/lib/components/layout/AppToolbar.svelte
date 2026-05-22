<script lang="ts">
  import type { WorkspaceMode } from '$lib/stores/workspaceMode.svelte.js';

  let {
    onsave,
    onexport,
    workspaceMode,
    onworkspace,
  }: {
    onsave: () => void;
    onexport: () => void;
    workspaceMode: WorkspaceMode;
    onworkspace: (mode: WorkspaceMode) => void;
  } = $props();

  const workspaceModes: { mode: WorkspaceMode; label: string }[] = [
    { mode: 'script', label: 'Script' },
    { mode: 'graph', label: 'Graph' },
    { mode: 'split', label: 'Split' },
  ];
</script>

<div class="toolbar">
  <div class="workspace-controls" role="group" aria-label="Workspace mode">
    {#each workspaceModes as option (option.mode)}
      <button
        class="mode-btn"
        class:active={workspaceMode === option.mode}
        aria-pressed={workspaceMode === option.mode}
        onclick={() => onworkspace(option.mode)}
      >
        {option.label}
      </button>
    {/each}
  </div>

  <div class="toolbar-actions">
    <button class="toolbar-btn" title="Save (Ctrl+S)" onclick={onsave}>&#128190;</button>
    <button class="toolbar-btn" title="Export PDF (Ctrl+Shift+E)" onclick={onexport}>PDF</button>
  </div>
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 2px;
    justify-content: space-between;
    padding: 2px 8px;
    background: var(--color-bg-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
    flex-shrink: 0;
  }

  .workspace-controls,
  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .mode-btn,
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

  .mode-btn.active {
    background: var(--color-bg-surface);
    border-color: var(--color-border-default);
    color: var(--color-text-primary);
  }

  .mode-btn:hover:not(:disabled),
  .toolbar-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    border-color: var(--color-border-subtle);
  }

  .mode-btn:disabled,
  .toolbar-btn:disabled {
    opacity: 0.35;
    cursor: default;
  }
</style>
