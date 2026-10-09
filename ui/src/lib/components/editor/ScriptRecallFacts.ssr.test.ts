import { beforeEach, expect, it } from 'vitest';
import { render } from 'svelte/server';
import { bibleRecallState, clearBibleRecall } from '$lib/stores/bibleRecallProjection.svelte.js';
import ScriptRecallFacts from './ScriptRecallFacts.svelte';
import { packet } from './scriptRecallSelection.fixture.js';

beforeEach(clearBibleRecall);

it('offers explicit per-preview selection and reports unsupported evidence visibly', () => {
  const data = packet();
  data.nodes[1]!.fields.push({
    part_key: 'profile',
    field_key: 'weather',
    value: { type: 'text', value: 'Rain' },
    source: {
      kind: 'snapshot',
      snapshot_id: 'Rain',
      snapshot_field_id: 'Rain.weather',
      snapshot_revision_event_id: 'snapshot.revision',
      field_revision_event_id: 'field.revision',
      at_ms: 1000,
      label: 'Rain',
    },
  });
  bibleRecallState.projection = { version: 1, payload: data };
  const { body } = render(ScriptRecallFacts, { props: { scope: 'B/session1' } });
  expect(body).toContain('Use recalled facts for this preview (0/8 selected)');
  expect(body).toContain('Existing scene context remains in use.');
  expect(body).toContain('Exact fact 0 — 雨');
  expect(body).toContain('Snapshot-backed; inspection only.');
  expect(body).toContain('Unresolved; cannot select.');
  expect(body).toContain('cannot select omitted records.');
  expect(body).toMatch(/<input[^>]*disabled[^>]*>[^]*?Rain[^]*?Snapshot-backed/);
});

it('makes timed-query and unavailable recall rejection explicit without hiding ordinary preview', () => {
  const data = packet();
  data.request.story_time_ms = 1000;
  bibleRecallState.projection = { version: 1, payload: data };
  const timed = render(ScriptRecallFacts, { props: { scope: 'B/session1' } }).body;
  expect(timed).toContain('Timed recall cannot be used for this preview.');
  expect(timed.match(/<input[^>]*disabled/g)).toHaveLength(9);
  clearBibleRecall();
  const empty = render(ScriptRecallFacts, { props: { scope: 'B/session1' } }).body;
  expect(empty).toContain('story time first.');
});

it('disables related facts when the connecting anchor has unknown name custody', () => {
  const data = packet();
  data.nodes[0]!.name_revision_event_id = null;
  bibleRecallState.projection = { version: 1, payload: data };
  const body = render(ScriptRecallFacts, { props: { scope: 'B/session1' } }).body;
  expect(body).toContain('Connecting anchor name source unknown; cannot select.');
  expect(body.match(/<input[^>]*disabled/g)).toHaveLength(9);
});
