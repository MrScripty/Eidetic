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

it('retains distinct exact drafts and captured revisions through fresh Script/Graph/Split consumers and selection changes', async () => {
  invoke.mockResolvedValueOnce(projection());
  await refreshScriptDocumentProjection(key);
  const a = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  a.begin(block);
  a.state.text = '  A — 雨\n\nKeep these spaces.  ';
  const other = { ...block, block: { ...block.block, id: 'block.B', text: 'Original B' } };
  const b = getSessionScriptBlockEditDraft(key.document_id, other.block.id);
  b.begin(other);
  b.state.text = 'Independent B draft';
  for (const mode of ['graph', 'split', 'script'] as const) {
    editorState.selectedNodeId = 'scene.B';
    const body = render(AppWorkspace, { props: { workspaceMode: mode } }).body;
    if (mode === 'script') expect(body).toContain('A — 雨');
  }
  expect(getSessionScriptBlockEditDraft(key.document_id, block.block.id)).toBe(a);
  expect(a.state.text).toBe('  A — 雨\n\nKeep these spaces.  ');
  expect(a.state.baseRevision).toBe('event.original');
  a.begin({
    ...block,
    revision_event_id: 'event.newer',
    block: { ...block.block, text: 'Canonical update' },
  });
  expect(a.state.text).toBe('  A — 雨\n\nKeep these spaces.  ');
  a.cancel();
  expect(b.state.text).toBe('Independent B draft');
  expect(b.state.editing).toBe(true);
  expect(invoke).toHaveBeenCalledTimes(1);
});

it('reconciles a late lost acknowledgement after navigation with the exact edit and refreshes canonical memory once acknowledged', async () => {
  let reject!: (error: Error) => void;
  invoke.mockResolvedValueOnce(projection());
  await refreshScriptDocumentProjection(key);
  invoke.mockImplementationOnce(
    () =>
      new Promise((_resolve, fail) => {
        reject = fail;
      }),
  );
  const a = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  a.begin(block);
  a.state.text = '  Preserved — 雨\n\n';
  const revision = scriptDocumentProjectionState.contextRevision;
  const pending = a.save();
  const submitted = structuredClone(invoke.mock.calls[1]);
  expect(render(AppWorkspace, { props: { workspaceMode: 'graph' } }).body).not.toContain(
    'Preserved — 雨',
  );
  reject(new Error('committed acknowledgement lost'));
  await pending;
  const returned = render(AppWorkspace, { props: { workspaceMode: 'script' } }).body;
  expect(returned).toContain('Retry same save');
  expect(returned).toContain('Preserved — 雨');
  expect(returned).toMatch(/<textarea[^>]*disabled/);
  expect(returned.match(/<button([^>]*)>Cancel<\/button>/)?.[1]).toContain('disabled');
  expect(returned).not.toContain('Discard draft and reload');
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision);
  a.cancel();
  a.state.text = 'Mutable caller cannot change submitted text';
  invoke.mockResolvedValueOnce({
    outcome: 'already_recorded',
    projection: projection('  Preserved — 雨\n\n', 2),
  });
  await a.save();
  expect(invoke.mock.calls[2]).toEqual(submitted);
  expect(submitted?.[1].command.payload).toEqual({
    ...key,
    block_id: 'block.A',
    expected_revision_event_id: 'event.original',
    text: '  Preserved — 雨\n\n',
  });
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision + 1);
  expect(getCachedScriptDocumentProjection(key)?.payload.segments[0]?.blocks[0]?.block.text).toBe(
    '  Preserved — 雨\n\n',
  );
  expect(a.state.editing).toBe(false);
});

it.each([
  ['bad_request', 'script block changed; reload before saving'],
  ['conflict', 'script block changed; reload before saving'],
  ['conflict', 'invalid command: script block update would modify locked span text'],
])('retains an editable draft after definite %s refusal: %s', async (kind, message) => {
  const a = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  a.begin(block);
  a.state.text = '  Refused exact draft\n';
  invoke.mockRejectedValueOnce({ kind, message });
  await a.save();
  expect(a.state.uncertain).toBe(false);
  expect(a.state.text).toBe('  Refused exact draft\n');
  const body = render(ScriptBlockEditor, { props: { documentId: key.document_id, block } }).body;
  expect(body).toContain('Discard draft and reload');
  expect(body).not.toMatch(/<textarea[^>]*disabled/);
  const firstId = invoke.mock.calls[0]?.[1].command.id;
  a.state.text = 'Corrected text';
  invoke.mockResolvedValueOnce({
    outcome: 'recorded',
    projection: projection('Corrected text', 2),
  });
  await a.save();
  expect(invoke.mock.calls[1]?.[1].command.id).not.toBe(firstId);
  expect(invoke.mock.calls[1]?.[1].command.payload.text).toBe('Corrected text');
});

it.each(['internal', undefined])(
  'keeps a lookalike refusal uncertain without definite native provenance (%s)',
  async (kind) => {
    const a = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
    a.begin(block);
    invoke.mockRejectedValueOnce({ kind, message: 'script block changed; reload before saving' });
    await a.save();
    a.cancel();
    await a.reload();
    expect(a.state.editing).toBe(true);
    expect(a.state.uncertain).toBe(true);
    expect(invoke).toHaveBeenCalledTimes(1);
  },
);

it('keeps stale exact text on failed explicit reload and discards only after a successful canonical reload', async () => {
  const a = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  a.begin(block);
  a.state.text = 'Draft to retain until reload succeeds';
  invoke.mockRejectedValueOnce({
    kind: 'bad_request',
    message: 'script block changed; reload before saving',
  });
  await a.save();
  invoke.mockRejectedValueOnce(new Error('read failed'));
  await a.reload();
  expect(a.state.editing).toBe(true);
  expect(a.state.text).toBe('Draft to retain until reload succeeds');
  invoke.mockResolvedValueOnce(projection('New canonical text', 3));
  await a.reload();
  expect(a.state.editing).toBe(false);
  expect(a.state.text).toBe('');
  expect(getCachedScriptDocumentProjection(key)?.payload.segments[0]?.blocks[0]?.block.text).toBe(
    'New canonical text',
  );
});

it('isolates retired edit controllers and their late completion from a replacement session', async () => {
  let resolve!: (value: unknown) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const old = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  old.begin(block);
  old.state.text = 'Old submitted draft';
  const pending = old.save();
  resetSessionScriptBlockEditDrafts();
  clearScriptDocumentProjection(key);
  const current = getSessionScriptBlockEditDraft(key.document_id, block.block.id);
  current.begin(block);
  current.state.text = 'New draft';
  resolve({ outcome: 'recorded', projection: projection('Old response', 2) });
  await pending;
  expect(current.state.text).toBe('New draft');
  expect(current.state.editing).toBe(true);
  expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
  old.begin(block);
  await old.save();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(old.state.error).toBe('The project changed before this screenplay edit.');
});
