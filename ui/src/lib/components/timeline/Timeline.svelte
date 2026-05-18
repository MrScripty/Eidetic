<script lang="ts">
  import LevelTrack from './LevelTrack.svelte';
  import StructureBar from './StructureBar.svelte';
  import TimeRuler from './TimeRuler.svelte';
  import RelationshipLayer from './RelationshipLayer.svelte';
  import Playhead from './Playhead.svelte';
  import TimelineToolbar from './TimelineToolbar.svelte';
  import {
    timelineState,
    totalWidth,
    connectionDrag,
    timeToX,
    zoomToFit,
    zoomTo,
  } from '$lib/stores/timeline.svelte.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import { registerShortcut } from '$lib/stores/shortcuts.svelte.js';
  import { TIMELINE, timelineTrackRowsHeightPx } from '$lib/types.js';
  import type { TimelineGap, StoryLevel } from '$lib/types.js';
  import { getGaps } from '$lib/api.js';
  import { characterTimelineState } from '$lib/stores/characterTimeline.svelte.js';
  import {
    applyCreateTimelineRelationshipCommand,
    applyTimelineNodeRangeCommand,
  } from '$lib/stores/timelineRenderProjection.svelte.js';

  let gaps = $state<TimelineGap[]>([]);
  let scrollbarEl: HTMLDivElement | undefined = $state();
  let contentScrollEl: HTMLDivElement | undefined = $state();
  let scrollbarSyncing = false;
  let hasAutoFit = false;
  let visibleTrackCount = $derived(
    timelineState.timeline?.tracks.filter((track) => !track.collapsed).length ?? 0,
  );
  let trackRowsHeight = $derived(timelineTrackRowsHeightPx(visibleTrackCount));

  // Auto zoom-to-fit when timeline first loads and viewport is measured.
  $effect(() => {
    if (!hasAutoFit && timelineState.timeline && timelineState.viewportWidth > 0) {
      hasAutoFit = true;
      zoomToFit();
    }
  });

  // Refetch gaps when timeline changes.
  $effect(() => {
    if (timelineState.timeline) {
      getGaps()
        .then((g) => (gaps = g))
        .catch(() => {});
    }
  });

  function gapsForLevel(level: StoryLevel): TimelineGap[] {
    return gaps.filter((g) => g.level === level);
  }

  function onwheel(e: WheelEvent) {
    e.preventDefault();
    if (e.ctrlKey) {
      const factor = e.deltaY > 0 ? 0.9 : 1.1;
      zoomTo(timelineState.zoom * factor);
    } else if (
      contentScrollEl &&
      contentScrollEl.scrollHeight > contentScrollEl.clientHeight &&
      Math.abs(e.deltaY) > Math.abs(e.deltaX) &&
      !e.shiftKey
    ) {
      contentScrollEl.scrollTop += e.deltaY;
    } else {
      const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
      const maxScroll = Math.max(0, totalWidth() - timelineState.viewportWidth);
      timelineState.scrollX = Math.max(0, Math.min(maxScroll, timelineState.scrollX + delta));
    }
  }

  function onScrollbar(e: Event) {
    const el = e.currentTarget as HTMLDivElement;
    if (!scrollbarSyncing) {
      timelineState.scrollX = el.scrollLeft;
    }
  }

  // Keep scrollbar in sync when scrollX changes from wheel/zoom.
  $effect(() => {
    const sx = timelineState.scrollX;
    if (scrollbarEl && Math.abs(scrollbarEl.scrollLeft - sx) > 1) {
      scrollbarSyncing = true;
      scrollbarEl.scrollLeft = sx;
      scrollbarSyncing = false;
    }
  });

  // Register timeline tool shortcuts.
  $effect(() => {
    const unsubs = [
      registerShortcut({
        key: 'a',
        description: 'Selection tool',
        skipInInput: true,
        action: () => {
          timelineState.activeTool = 'select';
        },
      }),
      registerShortcut({
        key: 'b',
        description: 'Blade tool',
        skipInInput: true,
        action: () => {
          timelineState.activeTool = 'blade';
        },
      }),
      registerShortcut({
        key: 'n',
        description: 'Toggle snapping',
        skipInInput: true,
        action: () => {
          timelineState.snapping = !timelineState.snapping;
        },
      }),
      registerShortcut({
        key: 'c',
        description: 'Toggle character timeline',
        skipInInput: true,
        action: () => {
          characterTimelineState.visible = !characterTimelineState.visible;
        },
      }),
      registerShortcut({
        key: 'ArrowLeft',
        alt: true,
        description: 'Nudge selected timeline node left',
        skipInInput: true,
        action: () => {
          void nudgeSelectedNode(-1_000);
        },
      }),
      registerShortcut({
        key: 'ArrowRight',
        alt: true,
        description: 'Nudge selected timeline node right',
        skipInInput: true,
        action: () => {
          void nudgeSelectedNode(1_000);
        },
      }),
      registerShortcut({
        key: 'ArrowLeft',
        alt: true,
        shift: true,
        description: 'Nudge selected timeline node left by five seconds',
        skipInInput: true,
        action: () => {
          void nudgeSelectedNode(-5_000);
        },
      }),
      registerShortcut({
        key: 'ArrowRight',
        alt: true,
        shift: true,
        description: 'Nudge selected timeline node right by five seconds',
        skipInInput: true,
        action: () => {
          void nudgeSelectedNode(5_000);
        },
      }),
    ];
    return () => unsubs.forEach((fn) => fn());
  });

  async function nudgeSelectedNode(deltaMs: number) {
    const node = editorState.selectedNode;
    if (!node) return;

    const duration = node.time_range.end_ms - node.time_range.start_ms;
    const startMs = Math.max(
      0,
      Math.min(TIMELINE.DURATION_MS - duration, node.time_range.start_ms + deltaMs),
    );
    const endMs = startMs + duration;
    if (startMs === node.time_range.start_ms && endMs === node.time_range.end_ms) return;

    await applyTimelineNodeRangeCommand({
      node_id: node.id,
      start_ms: startMs,
      end_ms: endMs,
    }).catch(() => {});
  }

  function handleConnectStart(nodeId: string, x: number, y: number) {
    connectionDrag.active = true;
    connectionDrag.fromNodeId = nodeId;
    connectionDrag.fromX = x;
    connectionDrag.fromY = y;
    connectionDrag.currentX = x;
    connectionDrag.currentY = y;

    function onPointerMove(e: PointerEvent) {
      connectionDrag.currentX = e.clientX;
      connectionDrag.currentY = e.clientY;
    }

    async function onPointerUp(e: PointerEvent) {
      document.removeEventListener('pointermove', onPointerMove);
      document.removeEventListener('pointerup', onPointerUp);
      connectionDrag.active = false;

      const target = document.elementFromPoint(e.clientX, e.clientY);
      const clipEl = target?.closest('.node-clip');
      if (clipEl && timelineState.timeline && connectionDrag.fromNodeId) {
        // Find which node was dropped on by checking bounds
        for (const node of timelineState.timeline.nodes) {
          if (node.id === connectionDrag.fromNodeId) continue;
          const nodeLeft = timeToX(node.time_range.start_ms) - timelineState.scrollX;
          const nodeRight = timeToX(node.time_range.end_ms) - timelineState.scrollX;
          const bounds = clipEl.getBoundingClientRect();
          if (Math.abs(bounds.width - (nodeRight - nodeLeft)) < 10) {
            await applyCreateTimelineRelationshipCommand({
              relationship_id: crypto.randomUUID(),
              from_node_id: connectionDrag.fromNodeId,
              to_node_id: node.id,
              relationship_type: 'Causal',
            });
            connectionDrag.fromNodeId = null;
            return;
          }
        }
      }
      connectionDrag.fromNodeId = null;
    }

    document.addEventListener('pointermove', onPointerMove);
    document.addEventListener('pointerup', onPointerUp);
  }

