<script lang="ts">
	import { fade } from 'svelte/transition';
	import { projectState } from '$lib/stores/project.svelte.js';
	import { timelineState } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { createProject, loadProject, listProjects } from '$lib/api.js';
	import { notify } from '$lib/stores/notifications.svelte.js';
	import type { Project } from '$lib/types.js';

	type View = 'home' | 'new' | 'open';
	let view: View = $state('home');

	let projects: { name: string; path: string; modified: string }[] = $state([]);
	let loadingProjects = $state(false);
	let loadError: string | null = $state(null);
	let busy = $state(false);

	const templates = [
		{ id: 'multi_cam', label: 'Multi-Cam Sitcom', desc: 'Traditional multi-camera format with fast A/B cutting' },
		{ id: 'single_cam', label: 'Single-Cam Dramedy', desc: 'Single-camera comedy-drama with cinematic pacing' },
		{ id: 'animated', label: 'Animated Comedy', desc: 'Animated format with flexible scene structure' },
	];

	function hydrateStores(project: Project) {
		projectState.current = project;
		timelineState.timeline = project.timeline;
		storyState.arcs = project.arcs;
		storyState.entities = project.bible.entities;
	}

	async function fetchProjects() {
		loadingProjects = true;
		loadError = null;
		try {
			projects = await listProjects();
		} catch (e) {
			loadError = e instanceof Error ? e.message : 'Failed to load projects';
		} finally {
			loadingProjects = false;
		}
	}

	async function handleNewProject(templateId: string) {
		if (busy) return;
		busy = true;
		try {
			const project = await createProject('Untitled Episode', templateId);
			hydrateStores(project);
		} catch (e) {
			notify('error', `Failed to create project: ${e instanceof Error ? e.message : 'unknown error'}`);
			busy = false;
		}
	}

	async function handleLoadProject(path: string) {
		if (busy) return;
		busy = true;
		try {
			const project = await loadProject(path);
			hydrateStores(project);
		} catch (e) {
			notify('error', `Failed to load project: ${e instanceof Error ? e.message : 'unknown error'}`);
			busy = false;
		}
	}

	function formatDate(iso: string): string {
		try {
			const d = new Date(iso);
			return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' })
				+ ' ' + d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
		} catch {
			return iso;
		}
	}
</script>

{#if !projectState.current}
<div class="splash-overlay" transition:fade={{ duration: 200 }}>
	<div class="splash-card">
		{#if view === 'home'}
			<div class="branding">
				<h1 class="title">Eidetic</h1>
				<p class="tagline">AI-driven script writing for 30-minute TV episodes</p>
			</div>
			<div class="actions">
				<button class="action-btn" onclick={() => view = 'new'}>
					<span class="action-icon">+</span>
					<span class="action-label">New Project</span>
				</button>
				<button class="action-btn" onclick={() => { view = 'open'; fetchProjects(); }}>
					<span class="action-icon">&#128194;</span>
					<span class="action-label">Open Project</span>
				</button>
			</div>

		{:else if view === 'new'}
			<button class="back-btn" onclick={() => view = 'home'}>&larr; Back</button>
			<h2 class="heading">Choose a Template</h2>
			<div class="template-list">
				{#each templates as tmpl}
					<button
						class="template-card"
						disabled={busy}
						onclick={() => handleNewProject(tmpl.id)}
					>
						<span class="template-name">{tmpl.label}</span>
						<span class="template-desc">{tmpl.desc}</span>
					</button>
				{/each}
			</div>

		{:else if view === 'open'}
			<button class="back-btn" onclick={() => view = 'home'}>&larr; Back</button>
			<h2 class="heading">Open Project</h2>
			{#if loadingProjects}
				<p class="status-text">Loading projects...</p>
			{:else if loadError}
				<p class="error-text">{loadError}</p>
				<button class="retry-btn" onclick={fetchProjects}>Retry</button>
			{:else if projects.length === 0}
				<p class="status-text">No saved projects found.</p>
			{:else}
				<div class="project-list">
					{#each projects as proj}
						<button
							class="project-item"
							disabled={busy}
							onclick={() => handleLoadProject(proj.path)}
						>
							<span class="project-name">{proj.name}</span>
							<span class="project-modified">{formatDate(proj.modified)}</span>
						</button>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>
{/if}

<style>
	.splash-overlay {
		position: fixed;
		inset: 0;
		z-index: 200;
		background: var(--color-bg-primary);
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.splash-card {
		max-width: 520px;
		width: 100%;
		padding: 48px;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border-default);
		border-radius: 12px;
	}

	.branding {
		text-align: center;
		margin-bottom: 40px;
	}

	.title {
		font-size: 3rem;
		font-weight: 300;
		color: var(--color-text-primary);
		margin: 0 0 8px;
	}

	.tagline {
		color: var(--color-text-secondary);
		margin: 0;
	}

	.actions {
		display: flex;
		gap: 16px;
	}

	.action-btn {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 24px 16px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 8px;
		color: var(--color-text-primary);
		cursor: pointer;
		transition: background 0.15s;
	}

	.action-btn:hover {
		background: var(--color-bg-hover);
	}

	.action-icon {
		font-size: 1.8rem;
		line-height: 1;
	}

	.action-label {
		font-size: 0.95rem;
	}

	.back-btn {
		background: none;
		border: none;
		color: var(--color-text-secondary);
		cursor: pointer;
		padding: 0;
		margin-bottom: 16px;
		font-size: 0.85rem;
	}

	.back-btn:hover {
		color: var(--color-text-primary);
	}

	.heading {
		font-size: 1.3rem;
		font-weight: 400;
		color: var(--color-text-primary);
		margin: 0 0 20px;
	}

	.template-list {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.template-card {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 4px;
		padding: 16px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 8px;
		color: var(--color-text-primary);
		cursor: pointer;
		text-align: left;
		transition: background 0.15s;
	}

	.template-card:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.template-card:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.template-name {
		font-size: 0.95rem;
		font-weight: 500;
	}

	.template-desc {
		font-size: 0.8rem;
		color: var(--color-text-secondary);
	}

	.project-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 320px;
		overflow-y: auto;
	}

	.project-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 12px 16px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 8px;
		color: var(--color-text-primary);
		cursor: pointer;
		text-align: left;
		transition: background 0.15s;
	}

	.project-item:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.project-item:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.project-name {
		font-size: 0.9rem;
	}

	.project-modified {
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}

	.status-text {
		color: var(--color-text-secondary);
		text-align: center;
		padding: 24px 0;
		margin: 0;
	}

	.error-text {
		color: var(--color-danger);
		text-align: center;
		padding: 16px 0 8px;
		margin: 0;
	}

	.retry-btn {
		display: block;
		margin: 0 auto;
		padding: 6px 16px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
		color: var(--color-text-primary);
		cursor: pointer;
		font-size: 0.85rem;
	}

	.retry-btn:hover {
		background: var(--color-bg-hover);
	}
</style>
