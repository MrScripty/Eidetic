<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import PanelResizer from './PanelResizer.svelte';
  import BottomTimelineStack from './BottomTimelineStack.svelte';
  import AppToolbar from './AppToolbar.svelte';
  import GraphWorkspacePanel from './GraphWorkspacePanel.svelte';
  import BeatEditor from '../editor/BeatEditor.svelte';
  import ScriptPanel from '../editor/ScriptPanel.svelte';
  import BibleGraphNodeDetail from '../sidebar/bible/BibleGraphNodeDetail.svelte';
  import { PANEL, mainTimelinePanelHeightPx } from '$lib/timelineTypes.js';
  import { projectState } from '$lib/stores/project.svelte.js';
  import { timelineState, zoomToFit, zoomTo } from '$lib/stores/timeline.svelte.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import { aiStatusState, startAiStatusPolling } from '$lib/stores/aiStatus.svelte.js';
  import { bibleState, selectBibleGraphNode } from '$lib/stores/bible.svelte.js';
  import { saveProject, exportPdf } from '$lib/api.js';
  import { registerShortcut, handleKeydown } from '$lib/stores/shortcuts.svelte.js';
  import { notify } from '$lib/stores/notifications.svelte.js';
  import { applyDeleteTimelineNodeCommand } from '$lib/stores/timelineRenderProjection.svelte.js';
  import { refreshSelectedNodeEditorProjection } from '$lib/stores/selectedNodeEditorProjection.svelte.js';
  import { setWorkspaceMode, workspaceModeState } from '$lib/stores/workspaceMode.svelte.js';

  $effect(() => {
    return startAiStatusPolling();
  });

  // Register keyboard shortcuts.
  $effect(() => {
    const unsubs = [
      registerShortcut({
        key: 's',
        ctrl: true,
        description: 'Save',
        action: () => {
          saveProject()
            .then(() => notify('success', 'Saved'))
            .catch(() => notify('error', 'Save failed'));
        },
      }),
      registerShortcut({
        key: 'Delete',
        description: 'Delete selected node',
        skipInInput: true,
        action: () => {
          if (editorState.selectedNodeId) {
            const id = editorState.selectedNodeId;
            editorState.selectedNodeId = null;
            editorState.selectedLevel = null;
            void refreshSelectedNodeEditorProjection(null).catch(() => {});
            applyDeleteTimelineNodeCommand({ node_id: id }).catch(() => {});
          }
        },
      }),
      registerShortcut({
        key: '=',
        ctrl: true,
        description: 'Zoom in',
        action: () => {
          zoomTo(timelineState.zoom * 1.25);
        },
      }),
      registerShortcut({
        key: '-',
        ctrl: true,
        description: 'Zoom out',
        action: () => {
          zoomTo(timelineState.zoom / 1.25);
        },
      }),
      registerShortcut({
        key: '0',
        ctrl: true,
        description: 'Zoom to fit',
        action: () => {
          zoomToFit();
        },
      }),
      registerShortcut({
        key: 'e',
        ctrl: true,
        shift: true,
        description: 'Export PDF',
        action: () => {
          handleExportPdf();
        },
      }),
    ];
    return () => unsubs.forEach((fn) => fn());
  });

  let sidebarOpen = $state(true);
  let editorHeight = $state(300);
  let timelinePreferredHeight = $state(mainTimelinePanelHeightPx());
  let rightPanelWidth = $state(PANEL.DEFAULT_RELATIONSHIP_WIDTH_PX);
  let windowHeight = $state(0);

  const selectedGraphNodeId = $derived(bibleState.selectedGraphNodeId);
  const bibleDetailOpen = $derived(selectedGraphNodeId !== null);
  const rightPanelOpen = $derived(bibleDetailOpen);
  const workspaceMode = $derived(workspaceModeState.mode);
  const maxTimelineHeight = $derived.by(() => {
    if (windowHeight <= 0) return Infinity;
    return Math.max(
      PANEL.MIN_TIMELINE_HEIGHT_PX,
      windowHeight - PANEL.MIN_UPPER_WORKSPACE_HEIGHT_PX - PANEL.RESIZER_HEIGHT_PX,
    );
  });
  const timelineHeight = $derived(
    Math.max(PANEL.MIN_TIMELINE_HEIGHT_PX, Math.min(timelinePreferredHeight, maxTimelineHeight)),
  );

  async function handleExportPdf() {
    try {
      const blob = await exportPdf();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'screenplay.pdf';
      a.click();
      URL.revokeObjectURL(url);
      notify('success', 'PDF exported');
    } catch (e) {
      notify('error', `Export failed: ${e instanceof Error ? e.message : 'unknown error'}`);
    }
  }

  function handleSave() {
    saveProject()
      .then(() => notify('success', 'Saved'))
      .catch(() => notify('error', 'Save failed'));
  }
