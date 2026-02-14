<script lang="ts">
	import { editorState, startGeneration } from '$lib/stores/editor.svelte.js';
	import { updateBeatNotes, lockBeat, unlockBeat, getBeat, generateScript } from '$lib/api.js';
	import ScriptView from './ScriptView.svelte';

	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	let isGenerating = $derived(
		editorState.streamingClipId != null &&
		editorState.streamingClipId === editorState.selectedClipId
	);

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

	async function handleToggleLock() {
		if (!editorState.selectedClipId || !editorState.selectedClip) return;
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
			{:else if clip.content.generated_script}
				<label class="section-label">Generated Script</label>
				<ScriptView text={clip.content.generated_script} />
			{/if}

			{#if clip.content.user_refined_script}
				<label class="section-label">Refined Script</label>
				<ScriptView text={clip.content.user_refined_script} />
			{/if}

			{#if editorState.generationError}
				<div class="error-banner">{editorState.generationError}</div>
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
