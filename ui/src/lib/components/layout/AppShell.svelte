<script lang="ts">
	import Sidebar from './Sidebar.svelte';
	import PanelResizer from './PanelResizer.svelte';
	import BeatEditor from '../editor/BeatEditor.svelte';
	import Timeline from '../timeline/Timeline.svelte';
	import { PANEL } from '$lib/types.js';
	import { projectState } from '$lib/stores/project.svelte.js';
	import { timelineState } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { createProject } from '$lib/api.js';

	let sidebarOpen = $state(true);
	let editorHeight = $state(300);

	async function handleNewProject(template: string) {
		const project = await createProject('Untitled Episode', template);
		projectState.current = project;
		timelineState.timeline = project.timeline;
		storyState.arcs = project.arcs;
		storyState.characters = project.characters;
	}
</script>

<div class="app-shell" class:sidebar-open={sidebarOpen}>
	{#if sidebarOpen}
		<Sidebar onclose={() => sidebarOpen = false} />
	{/if}

	<div class="main-area">
		{#if projectState.current}
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
</style>
