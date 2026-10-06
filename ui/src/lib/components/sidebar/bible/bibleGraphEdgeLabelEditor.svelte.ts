import type { BibleGraphEdge, SetBibleGraphEdgeLabelCommand } from '$lib/bibleGraphTypes.js';

export function createBibleGraphEdgeLabelEditor(options: {
  edges: () => BibleGraphEdge[];
  revisions: () => Record<string, string>;
  save: (edge: BibleGraphEdge, command: SetBibleGraphEdgeLabelCommand) => Promise<unknown>;
  remove: (edge: BibleGraphEdge) => Promise<unknown>;
  refresh: (edge: BibleGraphEdge) => Promise<unknown>;
  confirm: (edge: BibleGraphEdge) => boolean;
}) {
  const state = $state({
    edge: null as BibleGraphEdge | null,
    revision: null as string | null,
    label: '',
    saving: false,
    deleting: null as string | null,
    error: undefined as string | undefined,
  });
  const busy = () => state.saving || state.deleting !== null;
  function edit(edge: BibleGraphEdge) {
    if (busy()) return;
    const revision = options.revisions()[edge.id];
    if (!revision) {
      state.error = 'Refresh the relationship before editing its label.';
      return;
    }
    state.edge = JSON.parse(JSON.stringify(edge)) as BibleGraphEdge;
    state.revision = revision;
    state.label = edge.label;
    state.error = undefined;
  }
  function cancel() {
    if (busy()) return;
    state.edge = null;
    state.revision = null;
  }
  async function save() {
    if (busy() || !state.edge || !state.revision || !state.label.trim()) return;
    const edge = options.edges().find((edge) => edge.id === state.edge?.id);
    if (!edge || options.revisions()[edge.id] !== state.revision) {
      state.error = 'Relationship changed while editing; reopen its label editor.';
      return;
    }
    state.saving = true;
    state.error = undefined;
    try {
      await options.save(edge, {
        edge_id: edge.id,
        label: state.label.trim(),
        expected_revision_event_id: state.revision,
      });
      await options.refresh(edge);
      state.edge = null;
      state.revision = null;
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'Failed to save relationship label';
    } finally {
      state.saving = false;
    }
  }
  async function remove(edge: BibleGraphEdge) {
    if (busy() || !options.confirm(edge)) return;
    state.deleting = edge.id;
    state.error = undefined;
    try {
      await options.remove(edge);
      await options.refresh(edge);
      // Keep an open label draft visible after deletion. Saving it will refuse.
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'Failed to delete edge';
    } finally {
      state.deleting = null;
    }
  }
  return { state, busy, edit, cancel, save, remove };
}
