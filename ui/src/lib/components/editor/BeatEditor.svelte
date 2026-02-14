<script lang="ts">
	import {
		editorState,
		startGeneration,
		removeConsistencySuggestion,
		clearConsistencySuggestions,
	} from '$lib/stores/editor.svelte.js';
	import {
		updateBeatNotes,
		updateBeatScript,
		lockBeat,
		unlockBeat,
		getBeat,
		generateScript,
		reactToEdit,
	} from '$lib/api.js';
	import ScriptView from './ScriptView.svelte';
	import DiffView from './DiffView.svelte';

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;
	let scriptDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	let editing = $state(false);
	let editingText = $state('');
	let previousScript = $state('');

	let isGenerating = $derived(
		editorState.streamingClipId != null &&
		editorState.streamingClipId === editorState.selectedClipId
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
				{isGenerating ? 'Generating...' : 'Generate'}
			</button>
		</div>

		<div class="editor-body">
			<label class="section-label">Beat Notes</label>
			<textarea
				class="notes-input"
				placeholder="Describe what happens in this beat..."
				value={clip.content.beat_notes}
				oninput={handleNotesInput}
				disabled={clip.locked}
			></textarea>

			{#if isGenerating}
				<label class="section-label">
					Generating Script
					<span class="token-count">{editorState.streamingTokenCount} tokens</span>
				</label>
				<ScriptView text={editorState.streamingText} streaming={true} />
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
				<ScriptView text={clip.content.user_refined_script} />
			{:else if clip.content.generated_script}
				<label class="section-label">
					Generated Script
					{#if !clip.locked}
						<button class="edit-btn" onclick={startEditing}>Edit</button>
					{/if}
				</label>
				<ScriptView text={clip.content.generated_script} />
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

	.status-badge[data-status="NotesOnly"] { background: var(--color-status-notes); color: #000; }
	.status-badge[data-status="Generating"] { background: var(--color-status-generating); color: #fff; }
	.status-badge[data-status="Generated"] { background: var(--color-status-generated); color: #000; }
	.status-badge[data-status="UserRefined"] { background: var(--color-status-written); color: #fff; }
	.status-badge[data-status="UserWritten"] { background: var(--color-status-written); color: #fff; }

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
		color: #fff;
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
		background: rgba(239, 68, 68, 0.15);
		border: 1px solid rgba(239, 68, 68, 0.4);
		border-radius: 4px;
		color: #f87171;
		font-size: 0.85rem;
	}

	.empty-state {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted);
	}
</style>
