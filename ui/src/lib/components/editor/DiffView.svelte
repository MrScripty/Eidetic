<script lang="ts">
	import type { ConsistencySuggestion } from '$lib/types.js';

	let { suggestion, onaccept, onreject }: {
		suggestion: ConsistencySuggestion;
		onaccept: () => void;
		onreject: () => void;
	} = $props();
</script>

<div class="diff-view">
	<div class="diff-header">
		<span class="diff-target">Target: {suggestion.target_node_id.slice(0, 8)}...</span>
	</div>

	<div class="diff-body">
		<div class="diff-line removed">{suggestion.original_text}</div>
		<div class="diff-line added">{suggestion.suggested_text}</div>
	</div>

	<div class="diff-reason">{suggestion.reason}</div>

	<div class="diff-actions">
		<button class="accept-btn" onclick={onaccept}>Accept</button>
		<button class="reject-btn" onclick={onreject}>Reject</button>
	</div>
</div>

<style>
	.diff-view {
		border: 1px solid var(--color-border-default);
		border-radius: 6px;
		padding: 10px;
		background: var(--color-bg-surface);
	}

	.diff-header {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		margin-bottom: 6px;
	}

	.diff-body {
		font-family: 'Courier New', monospace;
		font-size: 0.85rem;
		line-height: 1.4;
		margin-bottom: 6px;
	}

	.diff-line {
		padding: 2px 6px;
		border-radius: 2px;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.diff-line.removed {
		background: var(--color-danger-bg);
		color: var(--color-danger-light);
		text-decoration: line-through;
	}

	.diff-line.added {
		background: var(--color-success-bg);
		color: var(--color-success-light);
	}

	.diff-reason {
		font-size: 0.8rem;
		color: var(--color-text-secondary);
		font-style: italic;
		margin-bottom: 8px;
	}

	.diff-actions {
		display: flex;
		gap: 8px;
	}

	.diff-actions button {
		font-size: 0.75rem;
		padding: 3px 12px;
		border-radius: 4px;
		border: 1px solid var(--color-border-default);
		cursor: pointer;
	}

	.accept-btn {
		background: var(--color-success-bg-strong);
		color: var(--color-success-light);
		border-color: var(--color-success-border) !important;
	}

	.accept-btn:hover {
		background: var(--color-success-bg-hover);
	}

	.reject-btn {
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
	}

	.reject-btn:hover {
		background: var(--color-bg-hover);
	}
</style>
