import { beforeEach, expect, it } from 'vitest';
import { render } from 'svelte/server';
import BibleRecall from './BibleRecall.svelte';
import { bibleRecallState, clearBibleRecall } from '$lib/stores/bibleRecallProjection.svelte.js';

beforeEach(clearBibleRecall);
it('shows typed path, exact field sources, fictional-time qualifications and whole-record omissions', () => {
  bibleRecallState.anchor = 'Mara';
  bibleRecallState.projection = {
    version: 210,
    change_event_id: 'read-event',
    payload: {
      request: {
        anchor_node_id: 'Mara',
        story_time_ms: 1000,
        direction: 'both',
        edge_kinds: [],
        neighbor_limit: 8,
      },
      nodes: [
        {
          node_id: 'Mara',
          name: 'Mara',
          schema_key: 'character',
          name_revision_event_id: 'Mara-name',
          fields: [],
          unresolved_timed_fields: [],
          omitted_fields: 0,
        },
        {
          node_id: 'BeachHouse',
          name: 'Beach <House>',
          schema_key: 'location',
          name_revision_event_id: null,
          fields: [
            {
              part_key: 'profile',
              field_key: 'weather',
              value: { type: 'text', value: 'Rain — 雨' },
              source: {
                kind: 'snapshot',
                snapshot_id: 'Opening',
                snapshot_field_id: 'Opening.weather',
                snapshot_revision_event_id: 'snapshot-metadata',
                field_revision_event_id: 'weather-write',
                at_ms: 1000,
                label: 'Opening',
              },
            },
          ],
          unresolved_timed_fields: [{ part_key: 'profile', field_key: 'future' }],
          omitted_fields: 2,
        },
      ],
      paths: [
        {
          neighbor_node_id: 'BeachHouse',
          relationship: {
            revision_event_id: 'edge-write',
            edge: {
              edge_id: 'Mara.home',
              from_node_id: 'Mara',
              to_node_id: 'BeachHouse',
              edge_kind: 'located_in',
              label: 'Home',
              directed: true,
            },
          },
        },
      ],
      omitted_neighbors: 3,
      omitted_edges: 7,
      relationships_untimed: true,
    },
  };
  const { body } = render(BibleRecall, { props: { nodeId: 'Mara' } });
  for (const text of [
    'Recall related story facts',
    'story time 1000 ms',
    'Beach &lt;House>',
    'Mara → Beach &lt;House> · located in',
    'Rain — 雨',
    'Opening.weather',
    'weather-write',
    'snapshot-metadata',
    'Mara.home',
    'edge-write',
    'Unresolved: profile.future',
    'Relationships are untimed associations',
    'Connectedness does not establish truth',
    '2 whole field records omitted',
    '3 matching neighbors',
    '7 matching relationships',
    'name revision unknown',
  ]) {
    expect(body).toContain(text);
  }
});
it('labels unspecified fictional time and an invalidated read without stale values', () => {
  bibleRecallState.anchor = 'Mara';
  bibleRecallState.invalidated = true;
  const { body } = render(BibleRecall, { props: { nodeId: 'Mara' } });
  expect(body).toContain('Unspecified');
  expect(body).toContain('Facts changed. Recall again');
  expect(body).not.toContain('Rain — 雨');
});
