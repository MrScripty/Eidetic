import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import type { ChildPlanBibleContext } from '$lib/childPlanningTypes.js';
import type { FieldValue } from '$lib/projectionTypes.js';
import ChildPlanBibleEvidence from './ChildPlanBibleEvidence.svelte';
import ChildPlanReview from './ChildPlanReview.svelte';

function evidence(): ChildPlanBibleContext {
  return {
    context: {
      version: 7,
      change_event_id: 'captured-context',
      payload: {
        target_node_id: 'scene',
        nodes: [
          {
            node_id: 'Mara',
            name: '<Original Mara>',
            schema_key: 'character',
            fields: [
              {
                part_key: 'profile',
                part_name: 'Profile',
                field_key: 'tagline',
                value: { type: 'text', value: 'Blue umbrella\nExact second line.\n\n' },
              },
            ],
            snapshots: [],
            unresolved_timed_fields: [{ part_key: 'profile', field_key: 'destination' }],
            incoming_edges: [],
            outgoing_edges: [],
          },
        ],
      },
    },
    inputs: [
      {
        node_id: 'Mara',
        part_key: 'profile',
        field_key: 'tagline',
        field_id: 'fact-blue',
        revision_event_id: 'original-field-revision',
        value: { type: 'text', value: 'Blue umbrella\nExact second line.\n\n' },
      },
    ],
  };
}

it('shows original fact values, names and exact field revision with safely escaped text', () => {
  const original = evidence();
  const { body } = render(ChildPlanBibleEvidence, { props: { evidence: original } });
  expect(body).toContain('Bible facts used for this plan');
  expect(body).toContain('aria-label="Recorded Bible evidence"');
  expect(body).toContain('&lt;Original Mara>');
  expect(body).toContain('profile.tagline');
  expect(body).toContain('Blue umbrella\nExact second line.\n\n');
  expect(body).toContain('Field revision: original-field-revision');
  expect(body).toContain('data-bible-field="fact-blue"');
  expect(body).not.toContain('Field revision: captured-context');
  expect(body).toContain('Later Bible edits do not change this evidence.');
});

it('distinguishes missing legacy evidence from an explicitly recorded empty context', () => {
  const missing = render(ChildPlanBibleEvidence, { props: {} }).body;
  const empty = evidence();
  empty.context.payload.nodes = [];
  empty.inputs = [];
  const recorded = render(ChildPlanBibleEvidence, { props: { evidence: empty } }).body;
  expect(missing).toContain('Bible evidence was not recorded for this plan.');
  expect(missing).not.toContain('No Bible nodes were supplied');
  expect(recorded).toContain('No Bible nodes were supplied to this plan.');
  expect(recorded).not.toContain('Bible evidence was not recorded');
});

it('shows unresolved timed identities without presenting baseline values as resolved facts', () => {
  const receipt = evidence();
  const { body } = render(ChildPlanBibleEvidence, { props: { evidence: receipt } });
  expect(body).toContain('Fictional story time was unspecified. Timed fields were withheld.');
  expect(body).toContain('Unresolved timed field: profile.destination. No value was supplied.');
  expect(body).not.toContain('Fictional story time: 0ms');
});

it('shows effective timed facts at time zero without borrowing baseline field revision', () => {
  const receipt = evidence();
  receipt.context.payload.story_time_ms = 0;
  const node = receipt.context.payload.nodes[0]!;
  node.fields = [];
  node.snapshots = [
    {
      label: '<Departure>',
      at_ms: 0,
      fields: [
        {
          part_key: 'profile',
          part_name: 'Profile',
          field_key: 'tagline',
          value: { type: 'text', value: 'Timed green umbrella' },
        },
      ],
    },
  ];
  const { body } = render(ChildPlanBibleEvidence, { props: { evidence: receipt } });
  expect(body).toContain('Fictional story time: 0ms.');
  expect(body).toContain('Effective fact from &lt;Departure> at fictional time 0ms');
  expect(body).toContain('Timed green umbrella');
  expect(body).not.toContain('Blue umbrella');
  expect(body).not.toContain('Field revision: original-field-revision');
  expect(body).not.toContain('data-bible-field="fact-blue"');
});

it('preserves typed zero/false/reference values and scopes revision lookup to the node', () => {
  const receipt = evidence();
  const values: FieldValue[] = [
    { type: 'integer', value: 0 },
    { type: 'number', value: 0.25 },
    { type: 'bool', value: false },
    { type: 'object_ref', value: { kind: 'timeline_node', id: 'scene.source' } },
    { type: 'asset_ref', value: 'asset.original' },
  ];
  receipt.context.payload.nodes[0]!.fields = values.map((value, index) => ({
    part_key: 'profile',
    part_name: 'Profile',
    field_key: `value${index}`,
    value,
  }));
  receipt.inputs = [{ ...receipt.inputs[0]!, node_id: 'Another Mara', field_key: 'value0' }];
  const { body } = render(ChildPlanBibleEvidence, { props: { evidence: receipt } });
  for (const value of ['0', '0.25', 'false', 'timeline_node: scene.source', 'asset.original']) {
    expect([...body.matchAll(/<pre[^>]*>(.*?)<\/pre>/gs)].map((match) => match[1])).toContain(
      value,
    );
  }
  expect(body).toContain('Field revision was not recorded.');
  expect(body).not.toContain('Field revision: original-field-revision');
});

it('shows consumed graph connections with original endpoint names, kinds and untimed status', () => {
  const receipt = evidence();
  const node = receipt.context.payload.nodes[0]!;
  node.outgoing_edges = [
    {
      edge_id: 'connection',
      from_node_id: 'Mara',
      to_node_id: 'Station',
      label: '<Departure route>',
      edge_kind: 'located_in',
      directed: true,
    },
  ];
  node.incoming_edges = [
    {
      edge_id: 'outside',
      from_node_id: 'outside-node',
      to_node_id: 'Mara',
      label: 'Affinity',
      edge_kind: { custom: 'knows' },
      directed: false,
    },
  ];
  receipt.context.payload.nodes.push({
    node_id: 'Station',
    name: 'Original station',
    schema_key: 'location',
    parent_id: 'Mara',
    fields: [],
    snapshots: [],
    incoming_edges: [],
    outgoing_edges: [],
  });
  const { body } = render(ChildPlanBibleEvidence, { props: { evidence: receipt } });
  expect(body).toContain('&lt;Departure route>');
  const visible = body.replace(/\s+/g, ' ');
  expect(visible).toContain('&lt;Original Mara> → Original station (located in)');
  expect(visible).toContain('outside-node ↔ &lt;Original Mara> (knows)');
  expect(body).toContain('Parent: &lt;Original Mara>');
  expect(body).toContain('data-bible-edge="connection"');
  expect(visible).toContain(
    'Graph relationships are untimed; their validity at the fictional story time was not established.',
  );
});

it('integrates original Bible evidence with the existing explicit child acceptance surface', () => {
  const { body } = render(ChildPlanReview, {
    props: {
      plan: {
        id: 'p',
        parent_node_id: 'scene',
        target_child_level: 'Beat',
        children: [],
        script_context: [],
        bible_context: evidence(),
      },
      busy: false,
      uncertain: false,
      error: null,
      onaccept() {},
      onclose() {},
    },
  });
  expect(body).toContain('Saved screenplay used for this plan');
  expect(body).toContain('Bible facts used for this plan');
  expect(body).toContain('Blue umbrella');
  expect(body).toContain('Accept timeline plan');
  expect(body).toContain('Saved screenplay text stays unchanged.');
});
