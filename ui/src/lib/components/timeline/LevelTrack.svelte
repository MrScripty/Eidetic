<script lang="ts">
  import type { Track, StoryNode } from '$lib/timelineTypes.js';
  import type { TimelineRenderClip, TimelineRenderGap } from '$lib/timelineRenderTypes.js';
  import {
    adjacentTimelineRenderClipBounds,
    timelineRenderClipsByTrackId,
    type TimelineRenderModelClip,
    type TimelineRenderModelTrack,
  } from '$lib/timelineRenderModel.js';
  import { colorToHex } from '$lib/storyArcTypes.js';
  import { TIMELINE } from '$lib/timelineTypes.js';
  import { xToTime, timeToX, timelineState } from '$lib/stores/timeline.svelte.js';
  import { editorState, startGeneration } from '$lib/stores/editor.svelte.js';
  import { storyArcProjectionState } from '$lib/stores/storyArcProjection.svelte.js';
  import { generateContent } from '$lib/api.js';
  import { notify } from '$lib/stores/notifications.svelte.js';
  import {
    applyCreateTimelineNodeCommand,
    applyDeleteTimelineNodeCommand,
    applySplitTimelineNodeCommand,
    applyTimelineNodeRangeCommand,
    getCachedTimelineRenderModel,
  } from '$lib/stores/timelineRenderProjection.svelte.js';
  import { refreshSelectedNodeEditorProjection } from '$lib/stores/selectedNodeEditorProjection.svelte.js';
  import StoryNodeClip from './StoryNodeClip.svelte';

  let {
    track,
    gaps = [],
    onconnectstart,
  }: {
    track: Track | TimelineRenderModelTrack;
    gaps?: TimelineRenderGap[];
    onconnectstart: (nodeId: string, x: number, y: number) => void;
  } = $props();

  let renderModel = $derived(getCachedTimelineRenderModel());
  let trackId = $derived('track_id' in track ? track.track_id : track.id);
  let levelClips = $derived(renderModel ? timelineRenderClipsByTrackId(renderModel, trackId) : []);
  let storyArcs = $derived(storyArcProjectionState.projection?.payload.arcs ?? []);

  // Viewport-aware node filtering.
  let visibleClips = $derived.by(() => {
    const vw = timelineState.viewportWidth;
    if (vw <= 0) return levelClips;
    const sx = timelineState.scrollX;
    return levelClips.filter((clip) => {
      const left = timeToX(clip.start_ms);
      const right = timeToX(clip.end_ms);
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

  function nodeBounds(clip: TimelineRenderModelClip): { left: number; right: number } {
    if (!renderModel) return { left: 0, right: TIMELINE.DURATION_MS };
    const bounds = adjacentTimelineRenderClipBounds(renderModel, clip);
    return { left: bounds.left_ms, right: bounds.right_ms };
  }

  function arcColorForClip(clip: TimelineRenderClip): string {
    if (clip.arc_ids.length === 0) return 'var(--color-rel-default)';
    const arc = storyArcs.find((a) => a.id === clip.arc_ids[0]);
    return arc ? colorToHex(arc.color) : 'var(--color-rel-default)';
  }

  function storyNodeFromClip(clip: TimelineRenderClip): StoryNode {
    return {
      id: clip.node_id,
      parent_id: clip.parent_id ?? null,
      level: clip.level,
      sort_order: clip.sort_order,
      time_range: {
        start_ms: clip.start_ms,
        end_ms: clip.end_ms,
      },
      name: clip.name,
      content: {
        notes: '',
        content: '',
        status: clip.content_status,
      },
      beat_type: clip.beat_type ?? null,
      locked: clip.locked,
    };
  }

  function selectNode(clip: TimelineRenderClip) {
    editorState.selectedNodeId = clip.node_id;
    editorState.selectedLevel = clip.level;
    void refreshSelectedNodeEditorProjection(clip.node_id).catch(() => {});
  }

  async function handleMove(nodeId: string, startMs: number, endMs: number) {
    await applyTimelineNodeRangeCommand({
      node_id: nodeId,
      start_ms: startMs,
      end_ms: endMs,
    });
  }

  let regenPromptNodeId: string | null = $state(null);

  async function handleResize(nodeId: string, startMs: number, endMs: number) {
    await applyTimelineNodeRangeCommand({
      node_id: nodeId,
      start_ms: startMs,
      end_ms: endMs,
    });
    const clip = levelClips.find((clip) => clip.node_id === nodeId);
    if (clip?.content_status === 'HasContent') {
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
      editorState.selectedLevel = null;
      void refreshSelectedNodeEditorProjection(null).catch(() => {});
    }
    try {
      await applyDeleteTimelineNodeCommand({ node_id: nodeId });
    } catch (e) {
      notify('error', `Delete failed: ${e instanceof Error ? e.message : 'unknown error'}`);
    }
  }

  async function handleSplit(nodeId: string, atMs: number) {
    if (editorState.selectedNodeId === nodeId) {
      editorState.selectedNodeId = null;
      editorState.selectedLevel = null;
      void refreshSelectedNodeEditorProjection(null).catch(() => {});
    }
    try {
      await applySplitTimelineNodeCommand({
        node_id: nodeId,
        at_ms: atMs,
        left_node_id: crypto.randomUUID(),
        right_node_id: crypto.randomUUID(),
      });
    } catch (e) {
      notify('error', `Split failed: ${e instanceof Error ? e.message : 'unknown error'}`);
    }
  }

  async function handleFillGap(gap: TimelineRenderGap) {
    try {
      await applyCreateTimelineNodeCommand({
        node_id: crypto.randomUUID(),
        parent_id: null,
        level: track.level,
        name: 'Bridge',
        start_ms: gap.time_range.start_ms,
        end_ms: gap.time_range.end_ms,
        beat_type: null,
      });
    } catch (error) {
      notify(
        'error',
        `Fill gap failed: ${error instanceof Error ? error.message : 'unknown error'}`,
      );
    }
  }

  async function handleDblClick(e: MouseEvent) {
    const target = e.currentTarget as HTMLElement;
    const rect = target.getBoundingClientRect();
    const localX = e.clientX - rect.left;
    const timeMs = xToTime(localX + timelineState.scrollX);
    const defaultDuration = 60_000;
    const startMs = Math.max(0, Math.round(timeMs - defaultDuration / 2));
    const endMs = Math.min(startMs + defaultDuration, TIMELINE.DURATION_MS);
    try {
      await applyCreateTimelineNodeCommand({
        node_id: crypto.randomUUID(),
        parent_id: null,
        level: track.level,
        name: `New ${track.level}`,
        start_ms: startMs,
        end_ms: endMs,
        beat_type: track.level === 'Beat' ? 'Setup' : null,
      });
    } catch (error) {
      notify('error', `Create failed: ${error instanceof Error ? error.message : 'unknown error'}`);
    }
  }
</script>

{#if !track.collapsed}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="level-track" style="height: {TIMELINE.TRACK_ROW_HEIGHT_PX}px">
    <div
      class="track-lane"
      class:blade-mode={timelineState.activeTool === 'blade'}
      ondblclick={handleDblClick}
    >
      {#each visibleClips as clip (clip.clip_id)}
        {@const node = storyNodeFromClip(clip)}
        {@const bounds = nodeBounds(clip)}
        <StoryNodeClip
          {node}
          color={arcColorForClip(clip)}
          selected={editorState.selectedNodeId === clip.node_id}
          leftBoundMs={bounds.left}
          rightBoundMs={bounds.right}
          onselect={() => selectNode(clip)}
          onmove={(s, e) => handleMove(clip.node_id, s, e)}
          onresize={(s, e) => handleResize(clip.node_id, s, e)}
          ondelete={() => handleDelete(clip.node_id)}
          onsplit={(atMs) => handleSplit(clip.node_id, atMs)}
          {onconnectstart}
        />
      {/each}
      {#each visibleGaps as gap}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="gap-marker"
          style="left: {timeToX(gap.time_range.start_ms)}px; width: {timeToX(
            gap.time_range.end_ms,
          ) - timeToX(gap.time_range.start_ms)}px"
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
    box-sizing: border-box;
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
