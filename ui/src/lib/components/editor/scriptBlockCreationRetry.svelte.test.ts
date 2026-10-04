import { afterEach, expect, it, vi } from 'vitest';
import type { CommandEnvelope } from '$lib/projectionTypes.js';
import type { CreateScriptBlockCommand } from '$lib/scriptTypes.js';
import type { SelectedNodeEditorNode } from '$lib/selectedNodeEditorTypes.js';
import {
  applyScriptBlockCreationCommand,
  clearScriptDocumentProjection,
} from '$lib/stores/scriptDocumentProjection.svelte.js';
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

afterEach(() => {
  clearScriptDocumentProjection({ document_id: 'script.document.main' });
  vi.unstubAllGlobals();
});

it('reconciles a committed save whose acknowledgement was lost through the real frontend command path', async () => {
  const recorded = new Map<string, string>();
  let loseAcknowledgement = true;
  const invoke = vi.fn(
    async (_name: string, args: { command: CommandEnvelope<CreateScriptBlockCommand> }) => {
      const { command } = args;
      const payload = JSON.stringify(command.payload);
      const previous = recorded.get(command.id);
      if (previous && previous !== payload)
        throw {
          kind: 'bad_request',
          message: 'command id already exists with a different payload',
        };
      if (!previous) recorded.set(command.id, payload);
      if (loseAcknowledgement) {
        loseAcknowledgement = false;
        throw new Error('acknowledgement lost after commit');
      }
      return {
        outcome: previous ? 'already_recorded' : 'recorded',
        projection: {
          version: 5,
          change_event_id: 'event.original',
          payload: {
            document: { id: 'script.document.main', title: 'Cafe', sort_order: 0 },
            segments: [],
          },
        },
      };
    },
  );
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  const composer = createScriptBlockCreationDraft({
    documentId: () => 'script.document.main',
    save: applyScriptBlockCreationCommand,
    newCommandId: () => 'request.original',
  });
  composer.begin(source);
  composer.state.text = '  Original 雨\n\n';
  await composer.save();
  const submitted = structuredClone(invoke.mock.calls[0]?.[1]);
  expect(recorded.size).toBe(1);
  composer.state.text = 'Changed after acknowledgement loss';
  composer.state.kind = 'dialogue';
  await composer.save();
  expect(invoke.mock.calls[1]?.[1]).toEqual(submitted);
  expect(recorded.size).toBe(1);
  expect(composer.state.writing).toBe(false);
});

it('does not discard, restart or refresh placement while the submitted outcome is uncertain', async () => {
  const save = vi
    .fn()
    .mockRejectedValueOnce(new Error('connection lost'))
    .mockResolvedValueOnce({});
  const newCommandId = vi
    .fn()
    .mockReturnValueOnce('original.request')
    .mockReturnValueOnce('new.request');
  const composer = createScriptBlockCreationDraft({
    documentId: () => 'script.document.main',
    save,
    newCommandId,
  });
  composer.begin(source);
  composer.state.text = 'Exact original draft';
  await composer.save();
  composer.cancel();
  composer.begin({ ...source, node_id: 'scene.other', name: 'Other' });
  await composer.useCurrentPlacement({ ...source, start_ms: 6000, end_ms: 7000 });
  expect(composer.state.writing).toBe(true);
  expect(composer.state.commandId).toBe('original.request');
  expect(composer.state.target?.node_id).toBe(source.node_id);
  expect(composer.state.target?.start_ms).toBe(1000);
  expect(newCommandId).toHaveBeenCalledTimes(1);
  expect(save).toHaveBeenCalledTimes(1);
  await composer.save();
  expect(save.mock.calls[1]).toEqual(save.mock.calls[0]);
  expect(composer.state.writing).toBe(false);
});

it('unlocks placement recovery only after an exact retry receives the definite native refusal', async () => {
  const message = 'timeline placement changed; use its current placement and try again';
  const invoke = vi
    .fn()
    .mockRejectedValueOnce({ kind: 'internal', message })
    .mockRejectedValueOnce({ kind: 'bad_request', message })
    .mockResolvedValueOnce({
      outcome: 'recorded',
      projection: {
        version: 5,
        payload: {
          document: { id: 'script.document.main', title: 'Cafe', sort_order: 0 },
          segments: [],
        },
      },
    });
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  const composer = createScriptBlockCreationDraft({
    documentId: () => 'script.document.main',
    save: applyScriptBlockCreationCommand,
    newCommandId: () => 'one.request',
  });
  composer.begin(source);
  composer.state.text = 'Original draft';
  await composer.save();
  expect(composer.state.uncertain).toBe(true);
  expect(composer.state.placementRefused).toBe(false);
  await composer.useCurrentPlacement({ ...source, start_ms: 6000, end_ms: 7000 });
  expect(invoke).toHaveBeenCalledTimes(1);
  await composer.save();
  expect(invoke.mock.calls[1]).toEqual(invoke.mock.calls[0]);
  expect(composer.state.uncertain).toBe(false);
  expect(composer.state.placementRefused).toBe(true);
  composer.state.text = 'Corrected draft after definite refusal';
  composer.state.kind = 'scene_heading';
  await composer.useCurrentPlacement({ ...source, start_ms: 6000, end_ms: 7000 });
  expect(invoke.mock.calls[2]?.[1].command).toEqual({
    id: 'one.request',
    payload: {
      document_id: 'script.document.main',
      source_node_id: source.node_id,
      expected_start_ms: 6000,
      expected_end_ms: 7000,
      block_kind: 'scene_heading',
      text: 'Corrected draft after definite refusal',
    },
  });
  expect(composer.state.writing).toBe(false);
});

it('retains the original submission through repeated ambiguous failures even if mutable draft text is cleared', async () => {
  const save = vi
    .fn()
    .mockRejectedValueOnce(new Error('connection lost'))
    .mockRejectedValueOnce(
      new Error('timeline placement changed; use its current placement and try again'),
    )
    .mockResolvedValueOnce({});
  const composer = createScriptBlockCreationDraft({
    documentId: () => 'script.document.main',
    save,
    newCommandId: () => 'one.request',
  });
  composer.begin(source);
  composer.state.text = '  Original 雨\n\n';
  await composer.save();
  composer.state.text = '';
  composer.state.kind = 'dialogue';
  await composer.save();
  expect(composer.state.uncertain).toBe(true);
  expect(composer.state.placementRefused).toBe(false);
  await composer.save();
  expect(save.mock.calls[1]).toEqual(save.mock.calls[0]);
  expect(save.mock.calls[2]).toEqual(save.mock.calls[0]);
  expect(composer.state.writing).toBe(false);
});
