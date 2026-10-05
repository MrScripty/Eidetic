import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptBlockEditor from './ScriptBlockEditor.svelte';
import AppWorkspace from '../layout/AppWorkspace.svelte';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import {
  clearScriptDocumentProjection,
  getCachedScriptDocumentProjection,
  scriptDocumentProjectionState,
} from '$lib/stores/scriptDocumentProjection.svelte.js';
import type { ScriptBlockProjection } from '$lib/scriptTypes.js';

const key = { document_id: 'script.document.main' };
function block(
  text = 'Original saved text',
  revision = 'event.1',
  id = 'block.A',
): ScriptBlockProjection {
  return {
    block: { id, segment_id: 'segment.A', block_kind: 'action', text, sort_order: 0 },
    revision_event_id: revision,
    spans: [],
    locks: [],
  };
}
function projection(current = block('Other author — 雪\n', 'event.2'), version = 2) {
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
          blocks: [current],
        },
      ],
    },
  };
}
const invoke = vi.fn();
beforeEach(() => {
  resetSessionScriptBlockEditDrafts();
  clearScriptDocumentProjection(key);
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());
function begin() {
  const draft = getSessionScriptBlockEditDraft(key.document_id, 'block.A');
  draft.begin(block());
  draft.state.text = '  My exact draft — 雨\n\n  ';
  return draft;
}

it('compares a stale refused draft through real read/store helpers, retains it across workspace consumers and explicitly saves against the read version', async () => {
  const draft = begin();
  const revision = scriptDocumentProjectionState.contextRevision;
  invoke.mockRejectedValueOnce({
    kind: 'bad_request',
    message: 'script block changed; reload before saving',
  });
  await draft.save();
  const refusedId = invoke.mock.calls[0]?.[1].command.id;
  const current = block('Other author — 雪\n', 'event.2');
  invoke.mockResolvedValueOnce(projection(current));
  await draft.compare();
  expect(draft.state.baseRevision).toBe('event.1');
  expect(draft.state.text).toBe('  My exact draft — 雨\n\n  ');
  expect(invoke.mock.calls[1]?.[0]).toBe('projection_script_document');
  for (const mode of ['graph', 'split', 'script'] as const) {
    const body = render(AppWorkspace, { props: { workspaceMode: mode } }).body;
    if (mode === 'script') {
      expect(body).toContain('Saved text comparison');
      expect(body).toContain('Other author — 雪');
      expect(body).toContain('My exact draft — 雨');
    }
  }
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision);
  draft.useComparedRevision(current);
  expect(draft.state.baseRevision).toBe('event.2');
  expect(draft.state.text).toBe('  My exact draft — 雨\n\n  ');
  expect(invoke).toHaveBeenCalledTimes(2);
  invoke.mockResolvedValueOnce({
    outcome: 'recorded',
    projection: projection(block(draft.state.text, 'event.3'), 3),
  });
  await draft.save();
  const command = invoke.mock.calls[2]?.[1].command;
  expect(command.id).not.toBe(refusedId);
  expect(command.payload).toEqual({
    ...key,
    block_id: 'block.A',
    expected_revision_event_id: 'event.2',
    text: '  My exact draft — 雨\n\n  ',
  });
  expect(scriptDocumentProjectionState.contextRevision).toBe(revision + 1);
  expect(getCachedScriptDocumentProjection(key)?.payload.segments[0]?.blocks[0]?.block.text).toBe(
    '  My exact draft — 雨\n\n  ',
  );
});

it('requires another comparison when the canonical revision changes, including same-text ABA', async () => {
  const draft = begin();
  invoke.mockResolvedValueOnce(projection());
  await draft.compare();
  const newer = block('Other author — 雪\n', 'event.3');
  const body = render(ScriptBlockEditor, {
    props: { documentId: key.document_id, block: newer },
  }).body;
  expect(body).toContain('Saved text changed again.');
  expect(body.match(/<button([^>]*)>Continue draft from this version<\/button>/)?.[1]).toContain(
    'disabled',
  );
  draft.useComparedRevision(newer);
  expect(draft.state.baseRevision).toBe('event.1');
  expect(draft.state.comparison).toBeNull();
  expect(draft.state.text).toBe('  My exact draft — 雨\n\n  ');
  invoke.mockResolvedValueOnce(projection(newer, 3));
  await draft.compare();
  draft.useComparedRevision(newer);
  expect(draft.state.baseRevision).toBe('event.3');
  expect(invoke).toHaveBeenCalledTimes(2);
});

