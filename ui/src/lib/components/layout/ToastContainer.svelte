<script lang="ts">
	import { notifications, dismiss } from '$lib/stores/notifications.svelte.js';
</script>

{#if notifications.items.length > 0}
	<div class="toast-container">
		{#each notifications.items as toast (toast.id)}
			<div class="toast toast-{toast.type}" role="alert">
				<span class="toast-message">{toast.message}</span>
				<button class="toast-dismiss" onclick={() => dismiss(toast.id)}>&#x2715;</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.toast-container {
		position: fixed;
		bottom: 40px;
		right: 16px;
		display: flex;
		flex-direction: column-reverse;
		gap: 8px;
		z-index: 100;
		pointer-events: none;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-radius: 6px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		color: var(--color-text-primary);
		font-size: 0.8rem;
		pointer-events: auto;
		min-width: 200px;
		max-width: 360px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
	}

	.toast-info {
		border-left: 3px solid #3b82f6;
	}
	.toast-success {
		border-left: 3px solid #22c55e;
	}
	.toast-warning {
		border-left: 3px solid #f59e0b;
	}
	.toast-error {
		border-left: 3px solid #ef4444;
	}

	.toast-message {
		flex: 1;
	}

	.toast-dismiss {
		background: none;
		border: none;
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0 2px;
		font-size: 0.75rem;
		line-height: 1;
	}

	.toast-dismiss:hover {
		color: var(--color-text-primary);
	}
</style>
