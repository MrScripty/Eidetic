<script lang="ts">
	import type { ArcProgression } from '$lib/types.js';
	import { getArcProgression } from '$lib/api.js';

	let progressions = $state<ArcProgression[]>([]);
	let loading = $state(false);

	async function refresh() {
		loading = true;
		try {
			progressions = await getArcProgression();
		} catch {
			progressions = [];
		}
		loading = false;
	}

	$effect(() => {
		refresh();
	});
</script>

<div class="progression-panel">
	<div class="progression-header">
		<h3>Arc Analysis</h3>
		<button class="refresh-btn" onclick={refresh} disabled={loading}>
			{loading ? '...' : 'Refresh'}
		</button>
	</div>

	{#each progressions as prog (prog.arc_id)}
		<div class="arc-card">
			<div class="arc-header">
				<span class="arc-name">{prog.arc_name}</span>
				{#if prog.issues.length === 0}
					<span class="badge badge-ok">OK</span>
				{:else if prog.issues.some(i => i.severity === 'Error')}
					<span class="badge badge-error">{prog.issues.length}</span>
				{:else}
					<span class="badge badge-warn">{prog.issues.length}</span>
				{/if}
			</div>
			<div class="arc-stats">
				<span>{prog.beat_count} beats</span>
				<span>{prog.coverage_percent.toFixed(0)}% coverage</span>
				<span class:missing={!prog.has_setup}>{prog.has_setup ? 'Setup' : 'No setup'}</span>
				<span class:missing={!prog.has_resolution}>{prog.has_resolution ? 'Resolution' : 'No resolution'}</span>
			</div>
			{#if prog.issues.length > 0}
				<ul class="issue-list">
					{#each prog.issues as issue}
						<li class="issue issue-{issue.severity.toLowerCase()}">{issue.message}</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/each}

	{#if progressions.length === 0 && !loading}
		<p class="empty-msg">No arcs to analyze.</p>
	{/if}
</div>

<style>
	.progression-panel {
		padding: 8px;
	}

	.progression-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 8px;
	}

	h3 {
		font-size: 0.8rem;
		color: var(--color-text-secondary);
		margin: 0;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.refresh-btn {
		font-size: 0.65rem;
		padding: 2px 8px;
		border-radius: 4px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.refresh-btn:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.arc-card {
		background: var(--color-bg-primary);
		border-radius: 6px;
		padding: 8px;
		margin-bottom: 6px;
	}

	.arc-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 4px;
	}

	.arc-name {
		font-size: 0.8rem;
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.badge {
		font-size: 0.6rem;
		padding: 1px 6px;
		border-radius: 8px;
		font-weight: 600;
	}

	.badge-ok {
		background: var(--color-success-bg);
		color: var(--color-success);
	}

	.badge-warn {
		background: var(--color-warning-bg);
		color: var(--color-warning);
	}

	.badge-error {
		background: var(--color-danger-bg);
		color: var(--color-danger);
	}

	.arc-stats {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		font-size: 0.65rem;
		color: var(--color-text-muted);
		margin-bottom: 4px;
	}

	.missing {
		color: var(--color-warning);
	}

	.issue-list {
		list-style: none;
		padding: 0;
		margin: 4px 0 0;
	}

	.issue {
		font-size: 0.65rem;
		padding: 2px 0;
		padding-left: 8px;
		border-left: 2px solid;
	}

	.issue-warning {
		border-color: var(--color-warning);
		color: var(--color-warning);
	}

	.issue-error {
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.empty-msg {
		color: var(--color-text-muted);
		font-size: 0.7rem;
		text-align: center;
		padding: 16px 0;
		margin: 0;
	}
</style>
