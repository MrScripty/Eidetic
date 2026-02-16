<script lang="ts">
	import type { Entity, EntityDetails } from '$lib/types.js';
	import { updateEntity, deleteEntity, addRelation, deleteRelation } from '$lib/api.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import DevelopmentTimeline from './DevelopmentTimeline.svelte';

	let { entity, onback }: {
		entity: Entity;
		onback: () => void;
	} = $props();

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	const COLOR_PRESETS = [
		[100, 149, 237], // cornflower blue
		[119, 221, 119], // pastel green
		[255, 179, 71],  // pastel orange
		[168, 85, 247],  // purple
		[239, 68, 68],   // red
		[251, 191, 36],  // amber
		[20, 184, 166],  // teal
		[236, 72, 153],  // pink
	] as const;

	function categoryColor(cat: string): string {
		switch (cat) {
			case 'Character': return 'var(--color-entity-character)';
			case 'Location': return 'var(--color-entity-location)';
			case 'Prop': return 'var(--color-entity-prop)';
			case 'Theme': return 'var(--color-entity-theme)';
			case 'Event': return 'var(--color-entity-event)';
			default: return 'var(--color-text-muted)';
		}
	}

	function debounceUpdate(updates: Partial<Omit<Entity, 'id'>>) {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(async () => {
			await updateEntity(entity.id, updates);
		}, 500);
	}

	function handleField(field: 'name' | 'tagline' | 'description', value: string) {
		debounceUpdate({ [field]: value });
	}

	function handleDetails(updates: Partial<EntityDetails>) {
		const merged = { ...entity.details, ...updates };
		debounceUpdate({ details: merged as EntityDetails });
	}

	async function handleColorSelect(rgb: readonly [number, number, number]) {
		await updateEntity(entity.id, { color: { r: rgb[0], g: rgb[1], b: rgb[2] } });
	}

	async function handleDelete() {
		await deleteEntity(entity.id);
		onback();
	}

	async function handleToggleLock() {
		await updateEntity(entity.id, { locked: !entity.locked });
	}

	// Relations
	let addingRelation = $state(false);
	let newRelationTarget = $state('');
	let newRelationLabel = $state('');

	async function handleAddRelation() {
		if (!newRelationTarget || !newRelationLabel.trim()) return;
		await addRelation(entity.id, { target_entity_id: newRelationTarget, label: newRelationLabel.trim() });
		addingRelation = false;
		newRelationTarget = '';
		newRelationLabel = '';
	}

	async function handleDeleteRelation(idx: number) {
		await deleteRelation(entity.id, idx);
	}

	const otherEntities = $derived(
		storyState.entities.filter(e => e.id !== entity.id)
	);
</script>

