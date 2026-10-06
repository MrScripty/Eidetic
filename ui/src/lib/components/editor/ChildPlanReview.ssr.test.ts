import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import ChildPlanReview from './ChildPlanReview.svelte';

it('renders pending proposed material and exact escaped screenplay evidence with explicit acceptance', () => {
  const { body } = render(ChildPlanReview, {
    props: {
      plan: {
        id: 'p',
        parent_node_id: 'scene',
        target_child_level: 'Beat',
        children: [
          {
            name: '<Departure>',
            outline: 'Mara takes the midnight train.',
            weight: 1,
            beat_type: null,
          },
        ],
        script_context: [
          {
            document_id: 'main',
            segment_id: 's',
            block_id: 'b',
            source_node_id: 'scene',
            revision_event_id: 'write',
            segment_revision_event_id: 'placement',
            start_ms: 0,
            end_ms: 1000,
            text: '<Exact human text>\n\n',
          },
        ],
      },
      busy: false,
      uncertain: false,
      error: null,
      onaccept() {},
      onclose() {},
    },
  });
  expect(body).toContain('Accept timeline plan');
  expect(body).toContain('Saved screenplay text stays unchanged.');
  expect(body).toContain('&lt;Departure>');
  expect(body).toContain('&lt;Exact human text>\n\n');
  expect(body).toContain('data-source-revision="write"');
});

it('keeps uncertain acceptance retry visible and prevents closing the pending preview', () => {
  const { body } = render(ChildPlanReview, {
    props: {
      plan: {
        id: 'p',
        parent_node_id: 'scene',
        target_child_level: 'Beat',
        children: [],
        script_context: [],
      },
      busy: false,
      uncertain: true,
      error: 'Acknowledgement lost',
      onaccept() {},
      onclose() {},
    },
  });
  expect(body).toContain('Retry acceptance');
  expect(body).toMatch(/disabled(?:="")?>Close preview/);
  expect(body).toContain('No saved screenplay was selected for this plan.');
  expect(body).toContain('role="alert"');
});
