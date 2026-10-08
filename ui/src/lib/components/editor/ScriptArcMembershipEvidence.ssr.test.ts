import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptArcMembershipEvidence from './ScriptArcMembershipEvidence.svelte';
it('shows exact original/current identities, names, known empty and explicit acceptance boundary', () => {
  const html = render(ScriptArcMembershipEvidence, {
    props: {
      previous: { node_id: 'clip', arc_ids: ['arc.A'], revision_event_id: 'old' },
      current: { node_id: 'clip', arc_ids: [], revision_event_id: 'clear' },
      previousFields: [
        { arc_id: 'arc.A', field: 'name', value: 'Witness — 雨.', revision_event_id: 'name.old' },
      ],
    },
  }).body;
  for (const text of [
    'Originally consumed arc assignment',
    'Current preview arc assignment',
    'Witness — 雨.',
    'arc.A',
    '(no arcs)',
    'Revision old',
    'Revision clear',
    'Accept update replaces only the selected block',
  ])
    expect(html.replace(/\s+/g, ' ')).toContain(text);
});
it('legacy absence remains unknown while clock-free known empty is labelled baseline', () => {
  const html = render(ScriptArcMembershipEvidence, {
    props: { current: { node_id: 'clip', arc_ids: [], revision_event_id: null } },
  }).body;
  expect(html).toContain('Original arc assignment consumption is unknown');
  expect(html).toContain('known baseline; no assignment event');
});
