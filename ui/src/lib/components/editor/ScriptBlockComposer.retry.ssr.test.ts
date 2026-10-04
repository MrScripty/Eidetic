import { beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptBlockComposer from './ScriptBlockComposer.svelte';

const composer = vi.hoisted(() => ({
  state: {
    writing: true,
    documentId: 'script.document.main',
    target: { node_id: 'scene.cafe', name: 'Cafe', start_ms: 1000, end_ms: 2000 },
    commandId: 'one.request',
    text: 'Original draft',
    kind: 'action',
    saving: false,
    error: 'connection lost',
    uncertain: true,
    placementRefused: false,
  },
  save: vi.fn(),
  cancel: vi.fn(),
  begin: vi.fn(),
  useCurrentPlacement: vi.fn(),
}));
vi.mock('./scriptBlockCreationDraft.svelte.js', () => ({
  createScriptBlockCreationDraft: () => composer,
}));

beforeEach(() => {
  composer.state.uncertain = true;
  composer.state.placementRefused = false;
});

function body(): string {
  return render(ScriptBlockComposer, {
    props: {
      source: {
        node_id: 'scene.cafe',
        name: 'Cafe',
        level: 'Scene',
        sort_order: 0,
        start_ms: 6000,
        end_ms: 7000,
        notes: '',
        locked: false,
        content_status: 'Empty',
      },
    },
  }).body;
}

it('offers exact retry and disables text, kind and discard while the save result is uncertain', () => {
  const html = body();
  expect(html).toMatch(/<textarea[^>]*disabled/);
  expect(html).toMatch(/<select[^>]*disabled/);
  expect(html.match(/<button([^>]*)>Cancel<\/button>/)?.[1]).toContain('disabled');
  expect(html.match(/<button([^>]*)>Retry same save<\/button>/)?.[1]).not.toContain('disabled');
  expect(html).toContain(
    'The save may have completed. Retry to confirm it before changing this text.',
  );
  expect(html).not.toContain('Use current placement and save');
});

it('restores editing and placement recovery after a definite refusal', () => {
  composer.state.uncertain = false;
  composer.state.placementRefused = true;
  const html = body();
  expect(html).not.toMatch(/<textarea[^>]*disabled/);
  expect(html).not.toMatch(/<select[^>]*disabled/);
  expect(html.match(/<button([^>]*)>Cancel<\/button>/)?.[1]).not.toContain('disabled');
  expect(html).toContain('Use current placement and save');
});
