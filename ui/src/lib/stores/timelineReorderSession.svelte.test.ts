import { expect, it, vi, beforeEach } from 'vitest';
import { getSessionTimelineReorderDraft } from './timelineReorderSession.svelte.js';
import { editorState, resetEditorState } from './editor.svelte.js';
import { applyTimelineSiblingReorderCommand } from './timelineRenderProjection.svelte.js';
import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import { reordered } from '$lib/components/editor/timelineReorderDraft.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
vi.mock('./timelineRenderProjection.svelte.js', () => ({
  applyTimelineSiblingReorderCommand: vi.fn(),
}));
vi.mock('$lib/projectionApi.js', () => ({ getSelectedNodeEditorProjection: vi.fn() }));
vi.mock('./selectedNodeEditorProjection.svelte.js', () => ({
  refreshSelectedNodeEditorProjection: vi.fn(),
}));
vi.mock('./scriptDocumentProjection.svelte.js', () => ({ invalidateScriptContext: vi.fn() }));
const node: SelectedNodeEditorNode = {
  node_id: 'a',
  name: 'A',
  level: 'Scene',
  sort_order: 1,
  start_ms: 100,
  end_ms: 200,
  notes: 'A',
  content_status: 'HasContent',
  locked: true,
  order_read: {
    node_id: 'a',
    parent_id: 'parent',
    level: 'Scene',
    membership_revision_event_id: null,
    siblings: [
      {
        node_id: 'a',
        name: 'A',
        start_ms: 100,
        end_ms: 200,
        sort_order: 1,
        revision_event_id: null,
      },
      {
        node_id: 'b',
        name: 'B',
        start_ms: 300,
        end_ms: 500,
        sort_order: 2,
        revision_event_id: null,
      },
    ],
  },
};
beforeEach(() => {
  resetEditorState();
  vi.clearAllMocks();
  vi.mocked(applyTimelineSiblingReorderCommand).mockImplementation(async (p) => {
    const r = reordered(p.expected, p.neighbor_id)!;
    for (const n of r.siblings) n.revision_event_id = 'ack';
    return { outcome: 'recorded', projection: {} as never, order_read: r };
  });
  vi.mocked(refreshSelectedNodeEditorProjection).mockResolvedValue({} as never);
});
it('retains per-clip reorder across remount and ends old-session command custody', async () => {
  const draft = getSessionTimelineReorderDraft(node);
  draft.observe(node.order_read);
  draft.state.neighborId = 'b';
  const b = { ...node, node_id: 'b', order_read: { ...node.order_read!, node_id: 'b' } };
  const second = getSessionTimelineReorderDraft(b);
  second.observe(b.order_read);
  second.state.neighborId = 'a';
  expect(getSessionTimelineReorderDraft({ ...node })).toBe(draft);
  expect(draft.state.neighborId).toBe('b');
  expect(second.state.neighborId).toBe('a');
  resetEditorState();
  expect(getSessionTimelineReorderDraft(node)).not.toBe(draft);
  await draft.apply();
  expect(applyTimelineSiblingReorderCommand).not.toHaveBeenCalled();
  expect(draft.state.error).toContain('project changed');
});
it('locked manual structural reorder ACK survives refresh failure and late selection', async () => {
  editorState.selectedNodeId = 'a';
  vi.mocked(refreshSelectedNodeEditorProjection).mockRejectedValueOnce(new Error('Refresh failed'));
  const draft = getSessionTimelineReorderDraft(node);
  draft.observe(node.order_read);
  draft.state.neighborId = 'b';
  await draft.apply();
  expect(draft.state.saved).toBe(true);
  expect(draft.state.uncertain).toBe(false);
  expect(invalidateScriptContext).toHaveBeenCalledTimes(1);
  const guard = vi.mocked(refreshSelectedNodeEditorProjection).mock.calls[0]![1]!;
  expect(guard()).toBe(true);
  editorState.selectedNodeId = 'b';
  expect(guard()).toBe(false);
  resetEditorState();
  editorState.selectedNodeId = 'a';
  expect(guard()).toBe(false);
});
it('background explicit read discards only its draft without retargeting current clip', async () => {
  editorState.selectedNodeId = 'b';
  vi.mocked(getSelectedNodeEditorProjection).mockResolvedValueOnce({ payload: { node } } as never);
  const draft = getSessionTimelineReorderDraft(node);
  draft.observe(node.order_read);
  draft.state.neighborId = 'b';
  await draft.reload();
  expect(draft.state.neighborId).toBeNull();
  expect(editorState.selectedNodeId).toBe('b');
  expect(refreshSelectedNodeEditorProjection).not.toHaveBeenCalled();
  expect(applyTimelineSiblingReorderCommand).not.toHaveBeenCalled();
});
