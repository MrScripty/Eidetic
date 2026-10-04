import { expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptBlockComposer from './ScriptBlockComposer.svelte';

it('makes manual writing discoverable in an empty screenplay and requires timeline selection', () => {
  const { body } = render(ScriptBlockComposer, {
    props: { documentId: 'script.document.main', source: null },
  });
  expect(body).toContain('Write screenplay');
  expect(body).toContain('disabled');
  expect(body).toContain('Select a timeline clip to start writing.');
});

it('offers authoring for the selected context with safely rendered names', () => {
  const { body } = render(ScriptBlockComposer, {
    props: {
      documentId: 'script.document.main',
      source: {
        node_id: 'scene.cafe',
        name: '<Cafe> — 雨',
        level: 'Scene',
        sort_order: 0,
        start_ms: 1000,
        end_ms: 2000,
        notes: '',
        locked: true,
        content_status: 'Empty',
      },
    },
  });
  expect(body).toContain('For &lt;Cafe> — 雨');
  expect(body).toContain('without replacing existing blocks');
  expect(body).not.toContain('disabled');
});
