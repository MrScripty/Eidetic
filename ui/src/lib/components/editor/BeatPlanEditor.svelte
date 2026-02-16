<script lang="ts">
	import type { ChildProposal, BeatType } from '$lib/types.js';

	let { beats, onapply, oncancel }: {
		beats: ChildProposal[];
		onapply: (beats: ChildProposal[]) => void;
		oncancel: () => void;
	} = $props();

	let editableBeats = $state<ChildProposal[]>([]);

	$effect(() => {
		editableBeats = beats.map(b => ({
			...b,
			characters: b.characters ? [...b.characters] : [],
			props: b.props ? [...b.props] : [],
			location: b.location ?? null,
		}));
	});

	const beatTypes: BeatType[] = ['Setup', 'Complication', 'Escalation', 'Climax', 'Resolution', 'Payoff', 'Callback'];

	function beatTypeLabel(bt: BeatType): string {
		if (typeof bt === 'string') return bt;
		return bt.Custom;
	}

	function addBeat() {
		editableBeats = [...editableBeats, { name: 'New Beat', beat_type: 'Setup', outline: '', weight: 1.0, characters: [], props: [], location: null }];
	}

	function removeBeat(idx: number) {
		editableBeats = editableBeats.filter((_, i) => i !== idx);
	}

	function totalWeight(): number {
		return editableBeats.reduce((sum, b) => sum + b.weight, 0);
	}

	function weightPercent(w: number): string {
		const total = totalWeight();
		if (total === 0) return '0%';
		return `${Math.round((w / total) * 100)}%`;
	}

	// --- Tag editing helpers ---
	function removeTag(beatIdx: number, field: 'characters' | 'props', tagIdx: number) {
		const beat = editableBeats[beatIdx];
		if (!beat) return;
		const list = beat[field];
		if (list) {
			beat[field] = list.filter((_, i) => i !== tagIdx);
		}
	}

	function addTag(beatIdx: number, field: 'characters' | 'props', event: KeyboardEvent) {
		if (event.key !== 'Enter') return;
		const input = event.target as HTMLInputElement;
		const value = input.value.trim();
		if (!value) return;

		const beat = editableBeats[beatIdx];
		if (!beat) return;
		const list = beat[field] ?? [];
		if (!list.includes(value)) {
			beat[field] = [...list, value];
		}
		input.value = '';
	}

	function clearLocation(beatIdx: number) {
		const beat = editableBeats[beatIdx];
		if (beat) beat.location = null;
	}
</script>

