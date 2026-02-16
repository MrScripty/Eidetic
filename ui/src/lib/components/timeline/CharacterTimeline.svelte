<script lang="ts">
	import { timelineState, timeToX, totalWidth, zoomTo, xToTime, findNode } from '$lib/stores/timeline.svelte.js';
	import { storyState } from '$lib/stores/story.svelte.js';
	import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';
	import { TIMELINE, colorToHex, formatTime } from '$lib/types.js';
	import CharacterMarker from './CharacterMarker.svelte';
	import type { ProgressionMarker } from './CharacterMarker.svelte';

	/** All Character entities from the story bible. */
	let characters = $derived(
		storyState.entities.filter(e => e.category === 'Character')
	);

	/** The currently selected character entity. */
	let selectedCharacter = $derived(
		characters.find(e => e.id === characterTimelineState.selectedCharacterId) ?? null
	);

	/** Guard: clear selection if entity was deleted. */
	$effect(() => {
		const id = characterTimelineState.selectedCharacterId;
		if (id && !storyState.entities.some(e => e.id === id)) {
			characterTimelineState.selectedCharacterId = null;
		}
	});

	/** Auto-select first character when timeline is shown with no selection. */
	$effect(() => {
		const first = characters[0];
		if (characterTimelineState.visible && !characterTimelineState.selectedCharacterId && first) {
			characterTimelineState.selectedCharacterId = first.id;
		}
	});

	/** Derive all progression markers for the selected character. */
	let markers = $derived.by((): ProgressionMarker[] => {
		const char = selectedCharacter;
		if (!char) return [];

		const result: ProgressionMarker[] = [];

		// 1. Snapshots — character development moments
		if (characterTimelineState.showSnapshots) {
			char.snapshots.forEach((snap, i) => {
				result.push({
					id: `snap-${char.id}-${i}`,
					kind: 'snapshot',
					timeMs: snap.at_ms,
					nodeId: snap.source_node_id ?? null,
					label: snap.description,
					detail: snap.state_overrides?.emotional_state
						? `Feeling: ${snap.state_overrides.emotional_state}`
						: '',
				});
			});
		}

		// 2. Events involving this character
		if (characterTimelineState.showEvents) {
			for (const entity of storyState.entities) {
				if (
					entity.category === 'Event' &&
					entity.details.type === 'Event' &&
					entity.details.timeline_ms != null &&
					entity.details.involved_entity_ids.includes(char.id)
				) {
					result.push({
						id: `event-${entity.id}`,
						kind: 'event',
						timeMs: entity.details.timeline_ms,
						nodeId: entity.node_refs[0] ?? null,
						label: entity.name,
						detail: entity.tagline,
					});
				}
			}
		}

		// 3. Script mentions — nodes that reference this character
		if (characterTimelineState.showMentions) {
			for (const nodeId of char.node_refs) {
				const node = findNode(nodeId);
				if (node) {
					const midMs = (node.time_range.start_ms + node.time_range.end_ms) / 2;
					result.push({
						id: `mention-${nodeId}`,
						kind: 'mention',
						timeMs: midMs,
						nodeId,
						label: node.name,
						detail: `Referenced in "${node.name}"`,
					});
				}
			}
		}

		return result.sort((a, b) => a.timeMs - b.timeMs);
	});

	/** Viewport culling: only render markers within visible range. */
	let visibleMarkers = $derived.by(() => {
		const vw = timelineState.viewportWidth;
		if (vw <= 0) return markers;
		const sx = timelineState.scrollX;
		const buffer = 12;
		return markers.filter(m => {
			const x = timeToX(m.timeMs);
			return x + buffer >= sx && x - buffer <= sx + vw;
		});
	});

	/** Playhead position relative to viewport. */
	let playheadX = $derived(timeToX(timelineState.playheadMs) - timelineState.scrollX);
	let playheadVisible = $derived(playheadX >= -2 && playheadX <= timelineState.viewportWidth + 2);

	/** Ruler ticks — matching the main timeline's tick generation. */
	function ticks(): { ms: number; label: string; major: boolean }[] {
		const result: { ms: number; label: string; major: boolean }[] = [];
		const minor = 30_000;
		const major = 150_000;
		for (let ms = 0; ms <= TIMELINE.DURATION_MS; ms += minor) {
			result.push({ ms, label: formatTime(ms), major: ms % major === 0 });
		}
		return result;
	}

	/** Sync zoom/scroll from wheel events (mirrors Timeline.svelte). */
	function onwheel(e: WheelEvent) {
		e.preventDefault();
		if (e.ctrlKey) {
			const factor = e.deltaY > 0 ? 0.9 : 1.1;
			zoomTo(timelineState.zoom * factor);
		} else {
			const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
			const maxScroll = Math.max(0, totalWidth() - timelineState.viewportWidth);
			timelineState.scrollX = Math.max(0, Math.min(maxScroll, timelineState.scrollX + delta));
		}
	}

	/** Click on ruler to move playhead. */
	function handleRulerClick(e: MouseEvent) {
		const target = e.currentTarget as HTMLElement;
		const rect = target.getBoundingClientRect();
		const localX = e.clientX - rect.left;
		const timeMs = xToTime(localX + timelineState.scrollX);
		timelineState.playheadMs = Math.max(0, Math.min(timeMs, TIMELINE.DURATION_MS));
	}

	function handleCharacterChange(e: Event) {
		const select = e.currentTarget as HTMLSelectElement;
		characterTimelineState.selectedCharacterId = select.value || null;
	}

	function toggleFilter(filter: 'showSnapshots' | 'showEvents' | 'showMentions') {
		characterTimelineState[filter] = !characterTimelineState[filter];
	}

	function handleClose() {
		characterTimelineState.visible = false;
	}
