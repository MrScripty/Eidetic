import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import ScriptSegmentSource from './ScriptSegmentSource.svelte';
import ScriptPanel from './ScriptPanel.svelte';
import { getScriptSourceClip, showScriptSource } from './scriptSourceNavigation.js';
import { editorState, resetEditorState } from '$lib/stores/editor.svelte.js';
import { timelineState } from '$lib/stores/timeline.svelte.js';
import {
  applyTimelineNodeRangeCommand,
  clearTimelineRenderProjection,
  refreshTimelineRenderProjection,
} from '$lib/stores/timelineRenderProjection.svelte.js';
import {
  clearSelectedNodeEditorProjection,
  refreshSelectedNodeEditorProjection,
  selectedNodeEditorProjectionState,
} from '$lib/stores/selectedNodeEditorProjection.svelte.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';
import {
  getSessionScriptBlockCreationDraft,
  resetSessionScriptBlockCreationDraft,
} from '$lib/stores/scriptBlockCreationSession.svelte.js';
import {
  clearScriptDocumentProjection,
  refreshScriptDocumentProjection,
} from '$lib/stores/scriptDocumentProjection.svelte.js';
import type { ScriptBlockProjection } from '$lib/scriptTypes.js';
import type {
  SelectedNodeEditorNode,
  SelectedNodeEditorProjection,
} from '$lib/selectedNodeEditorTypes.js';
import type { TimelineRenderProjection } from '$lib/timelineRenderTypes.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';

const node: SelectedNodeEditorNode = {
  node_id: 'scene.A',
  name: 'Cafe — 雨',
  level: 'Scene',
  sort_order: 0,
  start_ms: 1000,
  end_ms: 2000,
  notes: '',
  locked: false,
  content_status: 'Empty',
};
const block: ScriptBlockProjection = {
  block: {
    id: 'block.A',
    segment_id: 'segment.A',
    block_kind: 'action',
    text: 'Canonical A',
    sort_order: 0,
  },
  revision_event_id: 'event.A',
  spans: [],
  locks: [],
};
function timeline(
  start = 1000,
  end = 2000,
  version = 1,
  present = true,
): ProjectionEnvelope<TimelineRenderProjection> {
  return {
    version,
    payload: {
      total_duration_ms: 20000,
      tracks: [],
      relationships: [],
      clips: present
        ? [{ ...node, start_ms: start, end_ms: end, track_id: 'track.scene', arc_ids: [] }]
        : [],
    },
  };
}
function selected(
  id = node.node_id,
  start = 1000,
  end = 2000,
): ProjectionEnvelope<SelectedNodeEditorProjection> {
  return {
    version: 5,
    payload: {
      node: { ...node, node_id: id, start_ms: start, end_ms: end },
      has_children: false,
      children: [],
      siblings: [],
      adjacent_parents: {},
    },
  };
}
const invoke = vi.fn();
beforeEach(() => {
  resetEditorState();
  clearTimelineRenderProjection();
  clearSelectedNodeEditorProjection();
  resetSessionScriptBlockEditDrafts();
  resetSessionScriptBlockCreationDraft();
  clearScriptDocumentProjection({ document_id: 'script.document.main' });
  timelineState.viewportWidth = 300;
  timelineState.zoom = 1;
  timelineState.scrollX = 0;
  timelineState.playheadMs = 17;
  invoke.mockReset();
  vi.stubGlobal('window', { __TAURI__: { core: { invoke } } });
});
afterEach(() => vi.unstubAllGlobals());
async function seed() {
  invoke.mockResolvedValueOnce(timeline());
  await refreshTimelineRenderProjection();
}

it('shows canonical clip name/range inside the real Script panel without changing exact screenplay text', async () => {
  await seed();
  invoke.mockResolvedValueOnce({
    version: 1,
    payload: {
      document: { id: 'script.document.main', title: 'Story', sort_order: 0 },
      segments: [
        {
          segment: {
            id: 'segment.A',
            document_id: 'script.document.main',
            source_node_id: node.node_id,
            start_ms: 0,
            end_ms: 1,
            status: 'current',
            sort_order: 0,
          },
          blocks: [block],
        },
      ],
    },
  });
  await refreshScriptDocumentProjection({ document_id: 'script.document.main' });
  const body = render(ScriptPanel).body;
  expect(body).toContain('Screenplay source clip');
  expect(body).toContain('Cafe — 雨');
  expect(body).toContain('0:01 – 0:02');
  expect(body).toContain('Go to clip');
  expect(body).toContain('Canonical A');
  expect(invoke).toHaveBeenCalledTimes(2);
});

it('distinguishes unavailable and standalone sources without offering invalid navigation', async () => {
  await seed();
  for (const sourceNodeId of [null, 'scene.missing']) {
    const body = render(ScriptSegmentSource, { props: { sourceNodeId } }).body;
    expect(body).toContain(sourceNodeId ? 'Source clip unavailable.' : 'Standalone screenplay.');
    expect(body).not.toContain('Go to clip');
  }
});

