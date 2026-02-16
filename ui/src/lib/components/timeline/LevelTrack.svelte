<script lang="ts">
	import type { Track, StoryNode, TimelineGap } from '$lib/types.js';
	import { TIMELINE, colorToHex } from '$lib/types.js';
	import { xToTime, timeToX, timelineState, nodesAtLevel, arcsForNode } from '$lib/stores/timeline.svelte.js';
	import { editorState, startGeneration } from '$lib/stores/editor.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { updateNode, deleteNode, splitNode, createNode, fillGap, generateContent } from '$lib/api.js';
	import { notify } from '$lib/stores/notifications.svelte.js';
	import StoryNodeClip from './StoryNodeClip.svelte';

	let { track, gaps = [], onconnectstart }: {
		track: Track;
		gaps?: TimelineGap[];
		onconnectstart: (nodeId: string, x: number, y: number) => void;
	} = $props();

	// Get nodes at this track's level.
	let levelNodes = $derived(nodesAtLevel(track.level));

	// Viewport-aware node filtering.
	let visibleNodes = $derived.by(() => {
		const vw = timelineState.viewportWidth;
		if (vw <= 0) return levelNodes;
		const sx = timelineState.scrollX;
		return levelNodes.filter((n: StoryNode) => {
			const left = timeToX(n.time_range.start_ms);
			const right = timeToX(n.time_range.end_ms);
			return right >= sx && left <= sx + vw;
		});
	});

	let visibleGaps = $derived.by(() => {
		const vw = timelineState.viewportWidth;
		if (vw <= 0) return gaps;
		const sx = timelineState.scrollX;
		return gaps.filter((g) => {
			const left = timeToX(g.time_range.start_ms);
			const right = timeToX(g.time_range.end_ms);
			return right >= sx && left <= sx + vw;
		});
	});

	let sortedNodes = $derived(
		[...levelNodes].sort((a, b) => a.time_range.start_ms - b.time_range.start_ms)
	);

	function nodeBounds(nodeId: string): { left: number; right: number } {
		const idx = sortedNodes.findIndex(n => n.id === nodeId);
		const left = idx > 0 ? sortedNodes[idx - 1]!.time_range.end_ms : 0;
		const right = idx < sortedNodes.length - 1 ? sortedNodes[idx + 1]!.time_range.start_ms : TIMELINE.DURATION_MS;
		return { left, right };
	}

	function arcColorForNode(nodeId: string): string {
		const arcIds = arcsForNode(nodeId);
		if (arcIds.length === 0) return 'var(--color-rel-default)';
		const arc = storyState.arcs.find(a => a.id === arcIds[0]);
		return arc ? colorToHex(arc.color) : 'var(--color-rel-default)';
	}

	function selectNode(node: StoryNode) {
		editorState.selectedNodeId = node.id;
		editorState.selectedNode = node;
		editorState.selectedLevel = node.level;
	}

	async function handleMove(nodeId: string, startMs: number, endMs: number) {
		await updateNode(nodeId, { start_ms: startMs, end_ms: endMs });
	}

	let regenPromptNodeId: string | null = $state(null);

	async function handleResize(nodeId: string, startMs: number, endMs: number) {
		await updateNode(nodeId, { start_ms: startMs, end_ms: endMs });
		const node = levelNodes.find(n => n.id === nodeId);
		if (node && node.content.content) {
			regenPromptNodeId = nodeId;
		}
	}

	async function handleRegenerate() {
		if (!regenPromptNodeId) return;
		const nodeId = regenPromptNodeId;
		regenPromptNodeId = null;
		startGeneration(nodeId);
		await generateContent(nodeId);
	}

	function dismissRegenPrompt() {
		regenPromptNodeId = null;
	}

	async function handleDelete(nodeId: string) {
		if (editorState.selectedNodeId === nodeId) {
			editorState.selectedNodeId = null;
			editorState.selectedNode = null;
			editorState.selectedLevel = null;
		}
		await deleteNode(nodeId);
	}

	async function handleSplit(nodeId: string, atMs: number) {
		if (editorState.selectedNodeId === nodeId) {
			editorState.selectedNodeId = null;
			editorState.selectedNode = null;
			editorState.selectedLevel = null;
		}
		try {
			await splitNode(nodeId, atMs);
		} catch (e) {
			notify('error', `Split failed: ${e instanceof Error ? e.message : 'unknown error'}`);
		}
	}

	async function handleFillGap(gap: TimelineGap) {
		await fillGap(track.level, gap.time_range.start_ms, gap.time_range.end_ms);
	}

	async function handleDblClick(e: MouseEvent) {
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		const timeMs = xToTime(localX + timelineState.scrollX);
		const defaultDuration = 60_000;
		const startMs = Math.max(0, Math.round(timeMs - defaultDuration / 2));
		const endMs = Math.min(startMs + defaultDuration, TIMELINE.DURATION_MS);
		await createNode({
			level: track.level,
			name: `New ${track.level}`,
			beat_type: track.level === 'Beat' ? 'setup' : undefined,
			start_ms: startMs,
			end_ms: endMs,
		});
	}
