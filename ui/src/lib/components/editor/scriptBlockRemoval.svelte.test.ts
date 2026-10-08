import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import AppWorkspace from '../layout/AppWorkspace.svelte';
import ScriptBlockEditor from './ScriptBlockEditor.svelte';
import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import {
  clearScriptDocumentProjection,
  getCachedScriptDocumentProjection,
  refreshScriptDocumentProjection,
  scriptDocumentProjectionState,
} from '$lib/stores/scriptDocumentProjection.svelte.js';
import { editorState, resetEditorState } from '$lib/stores/editor.svelte.js';

const key = { document_id: 'script.document.main' };
const block: ScriptBlockProjection = {
  block: {
    id: 'block.A',
    segment_id: 'segment.A',
    block_kind: 'action',
    text: 'Original A',
    sort_order: 0,
  },
  revision_event_id: 'event.original',
  spans: [],
  locks: [],
};
function projection(text = 'Original A', version = 1) {
  return {
    version,
    payload: {
      document: { id: key.document_id, title: 'Story', sort_order: 0 },
      segments: [
        {
          segment: {
            id: 'segment.A',
            document_id: key.document_id,
            source_node_id: 'scene.A',
            start_ms: 1000,
            end_ms: 2000,
            status: 'current',
            sort_order: 0,
          },
          blocks: [
            { ...block, revision_event_id: `event.${version}`, block: { ...block.block, text } },
          ],
        },
      ],
    },
  };
}
const invoke = vi.fn();
beforeEach(() => {
  resetSessionScriptBlockEditDrafts();
  clearScriptDocumentProjection(key);
  resetEditorState();
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());

function removedProjection(version = 2) {
  const value = projection('', version);
  value.payload.segments[0]!.blocks = [];
  return value;
}

it('requires explicit confirmation, preserves manual draft ownership, and sends only captured removal identity', async () => {
  const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  editor.begin(block);
  editor.state.text = '  Unsaved manual draft — 雨\n';
  editor.beginRemoval(block);
  expect(editor.state.removal.active).toBe(false);
  expect(editor.state.text).toBe('  Unsaved manual draft — 雨\n');
  editor.cancel();
  editor.beginRemoval(block);
  expect(invoke).not.toHaveBeenCalled();
  const body = render(ScriptBlockEditor, { props: { documentId: key.document_id, block } }).body;
  expect(body).toContain('Remove this saved block?');
  expect(body).toContain('Original A');
  expect(body).toContain('Keep block');
  editor.begin({ ...block, revision_event_id: 'event.other' });
  expect(editor.state.editing).toBe(false);
  editor.cancelRemoval();
  expect(editor.state.removal.active).toBe(false);
  editor.beginRemoval(block);
  invoke.mockResolvedValueOnce({ outcome: 'recorded', projection: removedProjection() });
  const revision = scriptDocumentProjectionState.contextRevision;
  await editor.remove();
  expect(invoke.mock.calls[0]?.[0]).toBe('command_script_block_remove');
  expect(invoke.mock.calls[0]?.[1].command.payload).toEqual({
    ...key,
    block_id: block.block.id,
    expected_revision_event_id: 'event.original',
  });
  expect(editor.state.removal.active).toBe(false);
  expect(getCachedScriptDocumentProjection(key)?.payload.segments[0]?.blocks).toEqual([]);
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision + 1);
});

it('retains an exact uncertain removal through navigation and canonical disappearance, then retries its immutable receipt', async () => {
  invoke.mockResolvedValueOnce(projection());
  await refreshScriptDocumentProjection(key);
  const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  editor.beginRemoval(block);
  invoke.mockRejectedValueOnce(new Error('acknowledgement lost after commit'));
  await editor.remove();
  const submitted = structuredClone(invoke.mock.calls[1]);
  editor.cancelRemoval();
  editor.begin(block);
  expect(editor.state.removal.active).toBe(true);
  expect(editor.state.editing).toBe(false);
  invoke.mockResolvedValueOnce(removedProjection());
  await refreshScriptDocumentProjection(key);
  for (const mode of ['graph', 'split', 'script'] as const) {
    const body = render(AppWorkspace, { props: { workspaceMode: mode } }).body;
    editorState.selectedNodeId = 'scene.B';
    expect(getSessionScriptBlockEditDraft(key.document_id, block.block.id)).toBe(editor);
    if (mode === 'script') {
      expect(body).toContain('Retry same removal');
      expect(body).toContain('Original A');
      expect(body.match(/<button([^>]*)>Keep block<\/button>/)?.[1]).toContain('disabled');
    }
  }
  editor.state.removal.text = 'Mutable display text';
  invoke.mockResolvedValueOnce({ outcome: 'already_recorded', projection: removedProjection() });
  await editor.remove();
  expect(invoke.mock.calls[3]).toEqual(submitted);
  expect(editor.state.removal.active).toBe(false);
});

it.each(['script block changed; reload before removing', 'cannot remove a locked script block'])(
  'keeps exact removal text on definite refusal: %s',
  async (message) => {
    const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
    editor.beginRemoval(block);
    invoke.mockRejectedValueOnce({ kind: 'bad_request', message });
    await editor.remove();
    expect(editor.state.removal.uncertain).toBe(false);
    expect(editor.state.removal.text).toBe('Original A');
    expect(editor.state.removal.active).toBe(true);
    editor.cancelRemoval();
    editor.begin(block);
    expect(editor.state.editing).toBe(true);
  },
);

it('blocks locked removal and does not trust lookalike refusal messages', async () => {
  const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  editor.beginRemoval({
    ...block,
    locks: [{ id: 'lock.A', span_id: 'span.A', reason: 'Protected' }],
  });
  expect(editor.state.removal.active).toBe(false);
  editor.beginRemoval(block);
  invoke.mockRejectedValueOnce({
    kind: 'internal',
    message: 'script block changed; reload before removing',
  });
  await editor.remove();
  editor.cancelRemoval();
  expect(editor.state.removal.uncertain).toBe(true);
  expect(editor.state.removal.active).toBe(true);
});

it('isolates late removal responses and retired retries from replacement project sessions', async () => {
  let resolve!: (value: unknown) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const old = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  old.beginRemoval(block);
  const pending = old.remove();
  resetSessionScriptBlockEditDrafts();
  clearScriptDocumentProjection(key);
  const current = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  current.begin(block);
  current.state.text = 'Current project draft';
  resolve({ outcome: 'recorded', projection: removedProjection() });
  await pending;
  expect(current.state.text).toBe('Current project draft');
  expect(current.state.editing).toBe(true);
  expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
  old.beginRemoval(block);
  await old.remove();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(old.state.removal.error).toBe('The project changed before this screenplay edit.');
});
