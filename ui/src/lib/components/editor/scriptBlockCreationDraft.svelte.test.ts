import { expect, it, vi } from 'vitest';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import { createScriptBlockCreationDraft } from './scriptBlockCreationDraft.svelte.js';

const source: SelectedNodeEditorNode = {
  node_id: 'scene.cafe',
  name: 'Cafe',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  locked: false,
  content_status: 'Empty',
};

it('keeps exact refused text and its original context until placement is explicitly refreshed', async () => {
  const save = vi
    .fn()
    .mockRejectedValueOnce(new Error('timeline placement changed'))
    .mockResolvedValueOnce({});
  const composer = createScriptBlockCreationDraft({
    documentId: () => 'script.document.main',
    save,
    newCommandId: () => 'stable-request',
  });
  composer.begin(source);
  composer.state.text = '  Mara sees 雨.\n\n';
  await composer.save();
  expect(composer.state.text).toBe('  Mara sees 雨.\n\n');
  expect(composer.state.writing).toBe(true);
  expect(composer.state.error).toBe('timeline placement changed');
  expect(save).toHaveBeenCalledWith(
    {
      document_id: 'script.document.main',
      source_node_id: 'scene.cafe',
      expected_start_ms: 1000,
      expected_end_ms: 2000,
      block_kind: 'action',
      text: '  Mara sees 雨.\n\n',
    },
    'stable-request',
  );
  await composer.useCurrentPlacement({
    ...source,
    node_id: 'scene.station',
    start_ms: 3000,
    end_ms: 4000,
  });
  expect(save).toHaveBeenCalledTimes(1);
  await composer.useCurrentPlacement({ ...source, start_ms: 6000, end_ms: 7000 });
  expect(save).toHaveBeenLastCalledWith(
    {
      document_id: 'script.document.main',
      source_node_id: 'scene.cafe',
      expected_start_ms: 6000,
      expected_end_ms: 7000,
      block_kind: 'action',
      text: '  Mara sees 雨.\n\n',
    },
    'stable-request',
  );
  expect(composer.state.writing).toBe(false);
});

it('retries an ambiguous acknowledgement with the same command and captured document identity', async () => {
  const save = vi
    .fn()
    .mockRejectedValueOnce(new Error('connection lost'))
    .mockResolvedValueOnce({});
  let documentId = 'script.document.main';
  const composer = createScriptBlockCreationDraft({
    documentId: () => documentId,
    save,
    newCommandId: () => 'one-request',
  });
  composer.begin(source);
  composer.state.text = 'One authored block';
  documentId = 'other.document';
  await composer.save();
  await composer.save();
  expect(save.mock.calls[0]).toEqual(save.mock.calls[1]);
  expect(save.mock.calls[0]?.[0].document_id).toBe('script.document.main');
  expect(save.mock.calls[0]?.[1]).toBe('one-request');
  expect(composer.state.error).toBeNull();
});
