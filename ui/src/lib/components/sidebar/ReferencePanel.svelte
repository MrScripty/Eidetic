<script lang="ts">
	import type { ReferenceDocument } from '$lib/types.js';
	import { uploadReference, listReferences, deleteReference } from '$lib/api.js';
	import { notify } from '$lib/stores/notifications.svelte.js';

	let refs = $state<ReferenceDocument[]>([]);
	let name = $state('');
	let content = $state('');
	let docType = $state('style_guide');

	$effect(() => {
		listReferences().then((r) => (refs = r)).catch(() => {});
	});

	async function handleUpload() {
		if (!name.trim() || !content.trim()) return;
		try {
			const doc = await uploadReference(name.trim(), content.trim(), docType);
			refs = [...refs, doc];
			name = '';
			content = '';
			notify('success', 'Reference uploaded — embedding in background');
		} catch {
			notify('error', 'Failed to upload reference');
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteReference(id);
			refs = refs.filter((r) => r.id !== id);
			notify('info', 'Reference removed');
		} catch {
			notify('error', 'Failed to delete reference');
		}
	}
</script>

<div class="ref-panel">
	<h3>Reference Materials</h3>

	<div class="ref-form">
		<input type="text" bind:value={name} placeholder="Document name" />
		<select bind:value={docType}>
			<option value="style_guide">Style Guide</option>
			<option value="character_bible">Character Bible</option>
			<option value="world_building">World Building</option>
			<option value="previous_episode">Previous Episode</option>
			<option value="custom">Custom</option>
		</select>
		<textarea bind:value={content} placeholder="Paste reference text..." rows="4"></textarea>
		<button onclick={handleUpload} disabled={!name.trim() || !content.trim()}>Upload</button>
	</div>

	<div class="ref-list">
		{#each refs as ref (ref.id)}
			<div class="ref-item">
				<span class="ref-name">{ref.name}</span>
				<span class="ref-type">{typeof ref.doc_type === 'string' ? ref.doc_type : 'custom'}</span>
				<button class="ref-delete" onclick={() => handleDelete(ref.id)}>&#x2715;</button>
			</div>
		{/each}
		{#if refs.length === 0}
			<p class="ref-empty">No reference materials uploaded yet.</p>
		{/if}
	</div>
</div>

<style>
	.ref-panel {
		padding: 8px;
	}

	h3 {
		font-size: 0.8rem;
		color: var(--color-text-secondary);
		margin: 0 0 8px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.ref-form {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-bottom: 12px;
	}

	.ref-form input,
	.ref-form select,
	.ref-form textarea {
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		padding: 4px 8px;
		font-size: 0.75rem;
		font-family: inherit;
	}

	.ref-form textarea {
		resize: vertical;
	}

	.ref-form button {
		align-self: flex-end;
		padding: 4px 12px;
		font-size: 0.75rem;
		background: var(--color-accent);
		color: var(--color-bg-primary);
		border: none;
		border-radius: 4px;
		cursor: pointer;
	}

	.ref-form button:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.ref-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.ref-item {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px;
		background: var(--color-bg-primary);
		border-radius: 4px;
		font-size: 0.75rem;
	}

	.ref-name {
		flex: 1;
		color: var(--color-text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.ref-type {
		color: var(--color-text-muted);
		font-size: 0.65rem;
	}

	.ref-delete {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0 2px;
		font-size: 0.7rem;
	}

	.ref-delete:hover {
		color: #ef4444;
	}

	.ref-empty {
		color: var(--color-text-muted);
		font-size: 0.7rem;
		text-align: center;
		padding: 12px 0;
		margin: 0;
	}
</style>
