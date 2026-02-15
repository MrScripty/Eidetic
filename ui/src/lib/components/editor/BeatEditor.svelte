<script lang="ts">
	import type { ExtractionResult } from '$lib/types.js';
	import { colorToHex } from '$lib/types.js';
	import {
		editorState,
		startGeneration,
		startBatchGeneration,
		setBatchTotalCount,
		removeConsistencySuggestion,
		clearConsistencySuggestions,
	} from '$lib/stores/editor.svelte.js';
	import { storyState, entitiesForClip } from '$lib/stores/story.svelte.js';
	import { timelineState, zoomToRange } from '$lib/stores/timeline.svelte.js';
	import {
		updateBeatNotes,
		updateBeatScript,
		lockBeat,
		unlockBeat,
		getBeat,
		generateScript,
		reactToEdit,
		extractEntities,
		removeClipRef,
		planBeats,
		generateBeats,
		applyBeats,
	} from '$lib/api.js';
	import ScriptView from './ScriptView.svelte';
	import DiffView from './DiffView.svelte';
	import EntityExtractPanel from '../sidebar/bible/EntityExtractPanel.svelte';

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let scriptDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	let editing = $state(false);
	let editingText = $state('');
	let previousScript = $state('');

	let isGenerating = $derived(
		(editorState.streamingClipId != null &&
		editorState.streamingClipId === editorState.selectedClipId) ||
		(editorState.batchParentClipId != null &&
		editorState.batchParentClipId === editorState.selectedClipId)
	);

	// Beat planning state
	let planning = $state(false);

	// Track and sub-beat context
	let selectedTrack = $derived.by(() => {
		if (!editorState.selectedClipId || !timelineState.timeline) return null;
		return timelineState.timeline.tracks.find(t =>
			t.clips.some(c => c.id === editorState.selectedClipId) ||
			t.sub_beats.some(b => b.id === editorState.selectedClipId)
		) ?? null;
	});

	let isSubBeat = $derived(editorState.selectedClip?.parent_clip_id != null);

	let hasSubBeats = $derived.by(() => {
		if (!selectedTrack || !editorState.selectedClipId) return false;
		return selectedTrack.sub_beats.some(b => b.parent_clip_id === editorState.selectedClipId);
	});

	let parentClip = $derived.by(() => {
		if (!isSubBeat || !editorState.selectedClip || !selectedTrack) return null;
		return selectedTrack.clips.find(c => c.id === editorState.selectedClip!.parent_clip_id) ?? null;
	});

	let siblingBeats = $derived.by(() => {
		if (!isSubBeat || !editorState.selectedClip || !selectedTrack) return [];
		return selectedTrack.sub_beats
			.filter(b => b.parent_clip_id === editorState.selectedClip!.parent_clip_id)
			.sort((a, b) => a.time_range.start_ms - b.time_range.start_ms);
	});

	let currentBeatIndex = $derived(
		siblingBeats.findIndex(b => b.id === editorState.selectedClipId)
	);

	// Exit editing when switching clips.
	$effect(() => {
		editorState.selectedClipId;
		if (editing) finishEditing();
	});

	function statusLabel(status: string): string {
		switch (status) {
			case 'Empty': return 'No content';
			case 'NotesOnly': return 'Notes written';
			case 'Generating': return 'AI generating...';
			case 'Generated': return 'AI generated';
			case 'UserRefined': return 'User refined';
			case 'UserWritten': return 'User written';
			default: return status;
		}
	}

	function handleNotesInput(e: Event) {
		const value = (e.target as HTMLTextAreaElement).value;
		if (editorState.selectedClip) {
			editorState.selectedClip.content.beat_notes = value;
		}
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(async () => {
			if (editorState.selectedClipId) {
				await updateBeatNotes(editorState.selectedClipId, value);
			}
		}, 500);
	}

	function startEditing() {
		if (!editorState.selectedClip || editorState.selectedClip.locked || isGenerating) return;
		const clip = editorState.selectedClip;
		editingText = clip.content.user_refined_script ?? clip.content.generated_script ?? '';
		previousScript = editingText;
		editing = true;
	}

	function handleScriptInput(e: Event) {
		editingText = (e.target as HTMLTextAreaElement).value;
		if (scriptDebounceTimer) clearTimeout(scriptDebounceTimer);
		scriptDebounceTimer = setTimeout(async () => {
			if (editorState.selectedClipId) {
				await updateBeatScript(editorState.selectedClipId, editingText);
			}
		}, 800);
	}

	async function finishEditing() {
		if (!editing) return;
		editing = false;
		// Flush any pending debounced save.
		if (scriptDebounceTimer) {
			clearTimeout(scriptDebounceTimer);
			scriptDebounceTimer = null;
		}
		if (editorState.selectedClipId && editingText !== previousScript) {
			await updateBeatScript(editorState.selectedClipId, editingText);
			// Trigger consistency check.
			editorState.checkingConsistency = true;
			clearConsistencySuggestions();
			editorState.checkingConsistency = true;
			await reactToEdit(editorState.selectedClipId);
		}
	}

	async function handleToggleLock() {
		if (!editorState.selectedClipId || !editorState.selectedClip) return;
		if (editing) await finishEditing();
		if (editorState.selectedClip.locked) {
			await unlockBeat(editorState.selectedClipId);
			editorState.selectedClip.locked = false;
		} else {
			await lockBeat(editorState.selectedClipId);
			editorState.selectedClip.locked = true;
		}
		const content = await getBeat(editorState.selectedClipId) as typeof editorState.selectedClip.content;
		if (editorState.selectedClip) {
			editorState.selectedClip.content = content;
		}
	}

	async function handleGenerate() {
		if (!editorState.selectedClipId || !editorState.selectedClip) return;
		if (editorState.selectedClip.locked) return;
		if (!editorState.selectedClip.content.beat_notes.trim()) return;
		if (isGenerating) return;

		// If this scene clip has sub-beats, generate all beats instead.
		if (hasSubBeats) {
			startBatchGeneration(editorState.selectedClipId);
			const result = await generateBeats(editorState.selectedClipId);
			setBatchTotalCount(result.beat_count);
			return;
		}

		startGeneration(editorState.selectedClipId);
		editorState.selectedClip.content.status = 'Generating';
		await generateScript(editorState.selectedClipId);
	}

	async function handleAcceptSuggestion(targetClipId: string, suggestedText: string) {
		await updateBeatScript(targetClipId, suggestedText);
		removeConsistencySuggestion(targetClipId);
	}

	function handleRejectSuggestion(targetClipId: string) {
		removeConsistencySuggestion(targetClipId);
	}

	// Entity extraction
	let extracting = $state(false);
	let extractionResult: ExtractionResult | null = $state(null);

	const linkedEntities = $derived(
		editorState.selectedClipId ? entitiesForClip(editorState.selectedClipId) : []
	);

	async function handleExtract() {
		if (!editorState.selectedClipId) return;
		extracting = true;
		extractionResult = null;
		try {
			extractionResult = await extractEntities(editorState.selectedClipId);
		} finally {
			extracting = false;
		}
	}

	async function handleUnlinkEntity(entityId: string) {
		if (!editorState.selectedClipId) return;
		await removeClipRef(entityId, editorState.selectedClipId);
	}

	// Beat planning — applies directly to timeline and zooms to fit
	async function handlePlanBeats() {
		if (!editorState.selectedClipId || !editorState.selectedClip) return;
		planning = true;
		try {
			const plan = await planBeats(editorState.selectedClipId);
			await applyBeats(editorState.selectedClipId, plan.beats);
			// Zoom to the parent clip's time range so beats are visible
			const clip = editorState.selectedClip;
			zoomToRange(clip.time_range.start_ms, clip.time_range.end_ms);
		} finally {
			planning = false;
		}
	}

	function navigateBeat(direction: -1 | 1) {
		const targetIdx = currentBeatIndex + direction;
		if (targetIdx < 0 || targetIdx >= siblingBeats.length) return;
		const target = siblingBeats[targetIdx]!;
		editorState.selectedClipId = target.id;
		editorState.selectedClip = target;
	}
