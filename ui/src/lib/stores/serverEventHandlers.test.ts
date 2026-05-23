import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setupServerEventHandlers } from './serverEventHandlers.js';
import type { ServerMessage } from '$lib/serverEventTypes.js';
import { refreshScriptDocumentProjection } from './scriptDocumentProjection.svelte.js';
import { refreshTimelineRenderProjection } from './timelineRenderProjection.svelte.js';
import { refreshBibleRenderGraphProjection } from './bibleRenderGraphProjection.svelte.js';
import { clearProjectionRefreshQueue } from './projectionRefreshQueue.js';
import { completeGeneration } from './editor.svelte.js';

vi.mock('./timelineRenderProjection.svelte.js', () => ({
  refreshTimelineRenderProjection: vi.fn(),
}));

vi.mock('./scriptDocumentProjection.svelte.js', () => ({
  MAIN_SCRIPT_DOCUMENT_ID: 'script.document.main',
  refreshScriptDocumentProjection: vi.fn(),
}));

vi.mock('./storyArcProjection.svelte.js', () => ({
  refreshStoryArcListProjection: vi.fn(),
}));

vi.mock('./bibleGraphNodeProjection.svelte.js', () => ({
  refreshBibleGraphNodeListProjection: vi.fn(),
}));

vi.mock('./bibleRenderGraphProjection.svelte.js', () => ({
  getActiveBibleRenderGraphProjectionRequest: vi.fn(() => ({
    selected_timeline_node_id: 'node.scene.beach',
    selected_node_id: 'node.character.ada',
    neighborhood_depth: 1,
    max_nodes: 200,
  })),
  refreshBibleRenderGraphProjection: vi.fn(),
}));

vi.mock('./semanticProposalProjection.svelte.js', () => ({
  refreshBibleReferenceProposalListProjection: vi.fn(),
}));

vi.mock('./propagationProposalProjection.svelte.js', () => ({
  refreshPropagationProposalListProjection: vi.fn(),
}));

vi.mock('./changeReviewProjection.svelte.js', () => ({
  refreshChangeReviewProjection: vi.fn(),
}));

vi.mock('./editor.svelte.js', () => ({
  appendStreamingToken: vi.fn(),
  completeGeneration: vi.fn(),
  setGenerationContext: vi.fn(),
  setGenerationError: vi.fn(),
}));

const refreshTimelineRenderProjectionMock = vi.mocked(refreshTimelineRenderProjection);
const refreshScriptDocumentProjectionMock = vi.mocked(refreshScriptDocumentProjection);
const refreshBibleRenderGraphProjectionMock = vi.mocked(refreshBibleRenderGraphProjection);
const completeGenerationMock = vi.mocked(completeGeneration);

class MockServerEventClient {
  readonly handlers = new Map<ServerMessage['type'], (data: ServerMessage) => void>();

  on<T extends ServerMessage['type']>(
    type: T,
    handler: (data: Extract<ServerMessage, { type: T }>) => void,
  ): () => void {
    this.handlers.set(type, handler as (data: ServerMessage) => void);
    return () => this.handlers.delete(type);
  }

  emit(message: ServerMessage): void {
    this.handlers.get(message.type)?.(message);
  }
}

beforeEach(() => {
  clearProjectionRefreshQueue();
  refreshTimelineRenderProjectionMock.mockReset();
  refreshTimelineRenderProjectionMock.mockResolvedValue({
    version: 1,
    payload: { total_duration_ms: 1, tracks: [], clips: [], relationships: [] },
  });
  refreshScriptDocumentProjectionMock.mockReset();
  refreshScriptDocumentProjectionMock.mockResolvedValue({
    version: 1,
    payload: {
      document: { id: 'script.document.main', title: 'Script', sort_order: 0 },
      segments: [],
    },
  });
  refreshBibleRenderGraphProjectionMock.mockReset();
  refreshBibleRenderGraphProjectionMock.mockResolvedValue({
    version: 1,
    payload: { nodes: [], edges: [], neighborhoods: [], influences: [] },
  });
  completeGenerationMock.mockReset();
});

describe('backend event projection handlers', () => {
  it('refreshes timeline projections for timeline events', async () => {
    const events = new MockServerEventClient();
    setupServerEventHandlers(events as never);

    events.emit({ type: 'timeline_changed' });

    await vi.waitFor(() => {
      expect(refreshTimelineRenderProjectionMock).toHaveBeenCalledTimes(1);
    });
  });

  it('coalesces bursty timeline projection events', async () => {
    const events = new MockServerEventClient();
    setupServerEventHandlers(events as never);

    events.emit({ type: 'timeline_changed' });
    events.emit({ type: 'hierarchy_changed' });

    await vi.waitFor(() => {
      expect(refreshTimelineRenderProjectionMock).toHaveBeenCalledTimes(1);
    });
  });

  it('refreshes affected projections for node updates without local durable patches', async () => {
    const events = new MockServerEventClient();
    setupServerEventHandlers(events as never);

    events.emit({ type: 'node_updated', node_id: 'node.beat.one' });

    await vi.waitFor(() => {
      expect(refreshTimelineRenderProjectionMock).toHaveBeenCalledTimes(1);
      expect(refreshScriptDocumentProjectionMock).toHaveBeenCalledWith({
        document_id: 'script.document.main',
      });
    });
  });

  it('refreshes projections before completing generation progress', async () => {
    const events = new MockServerEventClient();
    setupServerEventHandlers(events as never);

    events.emit({ type: 'generation_complete', node_id: 'node.beat.one' });

    await vi.waitFor(() => {
      expect(refreshTimelineRenderProjectionMock).toHaveBeenCalledTimes(1);
      expect(refreshScriptDocumentProjectionMock).toHaveBeenCalledWith({
        document_id: 'script.document.main',
      });
      expect(completeGenerationMock).toHaveBeenCalledWith('node.beat.one');
    });
  });

  it('refreshes selected context graph projections for context influence changes', async () => {
    const events = new MockServerEventClient();
    setupServerEventHandlers(events as never);

    events.emit({
      type: 'context_influence_changed',
      target_node_id: 'node.scene.beach',
    });

    await vi.waitFor(() => {
      expect(refreshBibleRenderGraphProjectionMock).toHaveBeenCalledWith({
        selected_timeline_node_id: 'node.scene.beach',
        selected_node_id: 'node.character.ada',
        neighborhood_depth: 1,
        max_nodes: 200,
      });
    });
  });

  it('unsubscribes handlers during backend event teardown', async () => {
    const events = new MockServerEventClient();
    const teardown = setupServerEventHandlers(events as never);

    teardown();
    events.emit({ type: 'timeline_changed' });

    await Promise.resolve();

    expect(refreshTimelineRenderProjectionMock).not.toHaveBeenCalled();
  });

  it('clears queued projection refreshes during backend event teardown', async () => {
    const events = new MockServerEventClient();
    const teardown = setupServerEventHandlers(events as never);

    events.emit({ type: 'timeline_changed' });
    teardown();

    await Promise.resolve();

    expect(refreshTimelineRenderProjectionMock).not.toHaveBeenCalled();
  });
});