</script>

<div class="char-timeline-panel" {onwheel}>
	<!-- Header bar with character selector and controls -->
	<div class="char-toolbar">
		<span class="char-toolbar-label">Character</span>

		{#if characters.length > 0}
			{#if selectedCharacter}
				<span class="char-dot" style="background: {colorToHex(selectedCharacter.color)}"></span>
			{/if}
			<select
				class="char-select"
				value={characterTimelineState.selectedCharacterId ?? ''}
				onchange={handleCharacterChange}
			>
				{#each characters as char}
					<option value={char.id}>{char.name}</option>
				{/each}
			</select>
		{:else}
			<span class="no-chars">No characters</span>
		{/if}

		<div class="tl-sep"></div>

		<!-- Filter toggles -->
		<button
			class="filter-btn"
			class:active={characterTimelineState.showSnapshots}
			title="Plot points (snapshots)"
		  onclick={() => toggleFilter('showSnapshots')}
		>
			<span class="filter-icon diamond"></span>
			<span class="filter-label">Plot</span>
		</button>
		<button
			class="filter-btn"
			class:active={characterTimelineState.showEvents}
			title="Events"
			onclick={() => toggleFilter('showEvents')}
		>
			<span class="filter-icon triangle"></span>
			<span class="filter-label">Events</span>
		</button>
		<button
			class="filter-btn"
			class:active={characterTimelineState.showMentions}
			title="Script mentions"
			onclick={() => toggleFilter('showMentions')}
		>
			<span class="filter-icon circle"></span>
			<span class="filter-label">Mentions</span>
		</button>

		<div class="tl-sep"></div>

		<span class="marker-count">{markers.length} marker{markers.length !== 1 ? 's' : ''}</span>

		<button class="close-btn" title="Hide character timeline (C)" onclick={handleClose}>&times;</button>
	</div>

	<!-- Time ruler (compact, synced with main timeline) -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="char-ruler" onclick={handleRulerClick}>
		<div class="ruler-track" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
			{#each ticks() as tick}
				<div class="tick" class:major={tick.major} style="left: {timeToX(tick.ms)}px">
					<div class="tick-line"></div>
					{#if tick.major}
						<span class="tick-label">{tick.label}</span>
					{/if}
				</div>
			{/each}
		</div>
	</div>

	<!-- Marker track area with playhead -->
	<div class="char-content" style="position: relative;">
		<!-- Playhead line -->
		{#if playheadVisible}
			<div class="char-playhead" style="left: {playheadX}px"></div>
		{/if}

		<!-- Marker track -->
		<div class="marker-track" style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)">
			{#each visibleMarkers as marker (marker.id)}
				<CharacterMarker {marker} />
			{/each}
		</div>
	</div>
</div>

<style>
	.char-timeline-panel {
		display: flex;
		flex-direction: column;
		width: 100%;
		overflow: hidden;
		user-select: none;
		background: var(--color-bg-primary);
		border-top: 2px solid var(--color-border-default);
	}

	/* Toolbar */
	.char-toolbar {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border-subtle);
		flex-shrink: 0;
	}

	.char-toolbar-label {
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-right: 4px;
	}

	.char-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.char-select {
		background: var(--color-bg-primary);
		border: 1px solid var(--color-border-subtle);
		border-radius: 3px;
		color: var(--color-text-primary);
		font-size: 0.8rem;
		padding: 2px 6px;
		cursor: pointer;
		outline: none;
		max-width: 160px;
	}

	.char-select:hover {
		border-color: var(--color-border-default);
	}

	.char-select:focus {
		border-color: var(--color-accent);
	}

	.no-chars {
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}

	.tl-sep {
		width: 1px;
		height: 16px;
		background: var(--color-border-subtle);
		margin: 0 2px;
	}

	.filter-btn {
		background: none;
		border: 1px solid transparent;
		padding: 2px 6px;
		border-radius: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 4px;
		opacity: 0.4;
		transition: opacity 0.15s;
		color: var(--color-text-secondary);
	}

	.filter-btn.active {
		opacity: 1;
	}

	.filter-btn:hover {
		background: var(--color-bg-hover);
		border-color: var(--color-border-subtle);
	}

	.filter-label {
		font-size: 0.7rem;
	}

	.filter-icon.diamond {
		width: 7px;
		height: 7px;
		background: var(--color-marker-snapshot);
		transform: rotate(45deg);
	}

	.filter-icon.triangle {
		width: 0;
		height: 0;
		border-left: 4px solid transparent;
		border-right: 4px solid transparent;
		border-bottom: 7px solid var(--color-marker-event);
	}

	.filter-icon.circle {
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--color-marker-mention);
	}

	.marker-count {
		font-size: 0.7rem;
		color: var(--color-text-muted);
		margin-left: auto;
	}

	.close-btn {
		background: none;
		border: none;
		color: var(--color-text-muted);
		font-size: 1rem;
		cursor: pointer;
		padding: 0 4px;
		line-height: 1;
		border-radius: 3px;
	}

	.close-btn:hover {
		color: var(--color-text-primary);
		background: var(--color-bg-hover);
	}

	/* Compact time ruler */
	.char-ruler {
		flex-shrink: 0;
		height: 20px;
		border-bottom: 1px solid var(--color-border-subtle);
		overflow: hidden;
		position: relative;
		background: var(--color-bg-secondary);
		cursor: pointer;
	}

	.ruler-track {
		position: relative;
		height: 100%;
	}

	.tick {
		position: absolute;
		top: 0;
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.tick-line {
		width: 1px;
		height: 4px;
		background: var(--color-text-muted);
	}

	.tick.major .tick-line {
		height: 8px;
		background: var(--color-text-secondary);
	}

	.tick-label {
		font-size: 0.55rem;
		color: var(--color-text-muted);
		margin-top: 1px;
		white-space: nowrap;
	}

	/* Marker content area */
	.char-content {
		flex: 1;
		overflow: hidden;
		min-height: 48px;
	}

	.char-playhead {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--color-playhead);
		z-index: 10;
		pointer-events: none;
	}

	.marker-track {
		position: relative;
		height: 100%;
	}
</style>
