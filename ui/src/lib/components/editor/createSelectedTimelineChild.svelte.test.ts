import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { editorState, resetEditorState } from '$lib/stores/editor.svelte.js';
import { clearTimelineRenderProjection } from '$lib/stores/timelineRenderProjection.svelte.js';
import { createSelectedTimelineChild } from './createSelectedTimelineChild.js';

beforeEach(() => {
  resetEditorState();
  clearTimelineRenderProjection();
  editorState.selectedNodeId = 'parent';
});
afterEach(() => vi.unstubAllGlobals());

function provider() {
  let acknowledge!: () => void;
  const invoke = vi.fn().mockImplementation(
    (_name, { command }) =>
      new Promise((resolve) => {
        acknowledge = () =>
          resolve({
            outcome: 'recorded',
            projection: {
              version: 1,
              payload: {
                total_duration_ms: 600000,
                tracks: [],
                relationships: [],
                clips: [
                  {
                    node_id: command.payload.node_id,
                    parent_id: 'parent',
                    level: 'Scene',
                    name: 'New Scene',
                  },
                ],
              },
            },
          });
      }),
  );
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
  return { invoke, acknowledge: () => acknowledge() };
}

it('uses canonical parent-derived creation and selects the acknowledged child', async () => {
  const api = provider();
  const pending = createSelectedTimelineChild('parent', () => true);
  expect(api.invoke.mock.calls[0]![0]).toBe('command_timeline_create_child_from_parent');
  expect(api.invoke.mock.calls[0]![1].command.payload.parent_id).toBe('parent');
  expect(editorState.selectedNodeId).toBe('parent');
  api.acknowledge();
  const child = await pending;
  expect(child).toBe(api.invoke.mock.calls[0]![1].command.payload.node_id);
  expect(editorState.selectedNodeId).toBe(child);
  expect(editorState.selectedLevel).toBe('Scene');
});

it.each(['selection', 'session', 'unmount'])(
  'retains user selection after delayed %s change',
  async (change) => {
    const api = provider();
    let mounted = true;
    const pending = createSelectedTimelineChild('parent', () => mounted);
    if (change === 'selection') editorState.selectedNodeId = 'different-scene';
    if (change === 'session') {
      resetEditorState();
      editorState.selectedNodeId = 'parent';
    }
    if (change === 'unmount') mounted = false;
    const chosen = editorState.selectedNodeId;
    api.acknowledge();
    expect(await pending).toBeNull();
    expect(editorState.selectedNodeId).toBe(chosen);
  },
);
