import { expect, it, vi, beforeEach } from 'vitest';
import { getSessionTimelineNotesDraft } from './timelineNotesSession.svelte.js';
import { editorState, resetEditorState } from './editor.svelte.js';
import { applyTimelineNodeNotesCommand } from './timelineRenderProjection.svelte.js';
import { getSelectedNodeEditorProjection } from '$lib/projectionApi.js';
import { refreshSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import { invalidateScriptContext } from './scriptDocumentProjection.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
vi.mock('./timelineRenderProjection.svelte.js', () => ({ applyTimelineNodeNotesCommand: vi.fn() }));
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
  notes_read: { node_id: 'A', notes: 'Saved A', revision_event_id: null },
};
beforeEach(() => {
  resetEditorState();
  vi.clearAllMocks();
  vi.mocked(applyTimelineNodeNotesCommand).mockResolvedValue({
    outcome: 'recorded',
    projection: {} as never,
    notes_read: { ...node.notes_read!, notes: 'Accepted', revision_event_id: 'ack' },
  });
  vi.mocked(refreshSelectedNodeEditorProjection).mockResolvedValue({} as never);
});
it('retains two exact clip drafts through disposal/re-entry without saving; reopening clears custody', async () => {
  const first = getSessionTimelineNotesDraft(node);
  first.observe(node.notes_read);
  first.state.notes = '  Draft A — 雨.\n\n';
  const b = { ...node, node_id: 'B', notes_read: { ...node.notes_read!, node_id: 'B' } };
  const second = getSessionTimelineNotesDraft(b);
  second.observe(b.notes_read);
  second.state.notes = 'Draft B';
  expect(getSessionTimelineNotesDraft({ ...node })).toBe(first);
  expect(first.state.notes).toBe('  Draft A — 雨.\n\n');
  expect(second.state.notes).toBe('Draft B');
  expect(applyTimelineNodeNotesCommand).not.toHaveBeenCalled();
  resetEditorState();
  const reopened = getSessionTimelineNotesDraft(node);
  reopened.observe(node.notes_read);
  expect(reopened).not.toBe(first);
  expect(reopened.state.notes).toBe('Saved A');
  await first.apply();
  expect(applyTimelineNodeNotesCommand).not.toHaveBeenCalled();
  expect(first.state.error).toContain('project changed');
});
it('keeps acknowledged save distinct from failed refresh and guards selection/session', async () => {
  editorState.selectedNodeId = 'A';
  vi.mocked(refreshSelectedNodeEditorProjection).mockRejectedValueOnce(
    new Error('Refresh unavailable'),
  );
  const draft = getSessionTimelineNotesDraft(node);
  draft.observe(node.notes_read);
  draft.state.notes = 'Accepted';
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
    payload: { node: { ...node, notes_read: { ...node.notes_read!, notes: 'Current' } } },
  } as never);
  const draft = getSessionTimelineNotesDraft(node);
  draft.observe(node.notes_read);
  draft.state.notes = 'Unsaved';
  await draft.reload();
  expect(draft.state.notes).toBe('Current');
  expect(editorState.selectedNodeId).toBe('B');
  expect(refreshSelectedNodeEditorProjection).not.toHaveBeenCalled();
  expect(applyTimelineNodeNotesCommand).not.toHaveBeenCalled();
});
it('rejects old-project read completion without replacing retained text', async () => {
  let resolve!: (value: never) => void;
  vi.mocked(getSelectedNodeEditorProjection).mockReturnValueOnce(
    new Promise((done) => {
      resolve = done;
    }),
  );
  const draft = getSessionTimelineNotesDraft(node);
  draft.observe(node.notes_read);
  draft.state.notes = 'Unsaved';
  const pending = draft.reload();
  resetEditorState();
  resolve({ payload: { node } } as never);
  await pending;
  expect(draft.state.notes).toBe('Unsaved');
  expect(draft.state.error).toContain('project changed');
});
