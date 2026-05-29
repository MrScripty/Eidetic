<script lang="ts">
  import { onMount } from 'svelte';
  import {
    bibleState,
    clearBibleGraphFocusedNeighborhood,
    clearBibleGraphSelection,
    focusBibleGraphNeighborhood,
    selectBibleGraphContextLayer,
    selectBibleGraphEdge,
    selectBibleGraphInfluence,
    selectBibleGraphNeighborhood,
    selectBibleGraphNode,
    selectedBibleGraphNodeId,
  } from '$lib/stores/bible.svelte.js';
  import { editorState } from '$lib/stores/editor.svelte.js';
  import {
    getCachedBibleRenderGraphProjection,
    refreshBibleRenderGraphProjection,
  } from '$lib/stores/bibleRenderGraphProjection.svelte.js';
  import {
    applyGraphRendererCameraCommand,
    applyGraphRendererTextEditorSettings,
  } from '$lib/graphRendererApi.js';
  import { ensureCanonicalBibleRootProjections } from '$lib/stores/bibleGraphNodeProjection.svelte.js';
  import {
    getCachedBibleGraphSchemaListProjection,
    refreshBibleGraphSchemaListProjection,
  } from '$lib/stores/bibleGraphSchemaProjection.svelte.js';
  import {
    clearContextStackProjection,
    getCachedContextStackProjection,
    refreshContextStackProjection,
  } from '$lib/stores/contextStackProjection.svelte.js';
  import BibleRenderGraphOutline from '../sidebar/bible/BibleRenderGraphOutline.svelte';
  import GraphWorkspaceSideLists from './GraphWorkspaceSideLists.svelte';
  import { timelineState } from '$lib/stores/timeline.svelte.js';
  import {
    graphWorkspaceEdgeItems,
    graphWorkspaceNeighborhoodItems,
  } from './graphWorkspaceItems.js';
  import { selectGraphContextLayerForTimeline } from './graphWorkspaceTimelineCrossLink.js';
  import GraphRendererWindowControls from './GraphRendererWindowControls.svelte';
  import { ensureGraphWorkspaceScaffoldProjection } from './graphWorkspaceBootstrap.js';
  import {
    graphWorkspaceEdgeKindFilters,
    toggleGraphWorkspaceEdgeKindFilter,
  } from './graphWorkspaceEdgeKindFilters.js';
  import { graphWorkspaceProjectionRequest } from './graphWorkspaceProjectionRequest.js';
  import BibleGraphAddControls from '../sidebar/bible/BibleGraphAddControls.svelte';
  import BibleGraphCategoryFilters from '../sidebar/bible/BibleGraphCategoryFilters.svelte';
  import {
    bibleGraphCreateOptions,
    bibleGraphFilterOptions,
    type BibleGraphFilter,
    type BibleGraphRootCategory,
  } from '../sidebar/bible/bibleGraphCategories.js';
  import { createBibleGraphNodeForCategory } from '../sidebar/bible/bibleGraphNodeCreateFlow.js';
  import type { BibleGraphEdgeKind } from '$lib/bibleGraphTypes.js';

  const renderGraphProjection = $derived(getCachedBibleRenderGraphProjection());
  const graph = $derived(renderGraphProjection?.payload ?? null);
  const contextStackProjection = $derived(getCachedContextStackProjection());
  const contextLayers = $derived(contextStackProjection?.payload.layers ?? []);
  const schemaProjection = $derived(getCachedBibleGraphSchemaListProjection());
  const selectedGraphNodeId = $derived(selectedBibleGraphNodeId());
  const graphSelection = $derived(bibleState.graphSelection);
  let showOutline = $state(false);
  let graphSearchQuery = $state('');
  let activeFilter: BibleGraphFilter = $state('all');
  let activeEdgeKinds: BibleGraphEdgeKind[] = $state([]);
  let activeGraphPanelTab: 'graph' | 'settings' = $state('graph');
  let textEditorPaddingPx = $state(17);
  let textEditorCornerRadiusPx = $state(4);
  let textEditorOutlineWidthPx = $state(1);
  let textEditorOutlineBrightness = $state(0.12);
  let textEditorOutlineTransparency = $state(0.05);
  let textEditorFontSizePx = $state(15);
  let textEditorFontBrightness = $state(0.88);
  let textEditorBackgroundBrightness = $state(0.075);
  let textEditorBackgroundTransparency = $state(0.08);
  let selectedNodeOutlineWidthPx = $state(4);
  let selectedNodeOutlineBrightness = $state(1);
  let selectedNodeOutlineColor = $state('#f2c94c');
  const focusedNeighborhoodNodeId = $derived(bibleState.graphFocusedNeighborhoodNodeId);
  let initialGraphScaffoldLoaded = $state(false);
  let graphLoadError = $state<string | null>(null);
  const graphFilterOptions = $derived(bibleGraphFilterOptions(schemaProjection?.payload));
  const graphCreateOptions = $derived(bibleGraphCreateOptions(schemaProjection?.payload));
  const activeFilterRootId = $derived(
    graphFilterOptions.find((option) => option.filter === activeFilter)?.rootNodeId ?? null,
  );
  const renderGraphRequest = $derived(
    graphWorkspaceProjectionRequest({
      selectedTimelineNodeId: editorState.selectedNodeId,
      focusedNeighborhoodNodeId,
      activeTimelineMs: timelineState.playheadMs,
      activeFilter,
      activeFilterRootId,
      edgeKinds: activeEdgeKinds,
      search: graphSearchQuery,
    }),
  );
  const edgeItems = $derived(graph ? graphWorkspaceEdgeItems(graph) : []);
  const neighborhoodItems = $derived(graph ? graphWorkspaceNeighborhoodItems(graph) : []);
  const hasSideLists = $derived(
    graph
      ? contextLayers.length > 0 ||
          graph.influences.length > 0 ||
          edgeItems.length > 0 ||
          neighborhoodItems.length > 0
      : false,
  );

  onMount(() => {
    void refreshBibleGraphSchemaListProjection().catch((error) => {
      graphLoadError = error instanceof Error ? error.message : 'Failed to load graph schemas';
    });
    void ensureGraphWorkspaceScaffoldProjection(renderGraphRequest, {
      ensureCanonicalRoots: ensureCanonicalBibleRootProjections,
      refreshRenderGraph: refreshBibleRenderGraphProjection,
    })
      .catch((error) => {
        graphLoadError = error instanceof Error ? error.message : 'Failed to load graph workspace';
      })
      .finally(() => {
        initialGraphScaffoldLoaded = true;
      });
  });

  $effect(() => {
    if (!initialGraphScaffoldLoaded) {
      return;
    }
    const selectedTimelineNodeId = editorState.selectedNodeId;
    void refreshBibleRenderGraphProjection(renderGraphRequest).catch((error) => {
      graphLoadError = error instanceof Error ? error.message : 'Failed to load graph workspace';
    });

    if (!selectedTimelineNodeId) {
      clearContextStackProjection();
      return;
    }
    void refreshContextStackProjection(selectedTimelineNodeId).catch(() => {});
  });

  function handleSelect(id: string) {
    selectBibleGraphNode(selectedGraphNodeId === id ? null : id);
  }

  function toggleEdgeKindFilter(kind: BibleGraphEdgeKind): void {
    activeEdgeKinds = toggleGraphWorkspaceEdgeKindFilter(activeEdgeKinds, kind);
  }

  async function handleAddGraphNode(category: BibleGraphRootCategory): Promise<void> {
    try {
      graphLoadError = null;
      const projection = await createBibleGraphNodeForCategory(category);
      await refreshBibleRenderGraphProjection(renderGraphRequest);
      selectBibleGraphNode(projection.projection.payload.node.id);
    } catch (error) {
      graphLoadError = error instanceof Error ? error.message : 'Failed to create bible graph node';
    }
  }

  function focusSelectedNeighborhood() {
    if (selectedGraphNodeId) {
      focusBibleGraphNeighborhood(selectedGraphNodeId);
      void applyGraphRendererCameraCommand({
        type: 'navigate_to_neighborhood',
        node_id: selectedGraphNodeId,
      }).catch(() => {});
    }
  }

  function clearFocusedNeighborhood() {
    clearBibleGraphFocusedNeighborhood();
  }

  function clearGraphSelectionAndFocus() {
    clearBibleGraphSelection();
    clearBibleGraphFocusedNeighborhood();
  }

  function handleSelectInfluence(id: string) {
    selectBibleGraphInfluence(
      graphSelection.kind === 'influence' && graphSelection.influenceId === id ? null : id,
    );
  }

  function handleSelectContextLayer(id: string) {
    const result = selectGraphContextLayerForTimeline(
      {
        graphSelection,
        selectedTimelineNodeId: editorState.selectedNodeId,
      },
      id,
    );
    editorState.selectedNodeId = result.selectedTimelineNodeId;
    selectBibleGraphContextLayer(
      result.graphSelection.kind === 'context_layer' ? result.graphSelection.timelineNodeId : null,
    );
  }

  function handleSelectEdge(id: string) {
    selectBibleGraphEdge(
      graphSelection.kind === 'edge' && graphSelection.edgeId === id ? null : id,
    );
  }

  function handleSelectNeighborhood(id: string) {
    selectBibleGraphNeighborhood(
      graphSelection.kind === 'neighborhood' && graphSelection.nodeId === id ? null : id,
    );
  }

  function applyTextEditorSettings(): void {
    void applyGraphRendererTextEditorSettings({
      padding_px: textEditorPaddingPx,
      corner_radius_px: textEditorCornerRadiusPx,
      editor_outline_width_px: textEditorOutlineWidthPx,
      editor_outline_brightness: textEditorOutlineBrightness,
      editor_outline_transparency: textEditorOutlineTransparency,
      font_size_px: textEditorFontSizePx,
      font_brightness: textEditorFontBrightness,
      editor_background_brightness: textEditorBackgroundBrightness,
      editor_background_transparency: textEditorBackgroundTransparency,
      selected_node_outline_width_px: selectedNodeOutlineWidthPx,
      selected_node_outline_brightness: selectedNodeOutlineBrightness,
      selected_node_outline_color: selectedNodeOutlineColor,
    }).catch((error) => {
      graphLoadError =
        error instanceof Error ? error.message : 'Failed to update graph renderer settings';
    });
  }
