<script lang="ts">
	import Sidebar from './Sidebar.svelte';
	import PanelResizer from './PanelResizer.svelte';
	import BottomTimelineStack from './BottomTimelineStack.svelte';
	import BeatEditor from '../editor/BeatEditor.svelte';
	import ScriptPanel from '../editor/ScriptPanel.svelte';
	import RelationshipPanel from '../relationship/RelationshipPanel.svelte';
	import EntityDetail from '../sidebar/bible/EntityDetail.svelte';
	import { PANEL, mainTimelinePanelHeightPx } from '$lib/types.js';
	import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';
	import { projectState } from '$lib/stores/project.svelte.js';
	import { timelineState, zoomToFit, zoomTo } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { bibleState, selectEntity } from '$lib/stores/bible.svelte.js';
	import { getAiStatus, undo, redo, saveProject, deleteNode, exportPdf } from '$lib/api.js';
	import { registerShortcut, handleKeydown } from '$lib/stores/shortcuts.svelte.js';
	import { notify } from '$lib/stores/notifications.svelte.js';

	$effect(() => {
		getAiStatus().then(s => editorState.aiStatus = s).catch(() => {});
		const interval = setInterval(() => {
			getAiStatus().then(s => editorState.aiStatus = s).catch(() => {});
		}, 30_000);
		return () => clearInterval(interval);
	});

	// Register keyboard shortcuts.
	$effect(() => {
		const unsubs = [
			registerShortcut({
				key: 'z', ctrl: true, description: 'Undo',
				action: () => { undo().catch(() => {}); },
			}),
			registerShortcut({
				key: 'z', ctrl: true, shift: true, description: 'Redo',
				action: () => { redo().catch(() => {}); },
			}),
			registerShortcut({
				key: 'y', ctrl: true, description: 'Redo',
				action: () => { redo().catch(() => {}); },
			}),
			registerShortcut({
				key: 's', ctrl: true, description: 'Save',
				action: () => { saveProject().then(() => notify('success', 'Saved')).catch(() => notify('error', 'Save failed')); },
			}),
			registerShortcut({
				key: 'Delete', description: 'Delete selected node', skipInInput: true,
				action: () => {
					if (editorState.selectedNodeId) {
						const id = editorState.selectedNodeId;
						editorState.selectedNodeId = null;
						editorState.selectedNode = null;
						editorState.selectedLevel = null;
						deleteNode(id).catch(() => {});
					}
				},
			}),
			registerShortcut({
				key: '=', ctrl: true, description: 'Zoom in',
				action: () => { zoomTo(timelineState.zoom * 1.25); },
			}),
			registerShortcut({
				key: '-', ctrl: true, description: 'Zoom out',
				action: () => { zoomTo(timelineState.zoom / 1.25); },
			}),
			registerShortcut({
				key: '0', ctrl: true, description: 'Zoom to fit',
				action: () => { zoomToFit(); },
			}),
			registerShortcut({
				key: 'e', ctrl: true, shift: true, description: 'Export PDF',
				action: () => { handleExportPdf(); },
			}),
		];
		return () => unsubs.forEach(fn => fn());
	});

	let sidebarOpen = $state(true);
	let editorHeight = $state(300);
	let timelineHeight = $state(mainTimelinePanelHeightPx());
	let relationshipPanelOpen = $state(false);
	let relationshipPanelWidth = $state(PANEL.DEFAULT_RELATIONSHIP_WIDTH_PX);
	let windowHeight = $state(0);

	const selectedEntity = $derived(
		bibleState.selectedEntityId
			? storyState.entities.find(e => e.id === bibleState.selectedEntityId) ?? null
			: null
	);

	/** Right panel is open when graph is toggled OR an entity is selected. */
	const rightPanelOpen = $derived(relationshipPanelOpen || selectedEntity !== null);
	const bottomStackExtraHeight = $derived(
		characterTimelineState.visible ? PANEL.CHARACTER_TIMELINE_HEIGHT_PX : 0
	);
	const maxTimelineHeight = $derived.by(() => {
		if (windowHeight <= 0) return Infinity;
		return Math.max(
			PANEL.MIN_TIMELINE_HEIGHT_PX,
			windowHeight
				- PANEL.MIN_UPPER_WORKSPACE_HEIGHT_PX
				- PANEL.RESIZER_HEIGHT_PX
				- bottomStackExtraHeight,
		);
	});

	$effect(() => {
		if (timelineHeight > maxTimelineHeight) {
			timelineHeight = maxTimelineHeight;
		}
	});

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
				<Sidebar onclose={() => sidebarOpen = false} />
			{/if}

			<div class="main-area">
				<div class="toolbar">
					<button
						class="toolbar-btn"
						title="Undo (Ctrl+Z)"
						disabled={!editorState.canUndo}
						onclick={() => undo().catch(() => {})}
					>&#8617;</button>
					<button
						class="toolbar-btn"
						title="Redo (Ctrl+Shift+Z)"
						disabled={!editorState.canRedo}
						onclick={() => redo().catch(() => {})}
					>&#8618;</button>
					<div class="toolbar-sep"></div>
					<button
						class="toolbar-btn"
						title="Save (Ctrl+S)"
						onclick={() => saveProject().catch(() => {})}
					>&#128190;</button>
					<button
						class="toolbar-btn"
						title="Export PDF (Ctrl+Shift+E)"
						onclick={handleExportPdf}
					>PDF</button>
					<div class="toolbar-sep"></div>
					<button
						class="toolbar-btn"
						class:active={relationshipPanelOpen}
						title="Toggle relationship graph"
						onclick={() => relationshipPanelOpen = !relationshipPanelOpen}
					>Graph</button>
				</div>

				<div class="editor-panel" style="height: {editorHeight}px">
					<BeatEditor />
				</div>

				<PanelResizer
					min={PANEL.MIN_EDITOR_HEIGHT_PX}
					bind:position={editorHeight}
				/>

				<div class="script-panel">
					<ScriptPanel />
				</div>
			</div>

			{#if rightPanelOpen}
				<PanelResizer
					orientation="vertical"
					min={PANEL.MIN_RELATIONSHIP_WIDTH_PX}
					max={PANEL.MAX_RELATIONSHIP_WIDTH_PX}
					bind:position={relationshipPanelWidth}
				/>
				<aside class="right-panel" style="width: {relationshipPanelWidth}px">
					{#if relationshipPanelOpen}
						<RelationshipPanel
							width={relationshipPanelWidth}
							onclose={() => relationshipPanelOpen = false}
							compact={selectedEntity !== null}
						/>
					{/if}
					{#if selectedEntity}
						{#if relationshipPanelOpen}
							<div class="panel-divider"></div>
						{/if}
						<div class="entity-detail-panel">
							<EntityDetail entity={selectedEntity} onback={() => selectEntity(null)} />
						</div>
					{/if}
				</aside>
			{/if}
		</div>

		<PanelResizer
			min={PANEL.MIN_TIMELINE_HEIGHT_PX}
			max={maxTimelineHeight}
			reverse={true}
			bind:position={timelineHeight}
		/>

		<BottomTimelineStack {timelineHeight} />
	{/if}

	{#if !sidebarOpen && projectState.current}
		<button class="sidebar-toggle" onclick={() => sidebarOpen = true}>
			&#9776;
		</button>
	{/if}

	{#if projectState.current}
		<div class="ai-indicator" style="right: {rightPanelOpen ? relationshipPanelWidth + 16 : 12}px" title={editorState.aiStatus?.connected ? `AI: ${editorState.aiStatus.model ?? 'connected'}` : 'AI: disconnected'}>
			<span
				class="ai-dot"
				class:connected={editorState.aiStatus?.connected}
				class:disconnected={editorState.aiStatus && !editorState.aiStatus.connected}
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

	.toolbar-btn.active {
		background: var(--color-bg-hover);
		color: var(--color-accent);
		border-color: var(--color-border-default);
	}

	.toolbar-sep {
		width: 1px;
		height: 16px;
		background: var(--color-border-subtle);
		margin: 0 4px;
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

	.panel-divider {
		height: 1px;
		background: var(--color-border-default);
		flex-shrink: 0;
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
