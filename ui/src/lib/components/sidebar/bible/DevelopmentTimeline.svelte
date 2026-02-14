<script lang="ts">
	import type { Entity, EntitySnapshot } from '$lib/types.js';
	import { addSnapshot, updateSnapshot, deleteSnapshot } from '$lib/api.js';
	import DevelopmentPointCard from './DevelopmentPointCard.svelte';

	let { entity }: { entity: Entity } = $props();

	let adding = $state(false);
	let newAtMs = $state(0);
	let newDescription = $state('');

	const sortedSnapshots = $derived(
		[...entity.snapshots].sort((a, b) => a.at_ms - b.at_ms)
	);

	async function handleAdd() {
		if (!newDescription.trim()) return;
		await addSnapshot(entity.id, {
			at_ms: newAtMs,
			description: newDescription.trim(),
		});
		adding = false;
		newDescription = '';
		newAtMs = 0;
	}

	async function handleUpdate(idx: number, updates: Partial<EntitySnapshot>) {
		await updateSnapshot(entity.id, idx, updates);
	}

	async function handleDelete(idx: number) {
		await deleteSnapshot(entity.id, idx);
	}

	function cancelAdd() {
		adding = false;
		newDescription = '';
		newAtMs = 0;
	}
</script>

<div class="dev-timeline">
	<div class="timeline-header">
		<span class="section-label">Development Timeline</span>
		<button class="add-point-btn" onclick={() => adding = true}>+ Add</button>
	</div>

	{#if adding}
		<div class="add-form">
			<label class="form-label">
				Time (seconds)
				<input
					class="form-input"
					type="number"
					min="0"
					step="1"
					bind:value={newAtMs}
					oninput={(e) => newAtMs = parseInt((e.target as HTMLInputElement).value) * 1000 || 0}
				/>
			</label>
			<label class="form-label">
				Description
				<textarea
					class="form-textarea"
					bind:value={newDescription}
					placeholder="What changes at this point..."
					onkeydown={(e) => { if (e.key === 'Enter' && e.ctrlKey) handleAdd(); if (e.key === 'Escape') cancelAdd(); }}
				></textarea>
			</label>
			<div class="form-actions">
				<button class="save-btn" onclick={handleAdd}>Add Point</button>
				<button class="cancel-btn" onclick={cancelAdd}>Cancel</button>
			</div>
		</div>
	{/if}

	{#if sortedSnapshots.length === 0 && !adding}
		<p class="empty-state">No development points yet. Add one to track how this entity changes over the episode.</p>
	{/if}

	<div class="points-list">
		{#each sortedSnapshots as snapshot, i (snapshot.at_ms + '-' + i)}
			<div class="point-row">
				{#if i > 0}
					<div class="connector-line"></div>
				{/if}
				<DevelopmentPointCard
					{snapshot}
					idx={i}
					onupdate={handleUpdate}
					ondelete={handleDelete}
				/>
			</div>
		{/each}
	</div>
</div>

<style>
	.dev-timeline {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.timeline-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.section-label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.add-point-btn {
		font-size: 0.75rem;
		color: var(--color-bible-development);
		background: none;
		border: none;
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 3px;
	}

	.add-point-btn:hover {
		background: var(--color-bg-hover);
	}

	.points-list {
		display: flex;
		flex-direction: column;
	}

	.point-row {
		position: relative;
	}

	.connector-line {
		width: 2px;
		height: 8px;
		background: var(--color-bible-development);
		margin-left: 16px;
		opacity: 0.4;
	}

	.empty-state {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		text-align: center;
		padding: 12px 8px;
		margin: 0;
	}

	.add-form {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
	}

	.form-label {
		display: flex;
		flex-direction: column;
		gap: 2px;
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}

	.form-input {
		padding: 4px 8px;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.85rem;
	}

	.form-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.form-textarea {
		padding: 4px 8px;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.85rem;
		min-height: 50px;
		resize: vertical;
	}

	.form-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.form-actions {
		display: flex;
		gap: 6px;
	}

	.save-btn, .cancel-btn {
		padding: 4px 12px;
		font-size: 0.75rem;
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		cursor: pointer;
	}

	.save-btn {
		background: var(--color-bible-development);
		color: white;
		border-color: var(--color-bible-development);
	}

	.cancel-btn {
		background: var(--color-bg-elevated);
		color: var(--color-text-secondary);
	}
</style>
