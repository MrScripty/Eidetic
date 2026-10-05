import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptPanel from './ScriptPanel.svelte';
import { editorState, resetEditorState } from '$lib/stores/editor.svelte.js';
import {
  getSessionScriptBlockCreationDraft,
  resetSessionScriptBlockCreationDraft,
} from '$lib/stores/scriptBlockCreationSession.svelte.js';
import {
  clearSelectedNodeEditorProjection,
  refreshSelectedNodeEditorProjection,
  selectedNodeEditorProjectionState,
} from '$lib/stores/selectedNodeEditorProjection.svelte.js';
import {
  applyTimelineNodeRangeCommand,
  clearTimelineRenderProjection,
  refreshTimelineRenderProjection,
} from '$lib/stores/timelineRenderProjection.svelte.js';
import { clearScriptDocumentProjection } from '$lib/stores/scriptDocumentProjection.svelte.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type {
  SelectedNodeEditorNode,
  SelectedNodeEditorProjection,
} from '$lib/selectedNodeEditorTypes.js';
import type { TimelineRenderProjection } from '$lib/timelineRenderTypes.js';

const node: SelectedNodeEditorNode = {
  node_id: 'scene.A',
  name: 'Cafe',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  locked: false,
  content_status: 'Empty',
};
function selected(
  start = 1000,
  end = 2000,
  version = 1,
  id = node.node_id,
): ProjectionEnvelope<SelectedNodeEditorProjection> {
  return {
    version,
    payload: {
      node: { ...node, node_id: id, start_ms: start, end_ms: end },
      has_children: false,
      children: [],
      siblings: [],
      adjacent_parents: {},
    },
  };
}
function timeline(
  start = 1000,
  end = 2000,
  version = 1,
): ProjectionEnvelope<TimelineRenderProjection> {
  return {
    version,
    payload: {
      total_duration_ms: 10000,
      tracks: [],
      relationships: [],
      clips: [{ ...node, track_id: 'track.scene', arc_ids: [], start_ms: start, end_ms: end }],
    },
  };
}
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
const invoke = vi.fn();
beforeEach(() => {
  resetEditorState();
  clearSelectedNodeEditorProjection();
  clearTimelineRenderProjection();
  resetSessionScriptBlockCreationDraft();
  clearScriptDocumentProjection({ document_id: 'script.document.main' });
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());
async function seed() {
  editorState.selectedNodeId = node.node_id;
  invoke.mockResolvedValueOnce(selected());
  await refreshSelectedNodeEditorProjection(node.node_id);
  invoke.mockResolvedValueOnce(timeline());
  await refreshTimelineRenderProjection();
}
async function retime(start: number, end: number, version: number) {
  invoke.mockResolvedValueOnce({ outcome: 'recorded', projection: timeline(start, end, version) });
  await applyTimelineNodeRangeCommand({ node_id: node.node_id, start_ms: start, end_ms: end });
}

it('requires fresh same-node placement after the actual range command before recovering the exact refused draft', async () => {
  await seed();
  const composer = getSessionScriptBlockCreationDraft();
  composer.begin(node);
  composer.state.text = '  Exact draft — 雨\n\n';
  await retime(1500, 2700, 2);
  invoke.mockRejectedValueOnce({
    kind: 'bad_request',
    message: 'timeline placement changed; use its current placement and try again',
  });
  await composer.save();
  expect(editorState.selectedNodeId).toBe(node.node_id);
  expect(selectedNodeEditorProjectionState.projection?.payload.node?.start_ms).toBe(1000);
  // Original ScriptPanel offers recovery with stale times here. SSR executes
  // the real source derivation; its client effect is executed explicitly below.
  expect(render(ScriptPanel).body).not.toContain('Use current placement and save');
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  expect(getScriptCreationSource()).toBeNull();
  invoke.mockResolvedValueOnce(selected(1500, 2700, 2));
  await refreshRetimedScriptSource();
  expect(render(ScriptPanel).body).toContain('Use current placement and save');
  const current = getScriptCreationSource();
  expect(current).not.toBeNull();
  invoke.mockResolvedValueOnce({
    outcome: 'recorded',
    projection: {
      version: 3,
      payload: {
        document: { id: 'script.document.main', title: 'Story', sort_order: 0 },
        segments: [],
      },
    },
  });
  await composer.useCurrentPlacement(current!);
  expect(invoke.mock.calls.at(-1)?.[1].command.payload).toMatchObject({
    source_node_id: node.node_id,
    expected_start_ms: 1500,
    expected_end_ms: 2700,
    text: '  Exact draft — 雨\n\n',
  });
  expect(composer.state.writing).toBe(false);
});

it('supersedes a pending pre-move read and ignores its late stale completion', async () => {
  await seed();
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  const old = deferred<ProjectionEnvelope<SelectedNodeEditorProjection>>();
  invoke.mockReturnValueOnce(old.promise);
  const preMove = refreshSelectedNodeEditorProjection(node.node_id);
  await retime(1600, 2600, 2);
  expect(selectedNodeEditorProjectionState.pending).toBe(true);
  invoke.mockResolvedValueOnce(selected(1600, 2600, 2));
  await refreshRetimedScriptSource();
  old.resolve(selected());
  await preMove;
  expect(getScriptCreationSource()?.start_ms).toBe(1600);
  expect(getScriptCreationSource()?.end_ms).toBe(2600);
  expect(selectedNodeEditorProjectionState.pending).toBe(false);
});

it('uses the latest placement when a second resize races the first retime refresh', async () => {
  await seed();
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  await retime(1500, 2500, 2);
  const first = deferred<ProjectionEnvelope<SelectedNodeEditorProjection>>();
  invoke.mockReturnValueOnce(first.promise);
  const pending = refreshRetimedScriptSource();
  await retime(1700, 2900, 3);
  invoke.mockResolvedValueOnce(selected(1700, 2900, 3));
  await refreshRetimedScriptSource();
  first.resolve(selected(1500, 2500, 2));
  await pending;
  expect(getScriptCreationSource()?.start_ms).toBe(1700);
  expect(getScriptCreationSource()?.end_ms).toBe(2900);
});

it('keeps recovery unavailable after a failed or older-version refresh without changing the captured draft', async () => {
  await seed();
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  const composer = getSessionScriptBlockCreationDraft();
  composer.begin(node);
  composer.state.text = 'Original exact draft';
  await retime(1500, 2600, 2);
  invoke.mockRejectedValueOnce(new Error('read failed'));
  await expect(refreshRetimedScriptSource()).rejects.toThrow('read failed');
  expect(getScriptCreationSource()).toBeNull();
  expect(render(ScriptPanel).body).toContain('Refresh selected clip');
  expect(render(ScriptPanel).body).toContain('read failed');
  expect(composer.state.target?.start_ms).toBe(1000);
  expect(composer.state.text).toBe('Original exact draft');
  invoke.mockResolvedValueOnce(selected(1500, 2600, 0));
  await refreshRetimedScriptSource();
  expect(getScriptCreationSource()).toBeNull();
  expect(selectedNodeEditorProjectionState.projection?.version).toBe(1);
});

it('preserves clear/selection request guards and does not fetch for an unchanged or absent selected clip', async () => {
  await seed();
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  await refreshRetimedScriptSource();
  expect(invoke).toHaveBeenCalledTimes(2);
  await retime(1500, 2600, 2);
  const response = deferred<ProjectionEnvelope<SelectedNodeEditorProjection>>();
  invoke.mockReturnValueOnce(response.promise);
  const pending = refreshRetimedScriptSource();
  clearSelectedNodeEditorProjection();
  editorState.selectedNodeId = 'scene.B';
  response.resolve(selected(1500, 2600, 2));
  await pending;
  expect(selectedNodeEditorProjectionState.projection).toBeNull();
  expect(getScriptCreationSource()).toBeNull();
  const calls = invoke.mock.calls.length;
  await refreshRetimedScriptSource();
  expect(invoke).toHaveBeenCalledTimes(calls);
});

it('supersedes a delayed B read when A-to-B-to-A retiming returns to the matching cached placement', async () => {
  await seed();
  const { getScriptCreationSource, refreshRetimedScriptSource } =
    await import('./scriptCreationSource.js');
  const composer = getSessionScriptBlockCreationDraft();
  composer.begin(node);
  composer.state.text = '  Retained A draft — 雨\n\n';
  await retime(1500, 2500, 2);
  const delayedB = deferred<ProjectionEnvelope<SelectedNodeEditorProjection>>();
  invoke.mockReturnValueOnce(delayedB.promise);
  const readB = refreshRetimedScriptSource();
  expect(selectedNodeEditorProjectionState.pending).toBe(true);
  await retime(1000, 2000, 3);
  expect(getScriptCreationSource()?.start_ms).toBe(1000);
  // The current clip agrees with cached A, but an owned B request is still in flight.
  invoke.mockResolvedValueOnce(selected(1000, 2000, 3));
  await refreshRetimedScriptSource();
  delayedB.resolve(selected(1500, 2500, 2));
  await readB;
  expect(getScriptCreationSource()?.start_ms).toBe(1000);
  expect(getScriptCreationSource()?.end_ms).toBe(2000);
  expect(selectedNodeEditorProjectionState.projection?.version).toBe(3);
  expect(selectedNodeEditorProjectionState.pending).toBe(false);
  expect(composer.state.target?.start_ms).toBe(1000);
  expect(composer.state.text).toBe('  Retained A draft — 雨\n\n');
  const calls = invoke.mock.calls.length;
  await refreshRetimedScriptSource();
  expect(invoke).toHaveBeenCalledTimes(calls);
});
