import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptPanel from './ScriptPanel.svelte';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import { resetSessionScriptBlockCreationDraft } from '$lib/stores/scriptBlockCreationSession.svelte.js';
import {
  clearScriptDocumentProjection,
  refreshScriptDocumentProjection,
  MAIN_SCRIPT_DOCUMENT_ID,
} from '$lib/stores/scriptDocumentProjection.svelte.js';
import { getSessionScriptBlockCreationDraft } from '$lib/stores/scriptBlockCreationSession.svelte.js';
import { copyMissingScriptDraft } from './scriptMissingDraft.js';
import { createScriptBlockCreationDraft } from './scriptBlockCreationDraft.svelte.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
const key = { document_id: MAIN_SCRIPT_DOCUMENT_ID };
const block: ScriptBlockProjection = {
  block: {
    id: 'missing.A',
    segment_id: 'segment.A',
    block_kind: 'dialogue',
    text: 'Saved original',
    sort_order: 0,
  },
  revision_event_id: 'original.revision',
  spans: [],
  locks: [],
};
const exact = '  Keep my dialogue — 雨.\n\n  ';
const invoke = vi.fn();
beforeEach(() => {
  resetSessionScriptBlockEditDrafts();
  resetSessionScriptBlockCreationDraft();
  clearScriptDocumentProjection(key);
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());
it('keeps an exact authored draft reachable after its canonical block disappears', async () => {
  const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  editor.begin(block);
  editor.state.text = exact;
  invoke.mockResolvedValueOnce({
    version: 2,
    payload: { document: { id: key.document_id, title: 'Story', sort_order: 0 }, segments: [] },
  });
  await refreshScriptDocumentProjection(key);
  const body = render(ScriptPanel).body;
  expect(body).toContain('Keep my dialogue — 雨.');
  expect(editor.state.text).toBe(exact);
  expect(editor.state.baseRevision).toBe('original.revision');
  expect(body).toContain('Discard retained draft');
  expect(invoke).toHaveBeenCalledTimes(1);
});

const source: SelectedNodeEditorNode = {
  node_id: 'scene.selected',
  name: 'Explicit selected scene',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  locked: false,
  content_status: 'Empty',
};
function beginMissing() {
  const editor = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  editor.begin(block);
  editor.state.text = exact;
  return editor;
}
it('copies exact text and original kind into the existing composer only explicitly, retaining original across copy cancellation', () => {
  const editor = beginMissing(),
    composer = getSessionScriptBlockCreationDraft();
  expect(copyMissingScriptDraft(editor, composer, null)).toBe(false);
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(true);
  expect(composer.state.text).toBe(exact);
  expect(composer.state.kind).toBe('dialogue');
  expect(composer.state.target?.node_id).toBe(source.node_id);
  composer.state.text = 'Independently edited copy';
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(false);
  expect(composer.state.text).toBe('Independently edited copy');
  composer.cancel();
  expect(editor.state.text).toBe(exact);
  expect(editor.state.baseRevision).toBe('original.revision');
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(true);
  expect(composer.state.text).toBe(exact);
  expect(invoke).not.toHaveBeenCalled();
});
it('blocks repeated copies and cancellation during pending/uncertain creation, retries the immutable copy once, and never retires the original', async () => {
  const editor = beginMissing();
  let reject!: (e: Error) => void;
  const save = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise((_ok, fail) => {
          reject = fail;
        }),
    )
    .mockResolvedValueOnce({});
  const composer = createScriptBlockCreationDraft({
    documentId: () => key.document_id,
    save,
    newCommandId: () => 'exact.copy.request',
  });
  copyMissingScriptDraft(editor, composer, source);
  const pending = composer.save();
  const first = structuredClone(save.mock.calls[0]);
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(false);
  composer.cancel();
  await composer.save();
  expect(save).toHaveBeenCalledTimes(1);
  reject(new Error('Labelled lost acknowledgment'));
  await pending;
  composer.cancel();
  expect(composer.state.writing).toBe(true);
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(false);
  composer.state.text = 'Caller mutation cannot replace submission';
  await composer.save();
  expect(save.mock.calls[1]).toEqual(first);
  expect(first?.[0]).toMatchObject({
    text: exact,
    block_kind: 'dialogue',
    source_node_id: source.node_id,
  });
  expect(editor.state.text).toBe(exact);
  expect(editor.state.editing).toBe(true);
  expect(composer.state.writing).toBe(false);
  editor.cancel();
  expect(editor.state.editing).toBe(false);
});
it('keeps an unavailable original save retry reachable and refuses copy/discard until its exact acknowledgment', async () => {
  const editor = beginMissing();
  invoke.mockRejectedValueOnce(new Error('Labelled lost original acknowledgment'));
  await editor.save();
  const first = structuredClone(invoke.mock.calls[0]);
  invoke.mockResolvedValueOnce({
    version: 2,
    payload: { document: { id: key.document_id, title: 'Story', sort_order: 0 }, segments: [] },
  });
  await refreshScriptDocumentProjection(key);
  const body = render(ScriptPanel).body;
  expect(body).toContain('Retry original save');
  expect(body).toContain('Keep my dialogue — 雨.');
  const composer = getSessionScriptBlockCreationDraft();
  expect(copyMissingScriptDraft(editor, composer, source)).toBe(false);
  editor.cancel();
  expect(editor.state.text).toBe(exact);
  invoke.mockResolvedValueOnce({
    outcome: 'already_recorded',
    projection: {
      version: 3,
      payload: { document: { id: key.document_id, title: 'Story', sort_order: 0 }, segments: [] },
    },
  });
  await editor.save();
  expect(invoke.mock.calls[2]).toEqual(first);
  expect(editor.state.editing).toBe(false);
});
it('preserves independent drafts, fresh panel consumers and read failures, then reattaches the same owner when its block returns', async () => {
  const editor = beginMissing();
  const other = getSessionScriptBlockEditDraft(key.document_id, 'missing.B');
  other.begin({ ...block, block: { ...block.block, id: 'missing.B' } });
  other.state.text = 'Independent B';
  invoke.mockResolvedValueOnce({
    version: 2,
    payload: { document: { id: key.document_id, title: 'Story', sort_order: 0 }, segments: [] },
  });
  await refreshScriptDocumentProjection(key);
  for (let i = 0; i < 2; i++) expect(render(ScriptPanel).body).toContain('Keep my dialogue — 雨.');
  invoke.mockRejectedValueOnce(new Error('Read unavailable'));
  await editor.compare();
  expect(editor.state.text).toBe(exact);
  expect(editor.state.baseRevision).toBe('original.revision');
  expect(editor.state.comparison).toBeNull();
  expect(other.state.text).toBe('Independent B');
  invoke.mockResolvedValueOnce({
    version: 3,
    payload: {
      document: { id: key.document_id, title: 'Story', sort_order: 0 },
      segments: [
        {
          segment: {
            id: 'segment.A',
            document_id: key.document_id,
            source_node_id: source.node_id,
            start_ms: 1000,
            end_ms: 2000,
            status: 'current',
            sort_order: 0,
          },
          blocks: [{ ...block, revision_event_id: 'new.revision' }],
        },
      ],
    },
  });
  await refreshScriptDocumentProjection(key);
  const body = render(ScriptPanel).body;
  expect(body).toContain('Keep my dialogue — 雨.');
  expect(body).toContain('Edit screenplay text');
  expect(getSessionScriptBlockEditDraft(key.document_id, block.block.id)).toBe(editor);
  expect(editor.state.baseRevision).toBe('original.revision');
  editor.cancel();
  expect(other.state.text).toBe('Independent B');
  resetSessionScriptBlockEditDrafts();
  expect(render(ScriptPanel).body).not.toContain('Independent B');
});
