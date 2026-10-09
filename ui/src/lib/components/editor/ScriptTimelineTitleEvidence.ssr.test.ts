import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptTimelineTitleEvidence from './ScriptTimelineTitleEvidence.svelte';
import { scriptImpactCauseLabel } from './scriptImpactNotice.js';
const previous = [{ node_id: 'A', name: '  Earlier title — 雨.  ', revision_event_id: null }];
it('shows exact historical/current title and honest baseline evidence', () => {
  const body = render(ScriptTimelineTitleEvidence, {
    props: {
      previous,
      current: [{ node_id: 'A', name: '  Station departure — 雨.  ', revision_event_id: 'rename' }],
      absent: [],
    },
  }).body;
  expect(body).toContain('  Earlier title — 雨.  ');
  expect(body).toContain('  Station departure — 雨.  ');
  expect(body).toContain('Known baseline; no recorded title revision');
  expect(body).toContain('Title revision rename');
  expect(body.replace(/\s+/g, ' ')).toContain('Accept update replaces only the selected block');
});
it('shows owned removal without fabricating current prose and keeps legacy unknown quiet', () => {
  const body = render(ScriptTimelineTitleEvidence, {
    props: { previous, current: [], absent: [['A', 'delete']] },
  }).body;
  expect(body).toContain('Title source removed · Revision delete');
  expect(body).not.toContain('Current preview title');
  expect(render(ScriptTimelineTitleEvidence, { props: {} }).body).not.toContain(
    'Originally consumed title',
  );
});
it('labels timeline title causes distinctly from placement or Notes', () => {
  expect(
    scriptImpactCauseLabel({
      dependency_id: 'generation.B.timeline_title.A',
      input: { kind: 'timeline_node', node_id: 'A' },
      consumed_revision_event_id: 'read',
      current_revision_event_id: 'rename',
      reason: 'changed',
      input_excerpt: 'A',
    }),
  ).toBe('Consumed timeline title changed.');
});
