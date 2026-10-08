import { expect, it, vi, beforeEach } from 'vitest';
import { getSessionTimelineArcsDraft } from './timelineArcsSession.svelte.js';
import { editorState, resetEditorState } from './editor.svelte.js';
import { applyTimelineNodeArcsCommand } from './timelineRenderProjection.svelte.js';
import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
vi.mock('./timelineRenderProjection.svelte.js', () => ({ applyTimelineNodeArcsCommand: vi.fn() }));
vi.mock('$lib/projectionApi.js', () => ({ getSelectedNodeEditorProjection: vi.fn() }));
vi.mock('./selectedNodeEditorProjection.svelte.js', () => ({
  refreshSelectedNodeEditorProjection: vi.fn(),
}));
vi.mock('./scriptDocumentProjection.svelte.js', () => ({ invalidateScriptContext: vi.fn() }));
const node: SelectedNodeEditorNode = {
  node_id: 'A',
  name: 'A',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: 'Saved A',
  content_status: 'HasContent',
  locked: false,
  arc_read: { node_id: 'A', arc_ids: [], revision_event_id: null },
};
beforeEach(() => {
  resetEditorState();
  vi.clearAllMocks();
  vi.mocked(applyTimelineNodeArcsCommand).mockResolvedValue({
    outcome: 'recorded',
    projection: {} as never,
    arc_read: { ...node.arc_read!, arc_ids: ['Accepted'], revision_event_id: 'ack' },
  });
  vi.mocked(refreshSelectedNodeEditorProjection).mockResolvedValue({} as never);
});
it('retains two exact clip drafts through disposal/re-entry without saving; reopening clears custody', async () => {
  const first = getSessionTimelineArcsDraft(node);
  first.observe(node.arc_read);
  first.state.arcIds = ['Draft A'];
  const b = { ...node, node_id: 'B', arc_read: { ...node.arc_read!, node_id: 'B' } };
  const second = getSessionTimelineArcsDraft(b);
  second.observe(b.arc_read);
  second.state.arcIds = ['Draft B'];
  expect(getSessionTimelineArcsDraft({ ...node })).toBe(first);
  expect(first.state.arcIds).toEqual(['Draft A']);
  expect(second.state.arcIds).toEqual(['Draft B']);
  expect(applyTimelineNodeArcsCommand).not.toHaveBeenCalled();
  resetEditorState();
  const reopened = getSessionTimelineArcsDraft(node);
  reopened.observe(node.arc_read);
  expect(reopened).not.toBe(first);
  expect(reopened.state.arcIds).toEqual([]);
  await first.apply();
  expect(applyTimelineNodeArcsCommand).not.toHaveBeenCalled();
  expect(first.state.error).toContain('project changed');
});
it('keeps acknowledged save distinct from failed refresh and guards selection/session', async () => {
  editorState.selectedNodeId = 'A';
  vi.mocked(refreshSelectedNodeEditorProjection).mockRejectedValueOnce(
    new Error('Refresh unavailable'),
  );
  const draft = getSessionTimelineArcsDraft(node);
  draft.observe(node.arc_read);
  draft.state.arcIds = ['Accepted'];
  await draft.apply();
  expect(draft.state.saved).toBe(true);
  expect(draft.state.uncertain).toBe(false);
  expect(invalidateScriptContext).toHaveBeenCalledTimes(1);
  const guard = vi.mocked(refreshSelectedNodeEditorProjection).mock.calls[0]![1]!;
  expect(guard()).toBe(true);
  editorState.selectedNodeId = 'B';
  expect(guard()).toBe(false);
  resetEditorState();
  editorState.selectedNodeId = 'A';
  expect(guard()).toBe(false);
});
it('reads a background owner without retargeting selection or writing', async () => {
  editorState.selectedNodeId = 'B';
  vi.mocked(getSelectedNodeEditorProjection).mockResolvedValueOnce({
    payload: { node: { ...node, arc_read: { ...node.arc_read!, arc_ids: ['Current'] } } },
  } as never);
  const draft = getSessionTimelineArcsDraft(node);
  draft.observe(node.arc_read);
  draft.state.arcIds = ['Unsaved'];
  await draft.reload();
  expect(draft.state.arcIds).toEqual(['Current']);
  expect(editorState.selectedNodeId).toBe('B');
  expect(refreshSelectedNodeEditorProjection).not.toHaveBeenCalled();
  expect(applyTimelineNodeArcsCommand).not.toHaveBeenCalled();
});
it('rejects old-project read completion without replacing retained text', async () => {
  let resolve!: (value: never) => void;
  vi.mocked(getSelectedNodeEditorProjection).mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  const draft = getSessionTimelineArcsDraft(node);
  draft.observe(node.arc_read);
  draft.state.arcIds = ['Unsaved'];
  const pending = draft.reload();
  resetEditorState();
  resolve({ payload: { node } } as never);
  await pending;
  expect(draft.state.arcIds).toEqual(['Unsaved']);
  expect(draft.state.error).toContain('project changed');
});