it('preserves the exact draft when an edit after explicit continuation is refused, then retries from a newly read version', async () => {
  const draft = begin();
  const current = block('Latest', 'event.2');
  invoke.mockResolvedValueOnce(projection(current));
  await draft.compare();
  draft.useComparedRevision(current);
  invoke.mockRejectedValueOnce({
    kind: 'bad_request',
    message: 'script block changed; reload before saving',
  });
  await draft.save();
  expect(draft.state.editing).toBe(true);
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.text).toBe('  My exact draft — 雨\n\n  ');
  const latest = block('Newer again', 'event.3');
  invoke.mockResolvedValueOnce(projection(latest, 3));
  await draft.compare();
  draft.useComparedRevision(latest);
  invoke.mockResolvedValueOnce({
    outcome: 'recorded',
    projection: projection(block(draft.state.text, 'event.4'), 4),
  });
  await draft.save();
  expect(invoke.mock.calls[3]?.[1].command.payload.expected_revision_event_id).toBe('event.3');
});

it.each(['missing', 'failed'])(
  'preserves exact text and captured revision when comparison is %s',
  async (failure) => {
    const draft = begin();
    if (failure === 'missing')
      invoke.mockResolvedValueOnce(projection(block('Other block', 'event.2', 'block.B')));
    else invoke.mockRejectedValueOnce(new Error('read failed'));
    await draft.compare();
    expect(draft.state.comparison).toBeNull();
    expect(draft.state.baseRevision).toBe('event.1');
    expect(draft.state.text).toBe('  My exact draft — 雨\n\n  ');
    expect(draft.state.editing).toBe(true);
    expect(draft.state.comparing).toBe(false);
  },
);

it('guards pending comparison controls and retains a late read failure across fresh consumers', async () => {
  const draft = begin();
  let reject!: (error: Error) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((_done, fail) => {
        reject = fail;
      }),
  );
  const pending = draft.compare();
  expect(draft.state.comparing).toBe(true);
  const body = render(ScriptBlockEditor, {
    props: { documentId: key.document_id, block: block() },
  }).body;
  expect(body).toContain('Reading saved text…');
  expect(body).toMatch(/<textarea[^>]*disabled/);
  draft.cancel();
  await draft.reload();
  await draft.save();
  await draft.compare();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(render(AppWorkspace, { props: { workspaceMode: 'graph' } }).body).not.toContain(
    'Reading saved text…',
  );
  reject(new Error('late read failed'));
  await pending;
  const returned = render(ScriptBlockEditor, {
    props: { documentId: key.document_id, block: block() },
  }).body;
  expect(returned).toContain('late read failed');
  expect(returned).toContain('My exact draft — 雨');
});

it('keeps comparison and continuation unavailable during uncertain save and retries the immutable original request', async () => {
  const draft = begin();
  invoke.mockRejectedValueOnce(new Error('acknowledgement lost'));
  await draft.save();
  const submitted = structuredClone(invoke.mock.calls[0]);
  await draft.compare();
  draft.useComparedRevision(block('Unread', 'event.2'));
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(draft.state.baseRevision).toBe('event.1');
  expect(draft.state.uncertain).toBe(true);
  draft.state.text = 'Caller mutation must not replace submission';
  invoke.mockResolvedValueOnce({
    outcome: 'already_recorded',
    projection: projection(block('  My exact draft — 雨\n\n  ', 'event.2')),
  });
  await draft.save();
  expect(invoke.mock.calls[1]).toEqual(submitted);
});

it('does not admit an unread revision through mutable display state', async () => {
  const draft = begin();
  invoke.mockResolvedValueOnce(projection());
  await draft.compare();
  if (!draft.state.comparison) throw new Error('comparison missing');
  draft.state.comparison.revisionEventId = 'event.forged';
  draft.useComparedRevision(block('Unread', 'event.forged'));
  expect(draft.state.baseRevision).toBe('event.1');
  expect(draft.state.comparison).toBeNull();
  expect(invoke).toHaveBeenCalledTimes(1);
});

it('keeps a retired comparison response out of the replacement session draft and cache', async () => {
  const old = begin();
  let resolve!: (value: unknown) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const pending = old.compare();
  resetSessionScriptBlockEditDrafts();
  clearScriptDocumentProjection(key);
  const current = begin();
  current.state.text = 'New session draft';
  resolve(projection());
  await pending;
  expect(current.state.text).toBe('New session draft');
  expect(current.state.comparison).toBeNull();
  expect(getCachedScriptDocumentProjection(key)).toBeUndefined();
  await old.compare();
  expect(invoke).toHaveBeenCalledTimes(1);
  expect(old.state.error).toBe('The project changed before this screenplay edit.');
});