</script>

<section class="graph-workspace" aria-label="Bible graph workspace">
  {#if graph}
    <div class="graph-summary" aria-label="Bible graph projection summary">
      <span>{graph.nodes.length} nodes</span>
      <span>{graph.edges.length} edges</span>
      <span>{graph.influences.length} influences</span>
      {#if graphLoadError}
        <span class="error">{graphLoadError}</span>
      {/if}
    </div>

    <div class="graph-surface" class:has-side-lists={hasSideLists}>
      <div class="graph-renderer-shell">
        <GraphRendererWindowControls graphProjectionRequest={renderGraphRequest} />
        <div class="graph-renderer-primary" aria-label="Bevy graph renderer workspace">
          <div class="graph-control-panel">
            <div class="graph-control-tabs" role="tablist" aria-label="Graph workspace panels">
              <button
                type="button"
                role="tab"
                class:active={activeGraphPanelTab === 'graph'}
                aria-selected={activeGraphPanelTab === 'graph'}
                onclick={() => (activeGraphPanelTab = 'graph')}
              >
                Graph
              </button>
              <button
                type="button"
                role="tab"
                class:active={activeGraphPanelTab === 'settings'}
                aria-selected={activeGraphPanelTab === 'settings'}
                onclick={() => (activeGraphPanelTab = 'settings')}
              >
                Settings
              </button>
            </div>

            {#if activeGraphPanelTab === 'graph'}
              <div class="graph-controls" aria-label="Graph projection controls">
                <input
                  class="search-input"
                  type="text"
                  placeholder="Search graph..."
                  bind:value={graphSearchQuery}
                />
                <BibleGraphCategoryFilters
                  {activeFilter}
                  filters={graphFilterOptions}
                  schemaProjection={schemaProjection?.payload}
                  onselect={(filter) => (activeFilter = filter)}
                />
                <div class="edge-kind-controls" aria-label="Edge kind filters">
                  {#each graphWorkspaceEdgeKindFilters as filter (filter.label)}
                    <button
                      type="button"
                      class:active={activeEdgeKinds.includes(filter.kind)}
                      aria-pressed={activeEdgeKinds.includes(filter.kind)}
                      onclick={() => toggleEdgeKindFilter(filter.kind)}
                    >
                      {filter.label}
                    </button>
                  {/each}
                </div>
                <BibleGraphAddControls
                  {activeFilter}
                  categories={graphCreateOptions}
                  schemaProjection={schemaProjection?.payload}
                  onadd={handleAddGraphNode}
                />
                <button
                  type="button"
                  class="outline-toggle"
                  aria-expanded={showOutline}
                  onclick={() => {
                    showOutline = !showOutline;
                  }}
                >
                  {showOutline ? 'Hide outline' : 'Show outline'}
                </button>
                <button
                  type="button"
                  disabled={!selectedGraphNodeId ||
                    focusedNeighborhoodNodeId === selectedGraphNodeId}
                  onclick={focusSelectedNeighborhood}
                >
                  Focus neighborhood
                </button>
                <button
                  type="button"
                  disabled={!focusedNeighborhoodNodeId}
                  onclick={clearFocusedNeighborhood}
                >
                  Clear focus
                </button>
                <button
                  type="button"
                  disabled={graphSelection.kind === 'none' && !focusedNeighborhoodNodeId}
                  onclick={clearGraphSelectionAndFocus}
                >
                  Clear selection
                </button>
              </div>
            {:else}
              <div class="graph-settings" aria-label="Graph renderer settings">
                <label>
                  <span>Padding</span>
                  <input
                    type="range"
                    min="0"
                    max="48"
                    step="1"
                    bind:value={textEditorPaddingPx}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{textEditorPaddingPx}px</output>
                </label>
                <label>
                  <span>Rounding</span>
                  <input
                    type="range"
                    min="0"
                    max="16"
                    step="1"
                    bind:value={textEditorCornerRadiusPx}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{textEditorCornerRadiusPx}px</output>
                </label>
                <label>
                  <span>Editor outline</span>
                  <input
                    type="range"
                    min="0"
                    max="8"
                    step="1"
                    bind:value={textEditorOutlineWidthPx}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{textEditorOutlineWidthPx}px</output>
                </label>
                <label>
                  <span>Editor outline light</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={textEditorOutlineBrightness}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(textEditorOutlineBrightness * 100)}%</output>
                </label>
                <label>
                  <span>Editor outline transparency</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={textEditorOutlineTransparency}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(textEditorOutlineTransparency * 100)}%</output>
                </label>
                <label>
                  <span>Font size</span>
                  <input
                    type="range"
                    min="10"
                    max="28"
                    step="1"
                    bind:value={textEditorFontSizePx}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{textEditorFontSizePx}px</output>
                </label>
                <label>
                  <span>Font light</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={textEditorFontBrightness}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(textEditorFontBrightness * 100)}%</output>
                </label>
                <label>
                  <span>Background light</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={textEditorBackgroundBrightness}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(textEditorBackgroundBrightness * 100)}%</output>
                </label>
                <label>
                  <span>Background transparency</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={textEditorBackgroundTransparency}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(textEditorBackgroundTransparency * 100)}%</output>
                </label>
                <label>
                  <span>Selected outline</span>
                  <input
                    type="range"
                    min="1"
                    max="18"
                    step="1"
                    bind:value={selectedNodeOutlineWidthPx}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{selectedNodeOutlineWidthPx}px</output>
                </label>
                <label>
                  <span>Selected light</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    bind:value={selectedNodeOutlineBrightness}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{Math.round(selectedNodeOutlineBrightness * 100)}%</output>
                </label>
                <label>
                  <span>Selected color</span>
                  <input
                    type="color"
                    bind:value={selectedNodeOutlineColor}
                    oninput={applyTextEditorSettings}
                  />
                  <output>{selectedNodeOutlineColor}</output>
                </label>
              </div>
            {/if}
          </div>
        </div>
        {#if showOutline}
          <div class="graph-outline-inspector">
            <BibleRenderGraphOutline
              projection={graph}
              selectedNodeId={selectedGraphNodeId}
              onselect={handleSelect}
            />
          </div>
        {/if}
      </div>

      {#if hasSideLists}
        <GraphWorkspaceSideLists
          selection={graphSelection}
          influences={graph.influences}
          {contextLayers}
          {edgeItems}
          {neighborhoodItems}
          onselectinfluence={handleSelectInfluence}
          onselectcontextlayer={handleSelectContextLayer}
          onselectedge={handleSelectEdge}
          onselectneighborhood={handleSelectNeighborhood}
        />
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      {graphLoadError ?? 'No graph projection'}
    </div>
  {/if}
</section>

<style>
  .graph-workspace {
    display: flex;
    flex-direction: column;
    min-height: 0;
    height: 100%;
    background: var(--color-bg-primary);
  }

  .graph-summary {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 30px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--color-border-subtle);
    color: var(--color-text-muted);
    font-size: 0.76rem;
    font-variant-numeric: tabular-nums;
  }

  .graph-summary .error {
    color: var(--color-danger);
  }

  .graph-surface {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    min-height: 0;
    flex: 1;
    overflow: hidden;
  }

  .graph-surface.has-side-lists {
    grid-template-columns: minmax(0, 1fr) minmax(220px, 0.34fr);
  }

  .graph-renderer-shell {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
  }

  .graph-renderer-primary {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    min-width: 0;
    min-height: 0;
    flex: 1;
    padding: 10px;
    background: var(--color-bg-primary);
  }

  .graph-control-panel {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    max-width: min(100%, 760px);
  }

  .graph-control-tabs {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 3px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 5px;
    background: var(--color-bg-surface);
  }

  .graph-control-tabs button {
    border: 0;
    border-radius: 3px;
    padding: 5px 8px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.72rem;
    cursor: pointer;
  }

  .graph-control-tabs button.active {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .graph-controls {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    max-width: min(100%, 760px);
    padding: 7px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 5px;
    background: var(--color-bg-surface);
  }

  .graph-settings {
    display: grid;
    grid-template-columns: repeat(2, minmax(220px, 1fr));
    gap: 8px 12px;
    width: min(100%, 620px);
    padding: 9px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 5px;
    background: var(--color-bg-surface);
  }

  .graph-settings label {
    display: grid;
    grid-template-columns: minmax(120px, 0.48fr) minmax(120px, 1fr) minmax(48px, auto);
    align-items: center;
    gap: 8px;
    color: var(--color-text-secondary);
    font-size: 0.74rem;
  }

  .graph-settings input[type='range'] {
    width: 100%;
    accent-color: var(--color-accent);
  }

  .graph-settings output {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
    text-align: right;
  }

  .graph-controls :global(.add-buttons) {
    border: 0;
    background: transparent;
  }

  .graph-controls :global(.add-menu) {
    padding: 0;
  }

  .graph-controls :global(.add-btn) {
    width: auto;
    padding: 6px 9px;
  }

  .edge-kind-controls {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .edge-kind-controls button {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 5px 7px;
    background: var(--color-bg-primary);
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 0.72rem;
  }

  .edge-kind-controls button:hover,
  .edge-kind-controls button:focus-visible {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .edge-kind-controls button.active {
    border-color: color-mix(in srgb, var(--color-warning) 60%, var(--color-border-subtle));
    color: var(--color-warning);
  }

  .search-input {
    width: min(220px, 28vw);
    min-width: 120px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 6px 8px;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    font-size: 0.78rem;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--color-accent);
  }

  .outline-toggle {
    border: 1px solid var(--color-border-subtle);
    border-radius: 4px;
    padding: 6px 9px;
    background: var(--color-bg-surface);
    color: var(--color-text-secondary);
    font-size: 0.78rem;
    cursor: pointer;
  }

  .outline-toggle:hover,
  .outline-toggle:focus-visible {
    background: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .outline-toggle:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
  }

  .graph-outline-inspector {
    min-width: 0;
    max-height: 42%;
    overflow: hidden;
    border-top: 1px solid var(--color-border-subtle);
    background: var(--color-bg-primary);
  }

  .graph-outline-inspector :global(.graph-outline) {
    max-height: none;
    height: 100%;
    border: 0;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
    color: var(--color-text-muted);
    font-size: 0.85rem;
  }
</style>