it('navigates to the latest actual retimed clip and preserves independent exact edit and uncertain creation drafts', async () => {
  await seed();
  const edit = getSessionScriptBlockEditDraft('script.document.main', block.block.id);
  edit.begin(block);
  edit.state.text = '  Edit — 雪\n\n';
  const creation = getSessionScriptBlockCreationDraft();
  creation.begin(node);
  creation.state.text = '  Append — 雨\n\n';
  invoke.mockRejectedValueOnce(new Error('uncertain save'));
  await creation.save();
  const submittedId = creation.state.commandId;
  invoke.mockResolvedValueOnce({ outcome: 'recorded', projection: timeline(9000, 11000, 2) });
  await applyTimelineNodeRangeCommand({ node_id: node.node_id, start_ms: 9000, end_ms: 11000 });
  expect(render(ScriptSegmentSource, { props: { sourceNodeId: node.node_id } }).body).toContain(
    '0:09 – 0:11',
  );
  editorState.selectedNodeId = 'scene.B';
  invoke.mockResolvedValueOnce(selected(node.node_id, 9000, 11000));
  await expect(showScriptSource(node.node_id)).resolves.toBe(true);
  expect(editorState.selectedNodeId).toBe(node.node_id);
  expect(editorState.selectedLevel).toBe('Scene');
  expect(timelineState.scrollX).toBeGreaterThan(0);
  expect(timelineState.zoom).toBe(1);
  expect(timelineState.playheadMs).toBe(17);
  expect(edit.state.text).toBe('  Edit — 雪\n\n');
  expect(edit.state.baseRevision).toBe('event.A');
  expect(creation.state.text).toBe('  Append — 雨\n\n');
  expect(creation.state.commandId).toBe(submittedId);
  expect(creation.state.uncertain).toBe(true);
  expect(creation.state.target?.start_ms).toBe(1000);
  expect(invoke.mock.calls.at(-1)?.[0]).toBe('projection_selected_node');
  expect(invoke.mock.calls.at(-1)?.[1]).toEqual({ query: { node_id: node.node_id } });
});

it('rechecks a removed source at activation and preserves selection without issuing a read or write', async () => {
  await seed();
  expect(getScriptSourceClip(node.node_id)?.name).toBe(node.name);
  invoke.mockResolvedValueOnce(timeline(1000, 2000, 2, false));
  await refreshTimelineRenderProjection();
  editorState.selectedNodeId = 'scene.B';
  editorState.selectedLevel = 'Beat';
  const count = invoke.mock.calls.length;
  await expect(showScriptSource(node.node_id)).resolves.toBe(false);
  await expect(showScriptSource(null)).resolves.toBe(false);
  expect(editorState.selectedNodeId).toBe('scene.B');
  expect(editorState.selectedLevel).toBe('Beat');
  expect(invoke).toHaveBeenCalledTimes(count);
});

it('uses existing request guards when another selection supersedes a navigation read', async () => {
  await seed();
  let resolve!: (value: ProjectionEnvelope<SelectedNodeEditorProjection>) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const pending = showScriptSource(node.node_id);
  editorState.selectedNodeId = 'scene.B';
  invoke.mockResolvedValueOnce(selected('scene.B'));
  await refreshSelectedNodeEditorProjection('scene.B');
  resolve(selected());
  await pending;
  expect(editorState.selectedNodeId).toBe('scene.B');
  expect(selectedNodeEditorProjectionState.projection?.payload.node?.node_id).toBe('scene.B');
});

it('renders an explicit failed navigation read while preserving the draft and allowing retry', async () => {
  await seed();
  const edit = getSessionScriptBlockEditDraft('script.document.main', block.block.id);
  edit.begin(block);
  edit.state.text = 'Keep my exact draft\n';
  invoke.mockRejectedValueOnce(new Error('Could not read source clip'));
  await expect(showScriptSource(node.node_id)).rejects.toThrow('Could not read source clip');
  const body = render(ScriptSegmentSource, { props: { sourceNodeId: node.node_id } }).body;
  expect(body).toContain('role="alert"');
  expect(body).toContain('Could not read source clip');
  expect(body.match(/<button([^>]*)>Go to clip<\/button>/)?.[1]).not.toContain('disabled');
  expect(edit.state.text).toBe('Keep my exact draft\n');
  invoke.mockResolvedValueOnce(selected());
  await showScriptSource(node.node_id);
  expect(selectedNodeEditorProjectionState.error).toBeUndefined();
  expect(edit.state.editing).toBe(true);
});

it('does not republish a navigation response after session projection clearing', async () => {
  await seed();
  let resolve!: (value: ProjectionEnvelope<SelectedNodeEditorProjection>) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((done) => {
        resolve = done;
      }),
  );
  const pending = showScriptSource(node.node_id);
  clearSelectedNodeEditorProjection();
  resetEditorState();
  resolve(selected());
  await pending;
  expect(editorState.selectedNodeId).toBeNull();
  expect(selectedNodeEditorProjectionState.projection).toBeNull();
});
