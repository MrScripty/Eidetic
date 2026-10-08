import { beforeEach, afterEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptBlockEditor from './ScriptBlockEditor.svelte';
import ScriptBlockTypeEvidence from './ScriptBlockTypeEvidence.svelte';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import type { ScriptBlockProjection, ScriptContextBlock } from '$lib/scriptTypes.js';

const block: ScriptBlockProjection = {
  block: {
    id: 'A',
    segment_id: 'segment.A',
    block_kind: 'action',
    text: '  Exact line — 雨.\n\n  ',
    sort_order: 0,
  },
  revision_event_id: 'event.A',
  spans: [],
  locks: [],
};
const invoke = vi.fn();
beforeEach(() => {
  resetSessionScriptBlockEditDrafts();
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());

it('retains typed intent in the real session owner and retries an immutable type-only save after a lost acknowledgement', async () => {
  const draft = getSessionScriptBlockEditDraft('main', 'A');
  draft.begin(block);
  draft.state.kind = 'dialogue';
  const body = render(ScriptBlockEditor, { props: { documentId: 'main', block } }).body;
  expect(body).toContain('Block type');
  expect(body).toContain('value="dialogue" selected');
  invoke.mockRejectedValueOnce(new Error('lost acknowledgement'));
  await draft.save();
  const submitted = structuredClone(invoke.mock.calls[0]);
  expect(submitted?.[1].command.payload).toEqual({
    document_id: 'main',
    block_id: 'A',
    expected_revision_event_id: 'event.A',
    text: block.block.text,
    block_kind: 'dialogue',
  });
  const uncertain = render(ScriptBlockEditor, { props: { documentId: 'main', block } }).body;
  expect(uncertain).toMatch(/<select[^>]*disabled/);
  expect(uncertain).toContain('Retry same save');
  expect(getSessionScriptBlockEditDraft('main', 'A')).toBe(draft);
  draft.state.kind = 'shot';
  draft.state.text = 'Mutable later draft';
  invoke.mockResolvedValueOnce({
    outcome: 'already_recorded',
    projection: {
      version: 2,
      payload: { document: { id: 'main', title: 'Story', sort_order: 0 }, segments: [] },
    },
  });
  await draft.save();
  expect(invoke.mock.calls[1]).toEqual(submitted);
});

it('compares saved type without silently rebasing either draft value and explicitly continues only from the read version', async () => {
  const draft = getSessionScriptBlockEditDraft('main', 'A');
  draft.begin(block);
  draft.state.kind = 'dialogue';
  invoke.mockRejectedValueOnce({
    kind: 'conflict',
    message: 'cannot change the type of a locked script block',
  });
  await draft.save();
  expect(draft.state.uncertain).toBe(false);
  expect(draft.state.kind).toBe('dialogue');
  const current: ScriptBlockProjection = {
    ...block,
    revision_event_id: 'event.B',
    block: { ...block.block, block_kind: 'shot', text: 'New canonical text' },
  };
  invoke.mockResolvedValueOnce({
    version: 2,
    payload: {
      document: { id: 'main', title: 'Story', sort_order: 0 },
      segments: [{ segment: { id: 'segment.A' }, blocks: [current] }],
    },
  });
  await draft.compare();
  expect(draft.state.baseRevision).toBe('event.A');
  expect(draft.state.kind).toBe('dialogue');
  expect(draft.state.text).toBe(block.block.text);
  const body = render(ScriptBlockEditor, { props: { documentId: 'main', block: current } }).body;
  expect(body).toContain('Saved block type: shot');
  draft.state.comparison!.kind = 'note';
  draft.useComparedRevision(current);
  expect(draft.state.baseRevision).toBe('event.B');
  expect(draft.state.kind).toBe('dialogue');
  invoke.mockRejectedValueOnce({
    kind: 'conflict',
    message: 'script block changed; reload before saving',
  });
  await draft.save();
  expect(invoke.mock.calls[2]?.[1].command.payload.block_kind).toBe('dialogue');
  expect(invoke.mock.calls[2]?.[1].command.payload.expected_revision_event_id).toBe('event.B');
});

it('renders original/current owned type evidence and exact text, while legacy unknown and unchanged types remain absent', () => {
  const original: ScriptContextBlock = {
    document_id: 'main',
    segment_id: 's',
    block_id: 'A',
    source_node_id: null,
    revision_event_id: 'event.A',
    segment_revision_event_id: 'event.S',
    start_ms: 0,
    end_ms: 1,
    text: block.block.text,
    block_kind: 'action',
  };
  const current = { ...original, revision_event_id: 'event.B', block_kind: 'dialogue' as const };
  const body = render(ScriptBlockTypeEvidence, {
    props: { previous: [original], current: [current] },
  }).body;
  expect(body).toContain('Originally consumed block type');
  expect(body).toContain('>action</pre>');
  expect(body).toContain('>dialogue</pre>');
  expect(body).toContain('  Exact line — 雨.\n\n  ');
  expect(
    render(ScriptBlockTypeEvidence, {
      props: { previous: [{ ...original, block_kind: null }], current: [current] },
    }).body,
  ).not.toContain('Consumed screenplay type changed');
  expect(
    render(ScriptBlockTypeEvidence, { props: { previous: [original], current: [original] } }).body,
  ).not.toContain('Consumed screenplay type changed');
});