<div class="beat-plan-editor">
	<div class="plan-header">
		<h4 class="plan-title">Beat Plan</h4>
		<span class="beat-count">{editableBeats.length} beats</span>
	</div>

	<div class="plan-preview">
		{#each editableBeats as beat, i}
			<div
				class="preview-segment"
				style="flex: {beat.weight}"
				title="{beat.name} ({weightPercent(beat.weight)})"
			>
				<span class="preview-label">{i + 1}</span>
			</div>
		{/each}
	</div>

	<div class="beats-list">
		{#each editableBeats as beat, i}
			<div class="beat-item">
				<div class="beat-item-header">
					<span class="beat-number">{i + 1}</span>
					<input
						class="beat-name-input"
						type="text"
						bind:value={beat.name}
						placeholder="Beat name"
					/>
					<select
						class="beat-type-select"
						bind:value={beat.beat_type}
					>
						{#each beatTypes as bt}
							<option value={bt}>{beatTypeLabel(bt)}</option>
						{/each}
					</select>
					<button class="beat-remove-btn" title="Remove beat" onclick={() => removeBeat(i)}>&times;</button>
				</div>
				<textarea
					class="beat-outline-input"
					bind:value={beat.outline}
					placeholder="Beat outline..."
					rows="2"
				></textarea>

				<!-- Entity tags -->
				<div class="entity-tags">
					{#if beat.location}
						<div class="tag-row">
							<span class="tag-label">LOC</span>
							<span class="tag location-tag">
								{beat.location}
								<button class="tag-remove" onclick={() => clearLocation(i)}>&times;</button>
							</span>
						</div>
					{/if}

					{#if (beat.characters && beat.characters.length > 0) || true}
						<div class="tag-row">
							<span class="tag-label">CHR</span>
							<div class="tag-list">
								{#each beat.characters ?? [] as char, ci}
									<span class="tag character-tag">
										{char}
										<button class="tag-remove" onclick={() => removeTag(i, 'characters', ci)}>&times;</button>
									</span>
								{/each}
								<input
									class="tag-add-input"
									type="text"
									placeholder="+ character"
									onkeydown={(e) => addTag(i, 'characters', e)}
								/>
							</div>
						</div>
					{/if}

					{#if (beat.props && beat.props.length > 0) || true}
						<div class="tag-row">
							<span class="tag-label">PROP</span>
							<div class="tag-list">
								{#each beat.props ?? [] as prop, pi}
									<span class="tag prop-tag">
										{prop}
										<button class="tag-remove" onclick={() => removeTag(i, 'props', pi)}>&times;</button>
									</span>
								{/each}
								<input
									class="tag-add-input"
									type="text"
									placeholder="+ prop"
									onkeydown={(e) => addTag(i, 'props', e)}
								/>
							</div>
						</div>
					{/if}
				</div>

				<div class="beat-weight-row">
					<label class="weight-label">Weight</label>
					<input
						class="weight-input"
						type="range"
						min="0.5"
						max="3"
						step="0.25"
						bind:value={beat.weight}
					/>
					<span class="weight-value">{beat.weight.toFixed(2)} ({weightPercent(beat.weight)})</span>
				</div>
			</div>
		{/each}
	</div>

	<button class="add-beat-btn" onclick={addBeat}>+ Add Beat</button>

	<div class="plan-actions">
		<button class="cancel-btn" onclick={oncancel}>Cancel</button>
		<button
			class="apply-btn"
			onclick={() => onapply(editableBeats)}
			disabled={editableBeats.length === 0}
		>
			Apply Beats
		</button>
	</div>
</div>

<style>
	.beat-plan-editor {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.plan-header {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.plan-title {
		margin: 0;
		font-size: 0.9rem;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.beat-count {
		font-size: 0.7rem;
		color: var(--color-text-muted);
	}

	.plan-preview {
		display: flex;
		height: 20px;
		gap: 1px;
		border-radius: 4px;
		overflow: hidden;
		background: var(--color-border-subtle);
	}

	.preview-segment {
		background: var(--color-accent);
		opacity: 0.6;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 0;
		transition: opacity 0.15s;
	}

	.preview-segment:hover {
		opacity: 1;
	}

	.preview-label {
		font-size: 0.55rem;
		color: var(--color-text-on-dark);
		font-weight: 600;
	}

	.beats-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-height: 400px;
		overflow-y: auto;
	}

	.beat-item {
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		padding: 6px 8px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.beat-item-header {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.beat-number {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		font-weight: 600;
		width: 16px;
		text-align: center;
		flex-shrink: 0;
	}

	.beat-name-input {
		flex: 1;
		padding: 2px 6px;
		font-size: 0.8rem;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		min-width: 0;
	}

	.beat-name-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.beat-type-select {
		padding: 2px 4px;
		font-size: 0.7rem;
		background: var(--color-bg-primary);
		color: var(--color-text-secondary);
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		flex-shrink: 0;
	}

	.beat-remove-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 0.9rem;
		cursor: pointer;
		padding: 0 2px;
		line-height: 1;
		flex-shrink: 0;
	}

	.beat-remove-btn:hover {
		color: var(--color-danger);
	}

	.beat-outline-input {
		width: 100%;
		padding: 4px 6px;
		font-size: 0.75rem;
		background: var(--color-bg-primary);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 3px;
		font-family: inherit;
		resize: vertical;
	}

	.beat-outline-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	/* Entity tags */
	.entity-tags {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.tag-row {
		display: flex;
		align-items: center;
		gap: 4px;
		flex-wrap: wrap;
	}

	.tag-label {
		font-size: 0.55rem;
		font-weight: 700;
		color: var(--color-text-muted);
		width: 28px;
		flex-shrink: 0;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.tag-list {
		display: flex;
		align-items: center;
		gap: 3px;
		flex-wrap: wrap;
	}

	.tag {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 1px 5px;
		border-radius: 3px;
		font-size: 0.65rem;
		font-weight: 500;
		white-space: nowrap;
	}

	.character-tag {
		background: rgba(100, 149, 237, 0.15);
		color: rgb(100, 149, 237);
		border: 1px solid rgba(100, 149, 237, 0.3);
	}

	.location-tag {
		background: rgba(34, 197, 94, 0.15);
		color: rgb(34, 197, 94);
		border: 1px solid rgba(34, 197, 94, 0.3);
	}

	.prop-tag {
		background: rgba(249, 115, 22, 0.15);
		color: rgb(249, 115, 22);
		border: 1px solid rgba(249, 115, 22, 0.3);
	}

	.tag-remove {
		background: none;
		border: none;
		color: inherit;
		font-size: 0.7rem;
		cursor: pointer;
		padding: 0;
		line-height: 1;
		opacity: 0.6;
	}

	.tag-remove:hover {
		opacity: 1;
	}

	.tag-add-input {
		padding: 1px 4px;
		font-size: 0.6rem;
		background: transparent;
		color: var(--color-text-muted);
		border: 1px dashed var(--color-border-default);
		border-radius: 3px;
		width: 80px;
		min-width: 60px;
	}

	.tag-add-input:focus {
		outline: none;
		border-color: var(--color-accent);
		color: var(--color-text-primary);
		border-style: solid;
	}

	.tag-add-input::placeholder {
		color: var(--color-text-muted);
		opacity: 0.6;
	}

	.beat-weight-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.weight-label {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.weight-input {
		flex: 1;
		height: 4px;
		accent-color: var(--color-accent);
	}

	.weight-value {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		min-width: 60px;
		text-align: right;
	}

	.add-beat-btn {
		padding: 4px 8px;
		font-size: 0.75rem;
		background: none;
		border: 1px dashed var(--color-border-default);
		border-radius: 4px;
		color: var(--color-text-muted);
		cursor: pointer;
	}

	.add-beat-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-secondary);
	}

	.plan-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		padding-top: 4px;
	}

	.cancel-btn {
		font-size: 0.75rem;
		padding: 4px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.cancel-btn:hover {
		background: var(--color-bg-hover);
	}

	.apply-btn {
		font-size: 0.75rem;
		padding: 4px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-accent);
		background: var(--color-bg-surface);
		color: var(--color-accent);
		cursor: pointer;
	}

	.apply-btn:hover:not(:disabled) {
		background: var(--color-accent);
		color: var(--color-text-on-dark);
	}

	.apply-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
