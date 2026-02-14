<script lang="ts">
	import Sidebar from './Sidebar.svelte';
	import PanelResizer from './PanelResizer.svelte';
	import BeatEditor from '../editor/BeatEditor.svelte';
	import Timeline from '../timeline/Timeline.svelte';
	import { PANEL } from '$lib/types.js';
	import { projectState } from '$lib/stores/project.svelte.js';
	import { timelineState } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { createProject, getAiStatus, undo, redo, saveProject, deleteClip, exportPdf } from '$lib/api.js';
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
				key: 'Delete', description: 'Delete selected clip', skipInInput: true,
				action: () => {
					if (editorState.selectedClipId) {
						const id = editorState.selectedClipId;
						editorState.selectedClipId = null;
						editorState.selectedClip = null;
						deleteClip(id).catch(() => {});
					}
				},
			}),
			registerShortcut({
				key: '=', ctrl: true, description: 'Zoom in',
				action: () => { timelineState.zoom = Math.min(timelineState.zoom * 1.25, 10); },
			}),
			registerShortcut({
				key: '-', ctrl: true, description: 'Zoom out',
				action: () => { timelineState.zoom = Math.max(timelineState.zoom / 1.25, 0.1); },
			}),
			registerShortcut({
				key: '0', ctrl: true, description: 'Reset zoom',
				action: () => { timelineState.zoom = 1.0; },
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

	async function handleNewProject(template: string) {
		const project = await createProject('Untitled Episode', template);
		projectState.current = project;
		timelineState.timeline = project.timeline;
		storyState.arcs = project.arcs;
		storyState.characters = project.characters;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-shell" class:sidebar-open={sidebarOpen}>
	{#if sidebarOpen}
		<Sidebar onclose={() => sidebarOpen = false} />
	{/if}

	<div class="main-area">
		{#if projectState.current}
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
			</div>

			<div class="editor-panel" style="height: {editorHeight}px">
				<BeatEditor />
			</div>

			<PanelResizer
				min={PANEL.MIN_EDITOR_HEIGHT_PX}
				bind:position={editorHeight}
			/>

			<div class="timeline-panel" style="flex: 1; min-height: {PANEL.MIN_TIMELINE_HEIGHT_PX}px">
				<Timeline />
			</div>
		{:else}
			<div class="welcome">
				<h1>Eidetic</h1>
				<p>AI-driven script writing for 30-minute TV episodes</p>
				<div class="template-picker">
					<button onclick={() => handleNewProject('multi_cam')}>
						Multi-Cam Sitcom
					</button>
					<button onclick={() => handleNewProject('single_cam')}>
						Single-Cam Dramedy
					</button>
					<button onclick={() => handleNewProject('animated')}>
						Animated Comedy
					</button>
				</div>
			</div>
		{/if}
	</div>

	{#if !sidebarOpen}
		<button class="sidebar-toggle" onclick={() => sidebarOpen = true}>
			&#9776;
		</button>
	{/if}

	{#if projectState.current}
		<div class="ai-indicator" title={editorState.aiStatus?.connected ? `AI: ${editorState.aiStatus.model ?? 'connected'}` : 'AI: disconnected'}>
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

	.timeline-panel {
		overflow: hidden;
		background: var(--color-bg-primary);
	}

	.welcome {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 16px;
		color: var(--color-text-secondary);
	}

	.welcome h1 {
		font-size: 2.5rem;
		font-weight: 300;
		color: var(--color-text-primary);
		margin: 0;
	}

	.welcome p {
		margin: 0 0 24px;
	}

	.template-picker {
		display: flex;
		gap: 12px;
	}

	.template-picker button {
		padding: 12px 24px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.9rem;
		transition: background 0.15s;
	}

	.template-picker button:hover {
		background: var(--color-bg-hover);
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

	.ai-indicator {
		position: fixed;
		bottom: 8px;
		right: 12px;
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
		background: #22c55e;
	}

	.ai-dot.disconnected {
		background: #ef4444;
	}
</style>