</script>

<svelte:window bind:innerHeight={windowHeight} onkeydown={handleKeydown} />

<div
  class="app-shell"
  class:has-project={projectState.current}
  style="
		--panel-resizer-height: {PANEL.RESIZER_HEIGHT_PX}px;
		{windowHeight > 0 ? `height: ${windowHeight}px;` : ''}
	"
>
  {#if projectState.current}
    <div class="upper-section">
      {#if sidebarOpen}
        <Sidebar onclose={() => (sidebarOpen = false)} />
      {/if}

      <div class="main-area">
        <AppToolbar
          onsave={handleSave}
          onexport={handleExportPdf}
          {workspaceMode}
          onworkspace={setWorkspaceMode}
        />

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
      </div>

      {#if rightPanelOpen}
        <PanelResizer
          orientation="vertical"
          min={PANEL.MIN_RELATIONSHIP_WIDTH_PX}
          max={PANEL.MAX_RELATIONSHIP_WIDTH_PX}
          bind:position={rightPanelWidth}
        />
        <aside class="right-panel" style="width: {rightPanelWidth}px">
          {#if selectedGraphNodeId}
            <div class="entity-detail-panel">
              <BibleGraphNodeDetail
                nodeId={selectedGraphNodeId}
                onclose={() => selectBibleGraphNode(null)}
              />
            </div>
          {/if}
        </aside>
      {/if}
    </div>

    <PanelResizer
      min={PANEL.MIN_TIMELINE_HEIGHT_PX}
      max={maxTimelineHeight}
      reverse={true}
      bind:position={timelinePreferredHeight}
    />

    <BottomTimelineStack {timelineHeight} />
  {/if}

  {#if !sidebarOpen && projectState.current}
    <button class="sidebar-toggle" onclick={() => (sidebarOpen = true)}> &#9776; </button>
  {/if}

  {#if projectState.current}
    <div
      class="ai-indicator"
      style="right: {rightPanelOpen ? rightPanelWidth + 16 : 12}px"
      title={aiStatusState.status?.connected
        ? `AI: ${aiStatusState.status.model ?? 'connected'}`
        : 'AI: disconnected'}
    >
      <span
        class="ai-dot"
        class:connected={aiStatusState.status?.connected}
        class:disconnected={aiStatusState.status && !aiStatusState.status.connected}
      ></span>
    </div>
  {/if}
</div>

<style>
  .app-shell {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .app-shell.has-project {
    display: grid;
    grid-template-rows: minmax(0, 1fr) var(--panel-resizer-height) auto;
  }

  .upper-section {
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  :global(.app-shell.has-project > .resizer) {
    height: var(--panel-resizer-height);
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }

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

  .sidebar-toggle {
    position: fixed;
    top: 8px;
    left: 8px;
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    border: 1px solid var(--color-border-default);
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
    font-size: 1.2rem;
    z-index: 10;
  }

  .right-panel {
    display: flex;
    flex-direction: column;
    background: var(--color-bg-secondary);
    border-left: 1px solid var(--color-border-default);
    flex-shrink: 0;
    overflow: hidden;
  }

  .entity-detail-panel {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .ai-indicator {
    position: fixed;
    bottom: 8px;
    z-index: 10;
    display: flex;
    align-items: center;
    padding: 4px 8px;
    background: var(--color-bg-surface);
    border: 1px solid var(--color-border-subtle);
    border-radius: 10px;
    cursor: default;
  }

  .ai-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-muted);
  }

  .ai-dot.connected {
    background: var(--color-success);
  }

  .ai-dot.disconnected {
    background: var(--color-danger);
  }
</style>