</script>

{#if !track.collapsed}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="level-track" style="height: {TIMELINE.TRACK_HEIGHT_PX}px">
		<div class="track-lane" class:blade-mode={timelineState.activeTool === 'blade'} ondblclick={handleDblClick}>
			{#each visibleNodes as node (node.id)}
				{@const bounds = nodeBounds(node.id)}
				<StoryNodeClip
					{node}
					color={arcColorForNode(node.id)}
					selected={editorState.selectedNodeId === node.id}
					leftBoundMs={bounds.left}
					rightBoundMs={bounds.right}
					onselect={() => selectNode(node)}
					onmove={(s, e) => handleMove(node.id, s, e)}
					onresize={(s, e) => handleResize(node.id, s, e)}
					ondelete={() => handleDelete(node.id)}
					onsplit={(atMs) => handleSplit(node.id, atMs)}
					onconnectstart={onconnectstart}
				/>
			{/each}
			{#each visibleGaps as gap}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="gap-marker"
					style="left: {timeToX(gap.time_range.start_ms)}px; width: {timeToX(gap.time_range.end_ms) - timeToX(gap.time_range.start_ms)}px"
					title="Click to fill gap"
					onclick={() => handleFillGap(gap)}
				>
					<span class="gap-label">+</span>
				</div>
			{/each}
		</div>
		{#if regenPromptNodeId}
			<div class="regen-prompt">
				<span>Duration changed. Regenerate?</span>
				<button class="regen-btn" onclick={handleRegenerate}>Regenerate</button>
				<button class="keep-btn" onclick={dismissRegenPrompt}>Keep</button>
			</div>
		{/if}
	</div>
{/if}

<style>
	.level-track {
		position: relative;
		border-bottom: 1px solid var(--color-border-subtle);
	}

	.track-lane {
		position: relative;
		height: 100%;
	}

	.track-lane.blade-mode {
		cursor: crosshair;
	}

	.gap-marker {
		position: absolute;
		top: 4px;
		bottom: 4px;
		background: var(--color-overlay-faint);
		border: 1px dashed var(--color-border-subtle);
		border-radius: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: background 0.15s;
	}

	.gap-marker:hover {
		background: var(--color-overlay-light);
		border-color: var(--color-border-default);
	}

	.gap-label {
		font-size: 1rem;
		color: var(--color-text-muted);
		pointer-events: none;
	}

	.regen-prompt {
		position: absolute;
		right: 0;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 8px;
		background: var(--color-bg-surface);
		border: 1px solid var(--color-border-default);
		border-radius: 4px;
		font-size: 0.7rem;
		color: var(--color-text-secondary);
		z-index: 5;
		white-space: nowrap;
	}

	.regen-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-accent);
		background: var(--color-bg-surface);
		color: var(--color-accent);
		cursor: pointer;
	}

	.keep-btn {
		font-size: 0.65rem;
		padding: 1px 8px;
		border-radius: 8px;
		border: 1px solid var(--color-border-default);
		background: var(--color-bg-surface);
		color: var(--color-text-secondary);
		cursor: pointer;
	}
</style>
