import { render } from 'svelte/server';
import { expect, it, vi } from 'vitest';
import TimelineNotesEditor from './TimelineNotesEditor.svelte';
import { createTimelineNotesDraft } from './timelineNotesDraft.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
let owner: ReturnType<typeof createTimelineNotesDraft>;
vi.mock('$lib/stores/timelineNotesSession.svelte.js', () => ({
  getSessionTimelineNotesDraft: () => owner,
}));
const node: SelectedNodeEditorNode = {
  node_id: 'A',
  name: 'A',
  level: 'Scene',
  sort_order: 0,
  start_ms: 0,
  end_ms: 1000,
  notes: 'Canonical',
  content_status: 'HasContent',
  locked: false,
  notes_read: { node_id: 'A', notes: 'Canonical', revision_event_id: null },
};
function fixture() {
  owner = createTimelineNotesDraft({
    nodeId: 'A',
    read: vi.fn(),
    apply: vi.fn(),
    commandId: () => 'save',
  });
  owner.observe(node.notes_read);
  owner.state.notes = '  Exact draft — 雨.\n\n  ';
  return owner;
}
it('renders retained exact draft, explicit save and discard controls', () => {
  fixture();
  const html = render(TimelineNotesEditor, { props: { node } }).body;
  expect(html).toContain('  Exact draft — 雨.\n\n  ');
  expect(html).toContain('Save Notes');
  expect(html).toContain('Discard draft and read saved Notes');
  expect(html).toContain('Unsaved Notes draft');
});
it('shows canonical conflict separately while retaining the author draft', () => {
  fixture().observe({ ...node.notes_read!, notes: 'Concurrent Notes', revision_event_id: 'new' });
  const html = render(TimelineNotesEditor, { props: { node } }).body;
  expect(html).toContain('Exact draft');
  expect(html).toContain('Current saved Notes');
  expect(html).toContain('Concurrent Notes');
  expect(html).toMatch(/<button[^>]*disabled[^>]*>\s*Save Notes/);
});
it('permits immutable uncertain retry after a late lock, but keeps discard and text disabled', () => {
  fixture().state.uncertain = true;
  const html = render(TimelineNotesEditor, { props: { node: { ...node, locked: true } } }).body;
  expect(html).toMatch(/<button(?![^>]*disabled)[^>]*>\s*Retry exact save/);
  expect(html).toMatch(/<textarea[^>]*disabled/);
  expect(html).toMatch(/<button[^>]*disabled[^>]*>\s*Discard draft/);
});
it('blocks a fresh save on a locked clip', () => {
  fixture();
  const html = render(TimelineNotesEditor, { props: { node: { ...node, locked: true } } }).body;
  expect(html).toMatch(/<button[^>]*disabled[^>]*>\s*Save Notes/);
});
