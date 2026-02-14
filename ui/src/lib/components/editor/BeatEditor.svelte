<script lang="ts">
	import { editorState } from '$lib/stores/editor.svelte.js';

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
</script>

<div class="beat-editor">
	{#if editorState.selectedClip}
		{@const clip = editorState.selectedClip}
		<div class="editor-header">
			<h3 class="clip-title">{clip.name}</h3>
			<span class="status-badge" data-status={clip.content.status}>
				{statusLabel(clip.content.status)}
			</span>
			{#if clip.locked}
				<span class="lock-badge">Locked</span>
			{/if}
		</div>

		<div class="editor-body">
			<label class="section-label">Beat Notes</label>
			<textarea
				class="notes-input"
				placeholder="Describe what happens in this beat..."
				value={clip.content.beat_notes}
			></textarea>

			{#if clip.content.generated_script}
				<label class="section-label">Generated Script</label>
				<pre class="script-display">{clip.content.generated_script}</pre>
			{/if}

			{#if clip.content.user_refined_script}
				<label class="section-label">Refined Script</label>
				<pre class="script-display">{clip.content.user_refined_script}</pre>
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
	.status-badge[data-status="UserWritten"] { background: var(--color-status-written); color: #fff; }

	.lock-badge {
		font-size: 0.7rem;
		padding: 2px 8px;
		border-radius: 10px;
		background: var(--color-bg-hover);
		color: var(--color-text-muted);
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

	.script-display {
		padding: 8px 12px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-subtle);
		border-radius: 4px;
		font-family: 'Courier New', monospace;
		font-size: 0.85rem;
		white-space: pre-wrap;
		color: var(--color-text-primary);
		overflow: auto;
		margin: 0;
	}

	.empty-state {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted);
	}
</style>