<div class="entity-detail">
	<div class="detail-header">
		<button class="back-btn" onclick={onback}>&times; Close</button>
		<span class="category-label" style="color: {categoryColor(entity.category)}">{entity.category}</span>
		<div class="header-actions">
			<button
				class="lock-btn"
				class:locked={entity.locked}
				title={entity.locked ? 'Unlock (allow AI edits)' : 'Lock (prevent AI edits)'}
				onclick={handleToggleLock}
			>{entity.locked ? '&#128274;' : '&#128275;'}</button>
			<button class="delete-btn" onclick={handleDelete}>Delete</button>
		</div>
	</div>

	<div class="detail-body">
		<label class="field-label">Name</label>
		<input
			class="field-input"
			type="text"
			value={entity.name}
			oninput={(e) => handleField('name', (e.target as HTMLInputElement).value)}
		/>

		<label class="field-label">Tagline</label>
		<input
			class="field-input"
			type="text"
			value={entity.tagline}
			placeholder="Brief summary (~50 tokens)"
			oninput={(e) => handleField('tagline', (e.target as HTMLInputElement).value)}
		/>

		<label class="field-label">Description</label>
		<textarea
			class="field-textarea"
			value={entity.description}
			placeholder="Full description..."
			oninput={(e) => handleField('description', (e.target as HTMLTextAreaElement).value)}
		></textarea>

		<!-- Category-specific fields -->
		{#if entity.details.type === 'Character'}
			<label class="field-label">Voice Notes</label>
			<textarea
				class="field-textarea"
				value={entity.details.voice_notes}
				placeholder="How does this character speak? Mannerisms, vocabulary, tone..."
				oninput={(e) => handleDetails({ voice_notes: (e.target as HTMLTextAreaElement).value } as any)}
			></textarea>

			<label class="field-label">Audience Knowledge</label>
			<textarea
				class="field-textarea small"
				value={entity.details.audience_knowledge}
				placeholder="What the audience knows about this character..."
				oninput={(e) => handleDetails({ audience_knowledge: (e.target as HTMLTextAreaElement).value } as any)}
			></textarea>

			<label class="field-label">Traits</label>
			<input
				class="field-input"
				type="text"
				value={entity.details.traits.join(', ')}
				placeholder="Comma-separated traits"
				oninput={(e) => handleDetails({ traits: (e.target as HTMLInputElement).value.split(',').map(t => t.trim()).filter(Boolean) } as any)}
			/>

		{:else if entity.details.type === 'Location'}
			<label class="field-label">INT/EXT</label>
			<input
				class="field-input"
				type="text"
				value={entity.details.int_ext}
				placeholder="INT. / EXT. / INT./EXT."
				oninput={(e) => handleDetails({ int_ext: (e.target as HTMLInputElement).value } as any)}
			/>

			<label class="field-label">Scene Heading Name</label>
			<input
				class="field-input"
				type="text"
				value={entity.details.scene_heading_name}
				placeholder="e.g. JERRY'S APARTMENT"
				oninput={(e) => handleDetails({ scene_heading_name: (e.target as HTMLInputElement).value } as any)}
			/>

			<label class="field-label">Atmosphere</label>
			<textarea
				class="field-textarea small"
				value={entity.details.atmosphere}
				placeholder="Visual feel, lighting, mood..."
				oninput={(e) => handleDetails({ atmosphere: (e.target as HTMLTextAreaElement).value } as any)}
			></textarea>

		{:else if entity.details.type === 'Prop'}
			<label class="field-label">Significance</label>
			<textarea
				class="field-textarea small"
				value={entity.details.significance}
				placeholder="Why is this prop important?"
				oninput={(e) => handleDetails({ significance: (e.target as HTMLTextAreaElement).value } as any)}
			></textarea>

		{:else if entity.details.type === 'Theme'}
			<label class="field-label">Manifestation</label>
			<textarea
				class="field-textarea small"
				value={entity.details.manifestation}
				placeholder="How does this theme manifest in the story?"
				oninput={(e) => handleDetails({ manifestation: (e.target as HTMLTextAreaElement).value } as any)}
			></textarea>

		{:else if entity.details.type === 'Event'}
			<label class="field-label">
				<span>Backstory Event</span>
				<input
					type="checkbox"
					checked={entity.details.is_backstory}
					onchange={(e) => handleDetails({ is_backstory: (e.target as HTMLInputElement).checked } as any)}
				/>
			</label>
		{/if}

		<!-- Color -->
		<label class="field-label">Color</label>
		<div class="color-presets">
			{#each COLOR_PRESETS as rgb}
				<button
					class="color-swatch"
					class:active={entity.color.r === rgb[0] && entity.color.g === rgb[1] && entity.color.b === rgb[2]}
					style="background: rgb({rgb[0]}, {rgb[1]}, {rgb[2]})"
					onclick={() => handleColorSelect(rgb)}
				></button>
			{/each}
		</div>

		<!-- Relations -->
		<div class="section-divider"></div>
		<div class="relations-section">
			<div class="section-header">
				<span class="field-label">Relations</span>
				<button class="add-inline-btn" onclick={() => addingRelation = true}>+ Add</button>
			</div>

			{#if addingRelation}
				<div class="add-relation-form">
					<select class="field-input" bind:value={newRelationTarget}>
						<option value="">Select entity...</option>
						{#each otherEntities as other}
							<option value={other.id}>{other.name}</option>
						{/each}
					</select>
					<input
						class="field-input"
						type="text"
						placeholder="Label (e.g. 'lives at', 'fears')"
						bind:value={newRelationLabel}
					/>
					<div class="form-actions">
						<button class="save-btn" onclick={handleAddRelation}>Add</button>
						<button class="cancel-btn" onclick={() => { addingRelation = false; newRelationTarget = ''; newRelationLabel = ''; }}>Cancel</button>
					</div>
				</div>
			{/if}

			{#each entity.relations as rel, i}
				{@const target = storyState.entities.find(e => e.id === rel.target_entity_id)}
				<div class="relation-row">
					<span class="relation-label">{rel.label}</span>
					<span class="relation-target">{target?.name ?? 'Unknown'}</span>
					<button class="remove-btn" onclick={() => handleDeleteRelation(i)}>&times;</button>
				</div>
			{/each}

			{#if entity.relations.length === 0 && !addingRelation}
				<p class="empty-hint">No relations</p>
			{/if}
		</div>

		<!-- Development Timeline -->
		<div class="section-divider"></div>
		<DevelopmentTimeline {entity} />

		<!-- Node References -->
		{#if entity.node_refs.length > 0}
			<div class="section-divider"></div>
			<span class="field-label">Linked Nodes ({entity.node_refs.length})</span>
			<div class="node-refs">
				{#each entity.node_refs as nodeId}
					<span class="node-ref">{nodeId.slice(0, 8)}</span>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.entity-detail {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.detail-header {
		display: flex;
		align-items: center;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
		gap: 8px;
	}

	.back-btn {
		background: none;
		border: none;
		color: var(--color-accent);
		cursor: pointer;
		font-size: 0.85rem;
		padding: 4px 0;
	}

	.category-label {
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		flex: 1;
	}

	.header-actions {
		display: flex;
		gap: 4px;
	}

	.lock-btn {
		background: none;
		border: none;
		cursor: pointer;
		font-size: 0.85rem;
		padding: 4px;
	}

	.lock-btn.locked {
		opacity: 1;
	}

	.delete-btn {
		background: none;
		border: none;
		color: var(--color-danger);
		cursor: pointer;
		font-size: 0.8rem;
		padding: 4px 8px;
	}

	.detail-body {
		flex: 1;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		overflow-y: auto;
	}

	.field-label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.field-input {
		padding: 6px 10px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.9rem;
	}

	.field-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.field-textarea {
		padding: 6px 10px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.85rem;
		min-height: 80px;
		resize: vertical;
	}

	.field-textarea.small {
		min-height: 50px;
	}

	.field-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.color-presets {
		display: flex;
		gap: 6px;
		flex-wrap: wrap;
	}

	.color-swatch {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		border: 2px solid transparent;
		cursor: pointer;
		padding: 0;
	}

	.color-swatch.active {
		border-color: var(--color-text-primary);
	}

	.color-swatch:hover {
		transform: scale(1.15);
	}

	.section-divider {
		height: 1px;
		background: var(--color-border-subtle);
		margin: 4px 0;
	}

	.relations-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.add-inline-btn {
		font-size: 0.75rem;
		color: var(--color-accent);
		background: none;
		border: none;
		cursor: pointer;
		padding: 2px 6px;
		border-radius: 3px;
	}

	.add-inline-btn:hover {
		background: var(--color-bg-hover);
	}

	.add-relation-form {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 6px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
	}

	.form-actions {
		display: flex;
		gap: 6px;
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

	.relation-row {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 6px;
		font-size: 0.85rem;
		border-radius: 4px;
	}

	.relation-row:hover {
		background: var(--color-bg-hover);
	}

	.relation-label {
		color: var(--color-text-muted);
		font-style: italic;
	}

	.relation-target {
		color: var(--color-text-primary);
		flex: 1;
	}

	.remove-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 0.9rem;
		padding: 0 4px;
		visibility: hidden;
	}

	.relation-row:hover .remove-btn {
		visibility: visible;
	}

	.remove-btn:hover {
		color: var(--color-danger);
	}

	.empty-hint {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		margin: 0;
		padding: 2px 0;
	}

	.node-refs {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.node-ref {
		font-size: 0.7rem;
		color: var(--color-text-secondary);
		background: var(--color-bg-elevated);
		padding: 2px 6px;
		border-radius: 4px;
		font-family: monospace;
	}
</style>
