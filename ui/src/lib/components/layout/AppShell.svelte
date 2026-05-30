<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import PanelResizer from './PanelResizer.svelte';
  import BottomTimelineStack from './BottomTimelineStack.svelte';
  import AppToolbar from './AppToolbar.svelte';
  import AppWorkspace from './AppWorkspace.svelte';
  import GraphRightInspector from './GraphRightInspector.svelte';
  import AiStatusIndicator from './AiStatusIndicator.svelte';
  import { PANEL, mainTimelinePanelHeightPx } from '$lib/timelineTypes.js';
  import { projectState } from '$lib/stores/project.svelte.js';
  import { timelineState, zoomToFit, zoomTo } from '$lib/stores/timeline.svelte.js';
  import { startAiStatusPolling } from '$lib/stores/aiStatus.svelte.js';
  import { bibleState } from '$lib/stores/bible.svelte.js';
  import { saveProject, exportPdf } from '$lib/api.js';
  import { registerShortcut, handleKeydown } from '$lib/stores/shortcuts.svelte.js';
  import { notify } from '$lib/stores/notifications.svelte.js';
  import {
    TIMELINE_KEYBOARD_STEP_MS,
    deleteSelectedTimelineNodeFromKeyboard,
    nudgeSelectedTimelineNode,
    resizeSelectedTimelineNodeEnd,
    resizeSelectedTimelineNodeStart,
    splitSelectedTimelineNodeAtPlayhead,
    type TimelineKeyboardCommandResult,
  } from '$lib/stores/timelineKeyboardCommands.js';
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
          void runTimelineShortcut(() => deleteSelectedTimelineNodeFromKeyboard(), 'Delete failed');
        },
      }),
      registerShortcut({
        key: 'Enter',
        ctrl: true,
        description: 'Split selected node at playhead',
        skipInInput: true,
        action: () => {
          void runTimelineShortcut(() => splitSelectedTimelineNodeAtPlayhead(), 'Split failed');
        },
      }),
      registerShortcut({
        key: 'ArrowLeft',
        alt: true,
        description: 'Move selected node earlier',
        skipInInput: true,
        action: () => {
          void runTimelineShortcut(
            () => nudgeSelectedTimelineNode(-TIMELINE_KEYBOARD_STEP_MS),
            'Move failed',
          );
        },
      }),
      registerShortcut({
        key: 'ArrowRight',
        alt: true,
        description: 'Move selected node later',
        skipInInput: true,
        action: () => {
          void runTimelineShortcut(
            () => nudgeSelectedTimelineNode(TIMELINE_KEYBOARD_STEP_MS),
            'Move failed',
          );
        },
      }),
      registerShortcut({
        key: 'ArrowLeft',
        alt: true,
        shift: true,
        description: 'Resize selected node start',
        skipInInput: true,
        action: () => {
          void runTimelineShortcut(
            () => resizeSelectedTimelineNodeStart(-TIMELINE_KEYBOARD_STEP_MS),
            'Resize failed',
          );
        },
      }),
      registerShortcut({
        key: 'ArrowRight',
        alt: true,
        shift: true,
        description: 'Resize selected node end',
        skipInInput: true,
        action: () => {
          void runTimelineShortcut(
            () => resizeSelectedTimelineNodeEnd(TIMELINE_KEYBOARD_STEP_MS),
            'Resize failed',
          );
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
  let timelinePreferredHeight = $state(mainTimelinePanelHeightPx());
  let rightPanelWidth = $state(PANEL.DEFAULT_RELATIONSHIP_WIDTH_PX);
  let windowHeight = $state(0);

  const graphSelection = $derived(bibleState.graphSelection);
  const graphDetailOpen = $derived(graphSelection.kind !== 'none');
  const rightPanelOpen = $derived(graphDetailOpen);
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
  const aiStatusRightOffset = $derived(rightPanelOpen ? rightPanelWidth + 16 : 12);

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

  async function runTimelineShortcut(
    action: () => Promise<TimelineKeyboardCommandResult>,
    failureLabel: string,
  ) {
    try {
      await action();
    } catch (error) {
      notify(
        'error',
        `${failureLabel}: ${error instanceof Error ? error.message : 'unknown error'}`,
      );
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

        <AppWorkspace {workspaceMode} />
      </div>

      {#if rightPanelOpen}
        <PanelResizer
          orientation="vertical"
          min={PANEL.MIN_RELATIONSHIP_WIDTH_PX}
          max={PANEL.MAX_RELATIONSHIP_WIDTH_PX}
          bind:position={rightPanelWidth}
        />
        <aside class="right-panel" style="width: {rightPanelWidth}px">
          <GraphRightInspector />
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
    <button type="button" class="sidebar-toggle" onclick={() => (sidebarOpen = true)}>
      &#9776;
    </button>
  {/if}

  {#if projectState.current}
    <AiStatusIndicator rightOffsetPx={aiStatusRightOffset} />
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
</style>
