<script lang="ts">
  import BeatEditor from '../editor/BeatEditor.svelte';
  import ScriptPanel from '../editor/ScriptPanel.svelte';
  import { PANEL } from '$lib/timelineTypes.js';
  import type { WorkspaceMode } from '$lib/stores/workspaceMode.svelte.js';
  import GraphWorkspacePanel from './GraphWorkspacePanel.svelte';
  import PanelResizer from './PanelResizer.svelte';

  let {
    workspaceMode,
  }: {
    workspaceMode: WorkspaceMode;
  } = $props();

  let editorHeight = $state(300);
</script>

{#if workspaceMode === 'script'}
  <div class="editor-panel" style="height: {editorHeight}px">
    <BeatEditor />
  </div>

  <PanelResizer min={PANEL.MIN_EDITOR_HEIGHT_PX} bind:position={editorHeight} />

  <div class="script-panel">
    <ScriptPanel />
  </div>
{:else if workspaceMode === 'graph'}
  <div class="workspace-panel">
    <GraphWorkspacePanel />
  </div>
{:else}
  <div class="split-workspace">
    <div class="split-pane">
      <BeatEditor />
    </div>
    <div class="split-pane">
      <GraphWorkspacePanel />
    </div>
  </div>
{/if}

<style>
  .editor-panel {
    overflow: auto;
    border-bottom: 1px solid var(--color-border-default);
    background: var(--color-bg-secondary);
  }

  .script-panel {
    flex: 1;
    overflow: hidden;
    background: var(--color-bg-secondary);
  }

  .workspace-panel {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--color-bg-secondary);
  }

  .split-workspace {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(280px, 0.9fr);
    flex: 1;
    min-height: 0;
    overflow: hidden;
    background: var(--color-bg-secondary);
  }

  .split-pane {
    min-width: 0;
    min-height: 0;
    overflow: auto;
    border-right: 1px solid var(--color-border-default);
  }

  .split-pane:last-child {
    border-right: 0;
  }
</style>
