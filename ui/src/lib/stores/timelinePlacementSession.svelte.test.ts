import { expect, it, vi } from 'vitest';
import { getSessionTimelinePlacementDraft } from './timelinePlacementSession.svelte.js';
import { resetEditorState } from './editor.svelte.js';
import { applyTimelineNodeRangeCommand } from './timelineRenderProjection.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';

vi.mock('./timelineRenderProjection.svelte.js', () => ({
  applyTimelineNodeRangeCommand: vi.fn().mockResolvedValue({}),
}));
const node: SelectedNodeEditorNode = {
  node_id: 'A',
  name: 'A',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  content_status: 'HasContent',
  locked: false,
  range_read: { start_ms: 1000, end_ms: 2000, node_revision_event_id: 'write' },
};

it('keeps a placement intent through panel/selection replacement and excludes old project callers', async () => {
  resetEditorState();
  const first = getSessionTimelinePlacementDraft(node);
  first.initialize(node.range_read);
  first.state.start = '6';
  first.state.end = '7';
  getSessionTimelinePlacementDraft({ ...node, node_id: 'B' });
  expect(getSessionTimelinePlacementDraft({ ...node })).toBe(first);
  expect(first.state.start).toBe('6');
  resetEditorState();
  expect(getSessionTimelinePlacementDraft(node)).not.toBe(first);
  await first.apply();
  expect(applyTimelineNodeRangeCommand).not.toHaveBeenCalled();
  expect(first.state.error).toContain('project changed');
});
