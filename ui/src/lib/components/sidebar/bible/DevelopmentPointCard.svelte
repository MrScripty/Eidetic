<script lang="ts">
	import type { EntitySnapshot } from '$lib/types.js';
	import { formatTime } from '$lib/types.js';

	let { snapshot, idx, onupdate, ondelete }: {
		snapshot: EntitySnapshot;
		idx: number;
		onupdate: (idx: number, updates: Partial<EntitySnapshot>) => void;
		ondelete: (idx: number) => void;
	} = $props();

	let editing = $state(false);
	let editDescription = $state(snapshot.description);

	function save() {
		if (editDescription.trim() && editDescription !== snapshot.description) {
			onupdate(idx, { description: editDescription.trim() });
		}
		editing = false;
	}

	function cancel() {
		editDescription = snapshot.description;
		editing = false;
	}
</script>

<div class="dev-point">
	<div class="point-header">
		<span class="time-marker">{formatTime(snapshot.at_ms)}</span>
		<div class="point-actions">
			{#if !editing}
				<button class="action-btn" title="Edit" onclick={() => editing = true}>&#9998;</button>
			{/if}
			<button class="action-btn delete" title="Delete" onclick={() => ondelete(idx)}>&times;</button>
		</div>
	</div>

	{#if editing}
		<textarea
			class="edit-textarea"
			bind:value={editDescription}
			onkeydown={(e) => { if (e.key === 'Enter' && e.ctrlKey) save(); if (e.key === 'Escape') cancel(); }}
		></textarea>
		<div class="edit-actions">
			<button class="save-btn" onclick={save}>Save</button>
			<button class="cancel-btn" onclick={cancel}>Cancel</button>
		</div>
	{:else}
		<p class="description">{snapshot.description}</p>
	{/if}

	{#if snapshot.state_overrides?.emotional_state}
		<div class="override-tag">Mood: {snapshot.state_overrides.emotional_state}</div>
	{/if}
	{#if snapshot.state_overrides?.audience_knowledge}
		<div class="override-tag">Audience knows: {snapshot.state_overrides.audience_knowledge}</div>
	{/if}
</div>

<style>
	.dev-point {
		padding: 8px 10px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 6px;
		border-left: 3px solid var(--color-bible-development);
	}

	.point-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 4px;
	}

	.time-marker {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--color-bible-development);
		font-variant-numeric: tabular-nums;
	}

	.point-actions {
		display: flex;
		gap: 2px;
	}

	.action-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 0.8rem;
		padding: 2px 4px;
		border-radius: 3px;
	}

	.action-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.action-btn.delete:hover {
		color: var(--color-danger);
	}

	.description {
		margin: 0;
		font-size: 0.85rem;
		color: var(--color-text-primary);
		line-height: 1.4;
	}

	.override-tag {
		margin-top: 4px;
		font-size: 0.75rem;
		color: var(--color-text-secondary);
		font-style: italic;
	}

	.edit-textarea {
		width: 100%;
		padding: 6px 8px;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.85rem;
		min-height: 60px;
		resize: vertical;
		box-sizing: border-box;
	}

	.edit-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.edit-actions {
		display: flex;
		gap: 6px;
		margin-top: 4px;
	}

	.save-btn, .cancel-btn {
		padding: 2px 10px;
		font-size: 0.75rem;
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		cursor: pointer;
	}

	.save-btn {
		background: var(--color-accent);
		color: white;
		border-color: var(--color-accent);
	}

	.cancel-btn {
		background: var(--color-bg-elevated);
		color: var(--color-text-secondary);
	}
</style>
