<script lang="ts">
	import type { Character } from '$lib/types.js';
	import { updateCharacter, deleteCharacter } from '$lib/api.js';

	let { character, onback }: {
		character: Character;
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

	function handleInput(field: 'name' | 'description' | 'voice_notes', value: string) {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(async () => {
			await updateCharacter(character.id, { [field]: value });
		}, 500);
	}

	async function handleColorSelect(rgb: readonly [number, number, number]) {
		await updateCharacter(character.id, { color: { r: rgb[0], g: rgb[1], b: rgb[2] } });
	}

	async function handleDelete() {
		await deleteCharacter(character.id);
		onback();
	}
</script>

<div class="character-detail">
	<div class="detail-header">
		<button class="back-btn" onclick={onback}>&larr; Back</button>
		<button class="delete-btn" onclick={handleDelete}>Delete</button>
	</div>

	<div class="detail-body">
		<label class="field-label">Name</label>
		<input
			class="field-input"
			type="text"
			value={character.name}
			oninput={(e) => handleInput('name', (e.target as HTMLInputElement).value)}
		/>

		<label class="field-label">Description</label>
		<textarea
			class="field-textarea"
			value={character.description}
			placeholder="Describe this character..."
			oninput={(e) => handleInput('description', (e.target as HTMLTextAreaElement).value)}
		></textarea>

		<label class="field-label">Voice Notes</label>
		<textarea
			class="field-textarea"
			value={character.voice_notes}
			placeholder="How does this character speak? Mannerisms, vocabulary, tone..."
			oninput={(e) => handleInput('voice_notes', (e.target as HTMLTextAreaElement).value)}
		></textarea>

		<label class="field-label">Color</label>
		<div class="color-presets">
			{#each COLOR_PRESETS as rgb}
				<button
					class="color-swatch"
					class:active={character.color.r === rgb[0] && character.color.g === rgb[1] && character.color.b === rgb[2]}
					style="background: rgb({rgb[0]}, {rgb[1]}, {rgb[2]})"
					onclick={() => handleColorSelect(rgb)}
				></button>
			{/each}
		</div>
	</div>
</div>

<style>
	.character-detail {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.detail-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 8px 12px;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.back-btn {
		background: none;
		border: none;
		color: var(--color-accent);
		cursor: pointer;
		font-size: 0.85rem;
		padding: 4px 0;
	}

	.delete-btn {
		background: none;
		border: none;
		color: #e55;
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
</style>