</script>

<div
  class="timeline-container"
  style="
		--timeline-label-width: {TIMELINE.LABEL_WIDTH_PX}px;
		--timeline-ruler-height: {TIMELINE.TIME_RULER_TOTAL_HEIGHT_PX}px;
		--timeline-relationship-height: {TIMELINE.RELATIONSHIP_HEIGHT_PX}px;
		--timeline-track-row-height: {TIMELINE.TRACK_ROW_HEIGHT_PX}px;
		--timeline-structure-height: {TIMELINE.STRUCTURE_BAR_TOTAL_HEIGHT_PX}px;
		--timeline-scrollbar-height: {TIMELINE.SCROLLBAR_HEIGHT_PX}px;
	"
>
  <TimelineToolbar />

  <div class="timeline-body" {onwheel}>
    <div class="ruler-row">
      <div class="label-corner"></div>

      <div class="time-ruler-column" bind:clientWidth={timelineState.viewportWidth}>
        <TimeRuler
          durationMs={TIMELINE.DURATION_MS}
          width={totalWidth()}
          offsetX={timelineState.scrollX}
        />
      </div>
    </div>

    <div class="content-viewport">
      <div class="content-scroll" bind:this={contentScrollEl}>
        <div class="label-column">
          <div class="label-spacer rel-spacer"></div>

          <div class="labels-tracks">
            {#if timelineState.timeline}
              {#each timelineState.timeline.tracks as track}
                <div
                  class="track-label"
                  style="height: {track.collapsed ? 0 : TIMELINE.TRACK_ROW_HEIGHT_PX}px"
                  class:collapsed={track.collapsed}
                >
                  <span class="track-label-text">{track.label}</span>
                </div>
              {/each}
            {/if}
          </div>

          <div class="label-spacer structure-spacer"></div>
        </div>

        <div class="time-column">
          <div class="timeline-content">
            <Playhead />

            <div class="relationship-wrapper">
              <RelationshipLayer offsetX={timelineState.scrollX} />
            </div>

            <div class="tracks" style="height: {trackRowsHeight}px">
              <div
                class="tracks-content"
                style="width: {totalWidth()}px; transform: translateX(-{timelineState.scrollX}px)"
              >
                {#if timelineState.timeline}
                  {#each timelineState.timeline.tracks as track}
                    <LevelTrack
                      {track}
                      gaps={gapsForLevel(track.level)}
                      onconnectstart={handleConnectStart}
                    />
                  {/each}
                {/if}
              </div>
            </div>

            {#if timelineState.timeline}
              <StructureBar
                structure={timelineState.timeline.structure}
                width={totalWidth()}
                offsetX={timelineState.scrollX}
              />
            {/if}
          </div>
        </div>
      </div>
    </div>

    <div class="scrollbar-row">
      <div class="label-spacer scrollbar-spacer"></div>

      <div class="time-scrollbar-column">
        <div class="timeline-scrollbar" bind:this={scrollbarEl} onscroll={onScrollbar}>
          <div style="width: {totalWidth()}px; height: 1px;"></div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .timeline-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
    overflow: hidden;
    user-select: none;
    position: relative;
  }

  .timeline-body {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .ruler-row,
  .scrollbar-row {
    display: flex;
    flex-shrink: 0;
  }

  .label-corner,
  .label-column,
  .scrollbar-spacer {
    width: var(--timeline-label-width);
    flex-shrink: 0;
    background: var(--color-bg-primary);
    border-right: 1px solid var(--color-border-default);
  }

  .label-corner {
    height: var(--timeline-ruler-height);
    box-sizing: border-box;
    border-bottom: 1px solid var(--color-border-default);
  }

  .time-ruler-column,
  .time-scrollbar-column {
    flex: 1;
    min-width: 0;
  }

  .content-viewport {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .content-scroll {
    display: flex;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .label-column {
    display: flex;
    flex-direction: column;
  }

  .track-label {
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
    padding: 0 4px 0 8px;
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    overflow: hidden;
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .track-label.collapsed {
    display: none;
  }

  .track-label-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* -- Time content column -- */

  .time-column {
    flex: 1;
    min-width: 0;
  }

  .timeline-content {
    position: relative;
  }

  .label-spacer {
    flex-shrink: 0;
    box-sizing: border-box;
  }

  .rel-spacer,
  .relationship-wrapper {
    height: var(--timeline-relationship-height);
    box-sizing: border-box;
    border-bottom: 1px solid var(--color-border-subtle);
    background: var(--color-bg-primary);
    overflow: visible;
  }

  .tracks {
    overflow: hidden;
    position: relative;
    min-width: 0;
  }

  .tracks-content {
    position: relative;
    height: 100%;
  }

  .structure-spacer {
    height: var(--timeline-structure-height);
    border-top: 1px solid var(--color-border-default);
  }

  .timeline-scrollbar {
    height: var(--timeline-scrollbar-height);
    box-sizing: border-box;
    overflow-x: auto;
    overflow-y: hidden;
    background: var(--color-bg-secondary);
    border-top: 1px solid var(--color-border-subtle);
  }

  .scrollbar-spacer {
    height: var(--timeline-scrollbar-height);
    box-sizing: border-box;
    border-top: 1px solid var(--color-border-subtle);
  }

</style>
