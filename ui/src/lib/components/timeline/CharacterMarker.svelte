<script lang="ts">
	import { timeToX } from '$lib/stores/timeline.svelte.js';
	import { timelineState, findNode } from '$lib/stores/timeline.svelte.js';
	import { editorState } from '$lib/stores/editor.svelte.js';
	import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';
	import { formatTime } from '$lib/types.js';

	export type MarkerKind = 'snapshot' | 'event' | 'mention';

	export interface ProgressionMarker {
		id: string;
		kind: MarkerKind;
		timeMs: number;
		nodeId: string | null;
		label: string;
		detail: string;
	}

	let { marker }: { marker: ProgressionMarker } = $props();

	let hovered = $derived(characterTimelineState.hoveredMarkerId === marker.id);

	function handleClick() {
		if (marker.nodeId && timelineState.timeline) {
			editorState.selectedNodeId = marker.nodeId;
			const node = findNode(marker.nodeId);
			if (node) {
				editorState.selectedNode = node;
				editorState.selectedLevel = node.level;
			}
		} else {
			timelineState.playheadMs = marker.timeMs;
		}
	}

	function handlePointerEnter() {
		characterTimelineState.hoveredMarkerId = marker.id;
	}

	function handlePointerLeave() {
		if (characterTimelineState.hoveredMarkerId === marker.id) {
			characterTimelineState.hoveredMarkerId = null;
		}
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="marker marker-{marker.kind}"
	class:hovered
	style="left: {timeToX(marker.timeMs)}px"
	onclick={handleClick}
	onpointerenter={handlePointerEnter}
	onpointerleave={handlePointerLeave}
	title="{marker.label}{marker.detail ? ' — ' + marker.detail : ''} [{formatTime(marker.timeMs)}]"
>
	<div class="marker-shape"></div>
</div>

<style>
	.marker {
		position: absolute;
		top: 50%;
		transform: translate(-50%, -50%);
		cursor: pointer;
		z-index: 1;
	}

	.marker:hover {
		z-index: 3;
	}

	/* Snapshot — diamond */
	.marker-snapshot .marker-shape {
		width: 10px;
		height: 10px;
		background: var(--color-marker-snapshot);
		transform: rotate(45deg);
		opacity: 0.85;
		transition: opacity 0.15s, transform 0.15s;
	}

	.marker-snapshot:hover .marker-shape,
	.marker-snapshot.hovered .marker-shape {
		opacity: 1;
		transform: rotate(45deg) scale(1.3);
	}

	/* Event — triangle */
	.marker-event .marker-shape {
		width: 0;
		height: 0;
		border-left: 6px solid transparent;
		border-right: 6px solid transparent;
		border-bottom: 11px solid var(--color-marker-event);
		opacity: 0.85;
		transition: opacity 0.15s, transform 0.15s;
	}

	.marker-event:hover .marker-shape,
	.marker-event.hovered .marker-shape {
		opacity: 1;
		transform: scale(1.3);
	}

	/* Mention — circle */
	.marker-mention .marker-shape {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-marker-mention);
		opacity: 0.85;
		transition: opacity 0.15s, transform 0.15s;
	}

	.marker-mention:hover .marker-shape,
	.marker-mention.hovered .marker-shape {
		opacity: 1;
		transform: scale(1.3);
	}
</style>
