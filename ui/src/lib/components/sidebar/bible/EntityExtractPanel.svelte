<script lang="ts">
	import type { ExtractionResult, EntityCategory } from '$lib/types.js';
	import { createEntity, addSnapshot, addClipRef, listEntities } from '$lib/api.js';
	import { storyState } from '$lib/stores/story.svelte.js';

	let { result, clipId, onclose }: {
		result: ExtractionResult;
		clipId: string;
		onclose: () => void;
	} = $props();

	let entityAccepted = $state<boolean[]>(result.new_entities.map(() => true));
	let snapshotAccepted = $state<boolean[]>(result.snapshot_suggestions.map(() => true));
	let applying = $state(false);

	function categoryColor(cat: EntityCategory): string {
		switch (cat) {
			case 'Character': return 'var(--color-entity-character)';
			case 'Location': return 'var(--color-entity-location)';
			case 'Prop': return 'var(--color-entity-prop)';
			case 'Theme': return 'var(--color-entity-theme)';
			case 'Event': return 'var(--color-entity-event)';
			default: return 'var(--color-text-muted)';
		}
	}

	let applyError = $state<string | null>(null);

	async function handleApply() {
		applying = true;
		applyError = null;
		try {
			// Create accepted new entities
			for (let i = 0; i < result.new_entities.length; i++) {
				if (!entityAccepted[i]) continue;
				const sug = result.new_entities[i]!;
				const entity = await createEntity({
					name: sug.name,
					category: sug.category,
					tagline: sug.tagline,
					description: sug.description,
				});
				// Link to this clip
				await addClipRef(entity.id, clipId);
			}

			// Refresh entity list so snapshot matching can find newly created entities
			storyState.entities = await listEntities();

			// Apply accepted snapshots to matching existing entities
			for (let i = 0; i < result.snapshot_suggestions.length; i++) {
				if (!snapshotAccepted[i]) continue;
				const sug = result.snapshot_suggestions[i]!;
				const match = storyState.entities.find(
					e => e.name.toLowerCase() === sug.entity_name.toLowerCase()
				);
				if (match) {
					await addSnapshot(match.id, {
						at_ms: 0,
						description: sug.description,
					});
				}
			}

			// Add clip refs for suggested entity IDs
			for (const entityId of result.clip_ref_suggestions) {
				await addClipRef(entityId, clipId).catch(() => {});
			}

			onclose();
		} catch (err) {
			applyError = err instanceof Error ? err.message : String(err);
			console.error('Failed to apply extraction results:', err);
		} finally {
			applying = false;
		}
	}

	function acceptAll() {
		entityAccepted = result.new_entities.map(() => true);
		snapshotAccepted = result.snapshot_suggestions.map(() => true);
	}

	function rejectAll() {
		entityAccepted = result.new_entities.map(() => false);
		snapshotAccepted = result.snapshot_suggestions.map(() => false);
	}

	const hasAny = $derived(
		result.new_entities.length > 0 || result.snapshot_suggestions.length > 0
	);

	const anySelected = $derived(
		entityAccepted.some(Boolean) || snapshotAccepted.some(Boolean) || result.clip_ref_suggestions.length > 0
	);
</script>

<div class="extract-panel">
	<div class="extract-header">
		<span class="title">Extraction Results</span>
		<button class="close-btn" onclick={onclose}>&times;</button>
	</div>

	{#if !hasAny}
		<p class="empty-msg">No new entities or development points found.</p>
	{:else}
		<div class="bulk-actions">
			<button class="bulk-btn" onclick={acceptAll}>Accept All</button>
			<button class="bulk-btn" onclick={rejectAll}>Reject All</button>
		</div>

		{#if result.new_entities.length > 0}
			<div class="section">
				<span class="section-label">New Entities ({result.new_entities.length})</span>
				{#each result.new_entities as sug, i}
					<label class="suggestion-row">
						<input type="checkbox" bind:checked={entityAccepted[i]} />
						<span class="sug-badge" style="color: {categoryColor(sug.category)}">{sug.category}</span>
						<span class="sug-name">{sug.name}</span>
						<span class="sug-tagline">{sug.tagline}</span>
					</label>
				{/each}
			</div>
		{/if}

		{#if result.snapshot_suggestions.length > 0}
			<div class="section">
				<span class="section-label">Development Points ({result.snapshot_suggestions.length})</span>
				{#each result.snapshot_suggestions as sug, i}
					<label class="suggestion-row">
						<input type="checkbox" bind:checked={snapshotAccepted[i]} />
						<span class="sug-name">{sug.entity_name}</span>
						<span class="sug-desc">{sug.description}</span>
					</label>
				{/each}
			</div>
		{/if}

		{#if applyError}
			<p class="error-msg">{applyError}</p>
		{/if}

		<button
			class="apply-btn"
			disabled={!anySelected || applying}
			onclick={handleApply}
		>
			{applying ? 'Applying...' : 'Apply Selected'}
		</button>
	{/if}
</div>

<style>
	.extract-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 8px;
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
	}

	.extract-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.title {
		font-size: 0.85rem;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 1.1rem;
		padding: 0 4px;
	}

	.empty-msg {
		font-size: 0.85rem;
		color: var(--color-text-muted);
		text-align: center;
		margin: 8px 0;
	}

	.bulk-actions {
		display: flex;
		gap: 6px;
	}

	.bulk-btn {
		padding: 2px 8px;
		font-size: 0.75rem;
		background: var(--color-bg-elevated);
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.bulk-btn:hover {
		background: var(--color-bg-hover);
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.section-label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.suggestion-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px;
		font-size: 0.85rem;
		border-radius: 4px;
		cursor: pointer;
	}

	.suggestion-row:hover {
		background: var(--color-bg-hover);
	}

	.sug-badge {
		font-size: 0.65rem;
		font-weight: 600;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.sug-name {
		color: var(--color-text-primary);
		font-weight: 500;
		flex-shrink: 0;
	}

	.sug-tagline, .sug-desc {
		color: var(--color-text-secondary);
		font-size: 0.8rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.apply-btn {
		padding: 6px 12px;
		background: var(--color-accent);
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.85rem;
	}

	.apply-btn:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.apply-btn:hover:not(:disabled) {
		background: var(--color-accent-hover);
	}

	.error-msg {
		font-size: 0.8rem;
		color: var(--color-entity-event);
		margin: 0;
		padding: 4px 6px;
		background: rgba(239, 68, 68, 0.1);
		border-radius: 3px;
	}
</style>