</script>

<div class="beat-editor">
	{#if editorState.selectedClip}
		{@const clip = editorState.selectedClip}
		<div class="editor-header">
			<h3 class="clip-title">{clip.name}</h3>
			<span class="status-badge" data-status={clip.content.status}>
				{statusLabel(clip.content.status)}
			</span>
			<button class="lock-toggle" class:locked={clip.locked} onclick={handleToggleLock}>
				{clip.locked ? 'Unlock' : 'Lock'}
			</button>
			<button
				class="generate-btn"
				class:generating={isGenerating}
				onclick={handleGenerate}
				disabled={!clip.content.beat_notes.trim() || clip.locked || isGenerating}
			>
				{#if isGenerating}
					Generating...
				{:else if hasSubBeats}
					Generate Beats
				{:else}
					Generate
				{/if}
			</button>
		</div>

		<!-- Sub-beat: parent context and navigation -->
		{#if isSubBeat && parentClip}
			<div class="sub-beat-context">
				<div class="sub-beat-nav">
					<button
						class="nav-btn"
						disabled={currentBeatIndex <= 0}
						onclick={() => navigateBeat(-1)}
					>&lsaquo; Prev</button>
					<span class="nav-position">Beat {currentBeatIndex + 1} of {siblingBeats.length}</span>
					<button
						class="nav-btn"
						disabled={currentBeatIndex >= siblingBeats.length - 1}
						onclick={() => navigateBeat(1)}
					>Next &rsaquo;</button>
				</div>
				<details class="parent-context">
					<summary class="parent-summary">Scene: {parentClip.name}</summary>
					<p class="parent-notes">{parentClip.content.beat_notes || 'No scene notes'}</p>
				</details>
			</div>
		{/if}

		<!-- Scene clip: beat planning actions -->
		{#if !isSubBeat}
			<div class="beat-actions">
				<button
					class="plan-btn"
					onclick={handlePlanBeats}
					disabled={planning || !clip.content.beat_notes.trim()}
				>
					{planning ? 'Planning...' : hasSubBeats ? 'Replan Beats' : 'Plan Beats'}
				</button>
			</div>
		{/if}

		<div class="editor-body">
			<label class="section-label">Beat Notes</label>
			<textarea
				class="notes-input"
				placeholder="Describe what happens in this beat..."
				value={clip.content.beat_notes}
				oninput={handleNotesInput}
				disabled={clip.locked}
			></textarea>

			<!-- Linked entities -->
			<div class="entity-chips-section">
				<label class="section-label">
					Entities
					<button
						class="extract-btn"
						disabled={extracting || !clip.content.generated_script}
						onclick={handleExtract}
					>
						{extracting ? 'Extracting...' : 'Extract'}
					</button>
				</label>
				<div class="entity-chips">
					{#each linkedEntities as entity (entity.id)}
						<span class="entity-chip" style="border-color: {colorToHex(entity.color)}">
							{entity.name}
							<button class="chip-remove" onclick={() => handleUnlinkEntity(entity.id)}>&times;</button>
						</span>
					{/each}
					{#if linkedEntities.length === 0}
						<span class="chip-empty">No linked entities</span>
					{/if}
				</div>
			</div>

			{#if extractionResult && editorState.selectedClipId}
				<EntityExtractPanel
					result={extractionResult}
					clipId={editorState.selectedClipId}
					onclose={() => extractionResult = null}
				/>
			{/if}

			{#if isGenerating}
				<label class="section-label">
					AI Context
					<span class="token-count">{editorState.streamingTokenCount} tokens generated</span>
				</label>
				{#if editorState.generationContext}
					<div class="context-display">
						<details open>
							<summary class="context-heading">System Prompt</summary>
							<pre class="context-text">{editorState.generationContext.system}</pre>
						</details>
						<details open>
							<summary class="context-heading">User Prompt</summary>
							<pre class="context-text">{editorState.generationContext.user}</pre>
						</details>
					</div>
				{:else}
					<div class="context-display">
						<p class="context-loading">Building prompt...</p>
					</div>
				{/if}
			{:else if editing}
				<label class="section-label">
					Editing Script
					<button class="done-edit-btn" onclick={finishEditing}>Done Editing</button>
				</label>
				<textarea
					class="script-edit"
					value={editingText}
					oninput={handleScriptInput}
				></textarea>
			{:else if clip.content.user_refined_script}
				<label class="section-label">
					Refined Script
					{#if !clip.locked}
						<button class="edit-btn" onclick={startEditing}>Edit</button>
					{/if}
				</label>
				<ScriptView text={clip.content.user_refined_script} entities={storyState.entities} />
			{:else if clip.content.generated_script}
				<label class="section-label">
					Generated Script
					{#if !clip.locked}
						<button class="edit-btn" onclick={startEditing}>Edit</button>
					{/if}
				</label>
				<ScriptView text={clip.content.generated_script} entities={storyState.entities} />
			{/if}

			{#if editorState.generationError}
				<div class="error-banner">{editorState.generationError}</div>
			{/if}

			{#if editorState.checkingConsistency || editorState.consistencySuggestions.length > 0}
				<div class="suggestions-panel">
					<label class="section-label">
						Consistency Suggestions
						{#if editorState.checkingConsistency}
							<span class="checking-spinner">checking...</span>
						{/if}
					</label>
					{#each editorState.consistencySuggestions as suggestion (suggestion.target_clip_id)}
						<DiffView
							{suggestion}
							onaccept={() => handleAcceptSuggestion(suggestion.target_clip_id, suggestion.suggested_text)}
							onreject={() => handleRejectSuggestion(suggestion.target_clip_id)}
						/>
					{/each}
				</div>
			{/if}
		</div>
	{:else}
		<div class="empty-state">
			<p>Select a beat clip on the timeline to edit</p>
		</div>
	{/if}
</div>

<style>
	.beat-editor {
		height: 100%;
		display: flex;
		flex-direction: column;
		padding: 12px 16px;
		overflow: auto;
	}

	.editor-header {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-bottom: 12px;
		flex-shrink: 0;
	}

	.clip-title {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.status-badge {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 10px;
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
	}

	.status-badge[data-status="NotesOnly"] { background: var(--color-status-notes); color: var(--color-text-on-light); }
	.status-badge[data-status="Generating"] { background: var(--color-status-generating); color: var(--color-text-on-dark); }
	.status-badge[data-status="Generated"] { background: var(--color-status-generated); color: var(--color-text-on-light); }
	.status-badge[data-status="UserRefined"] { background: var(--color-status-written); color: var(--color-text-on-dark); }
	.status-badge[data-status="UserWritten"] { background: var(--color-status-written); color: var(--color-text-on-dark); }

	.lock-toggle {
		font-size: 0.75rem;
		padding: 2px 10px;
		border-radius: 10px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.lock-toggle.locked {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.generate-btn {
		font-size: 0.75rem;
		padding: 2px 10px;
		border-radius: 10px;
		border: 1px solid var(--color-accent);
		background: var(--color-bg-surface);
		color: var(--color-accent);
		cursor: pointer;
		margin-left: auto;
		transition: background 0.15s, color 0.15s;
	}

	.generate-btn:hover:not(:disabled) {
		background: var(--color-accent);
		color: var(--color-text-on-dark);
	}

	.generate-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.generate-btn.generating {
		animation: pulse 1.5s ease-in-out infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}

	.editor-body {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.token-count {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		text-transform: none;
		letter-spacing: normal;
	}

	.context-display {
		flex: 1;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-height: 0;
	}

	.context-heading {
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		cursor: pointer;
		user-select: none;
	}

	.context-text {
		font-family: 'Courier New', monospace;
		font-size: 0.75rem;
		line-height: 1.4;
		color: var(--color-text-secondary);
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		padding: 8px 12px;
		margin: 0;
		white-space: pre-wrap;
		word-wrap: break-word;
		overflow: auto;
		max-height: 300px;
	}

	.context-loading {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		font-style: italic;
		margin: 8px 0;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.notes-input {
		width: 100%;
		min-height: 100px;
		padding: 8px 12px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-family: inherit;
		font-size: 0.9rem;
		resize: vertical;
	}

	.notes-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.notes-input:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.script-edit {
		width: 100%;
		min-height: 200px;
		padding: 8px 12px;
		background: var(--color-bg-surface);
		color: var(--color-text-primary);
		border: 1px solid var(--color-accent);
		border-radius: 4px;
		font-family: 'Courier New', monospace;
		font-size: 0.85rem;
		line-height: 1.4;
		resize: vertical;
	}

	.script-edit:focus {
		outline: none;
		border-color: var(--color-accent);
		box-shadow: 0 0 0 1px var(--color-accent);
	}

	.edit-btn,
	.done-edit-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.edit-btn:hover,
	.done-edit-btn:hover {
		background: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.done-edit-btn {
		border-color: var(--color-accent);
		color: var(--color-accent);
	}

	.suggestions-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 4px;
	}

	.checking-spinner {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		text-transform: none;
		letter-spacing: normal;
		animation: pulse 1.5s ease-in-out infinite;
	}

	.error-banner {
		padding: 8px 12px;
		background: var(--color-danger-bg);
		border: 1px solid var(--color-danger-border);
		border-radius: 4px;
		color: var(--color-danger-light);
		font-size: 0.85rem;
	}

	.entity-chips-section {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.entity-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}

	.entity-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		font-size: 0.75rem;
		color: var(--color-text-primary);
		background: var(--color-bg-surface);
		border: 1px solid;
		border-radius: 12px;
		cursor: default;
	}

	.chip-remove {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		font-size: 0.8rem;
		padding: 0;
		line-height: 1;
	}

	.chip-remove:hover {
		color: var(--color-danger);
	}

	.chip-empty {
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}

	.extract-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-bible-development);
		background: var(--color-bg-surface);
		color: var(--color-bible-development);
		cursor: pointer;
	}

	.extract-btn:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.extract-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.sub-beat-context {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 8px;
	}

	.sub-beat-nav {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.nav-btn {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 4px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.nav-btn:hover:not(:disabled) {
		background: var(--color-bg-hover);
	}

	.nav-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.nav-position {
		font-size: 0.7rem;
		color: var(--color-text-muted);
	}

	.parent-context {
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		padding: 4px 8px;
	}

	.parent-summary {
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--color-text-secondary);
		cursor: pointer;
		user-select: none;
	}

	.parent-notes {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		margin: 4px 0 0;
		white-space: pre-wrap;
	}

	.beat-actions {
		display: flex;
		gap: 8px;
		margin-bottom: 8px;
	}

	.plan-btn {
		font-size: 0.75rem;
		padding: 3px 12px;
		border-radius: 10px;
		border: 1px solid var(--color-accent);
		background: var(--color-bg-surface);
		color: var(--color-accent);
		cursor: pointer;
	}

	.plan-btn:hover:not(:disabled) {
		background: var(--color-accent);
		color: var(--color-text-on-dark);
	}

	.plan-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.empty-state {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted);
	}
</style>
