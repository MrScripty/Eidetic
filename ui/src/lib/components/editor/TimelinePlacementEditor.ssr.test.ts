import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import TimelinePlacementEditor from './TimelinePlacementEditor.svelte';

it('exposes exact screen-time inputs and an explicit placement command', () => {
  const { body } = render(TimelinePlacementEditor, {
    props: {
      node: {
        node_id: 'placement-ssr',
        name: 'A',
        level: 'Scene',
        sort_order: 0,
        start_ms: 1250,
        end_ms: 7000,
        notes: '',
        content_status: 'HasContent',
        locked: false,
      },
    },
  });
  expect(body).toContain('Placement: 1.25–7 seconds');
  expect(body).toContain('aria-label="Placement start seconds"');
  expect(body).toContain('aria-label="Placement end seconds"');
  expect(body).toContain('Apply placement');
  expect(body).toContain('Discard placement draft and reload');
  expect(body).toContain('Screen time.');
});
