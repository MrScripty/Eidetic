import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { editorState, resetEditorState } from './editor.svelte.js';
import { timelineState } from './timeline.svelte.js';
import {
  clearTimelineRenderProjection,
  refreshTimelineRenderProjection,
} from './timelineRenderProjection.svelte.js';
import { clearSelectedNodeEditorProjection } from './selectedNodeEditorProjection.svelte.js';
import {
  TIMELINE_KEYBOARD_STEP_MS,
  deleteSelectedTimelineNodeFromKeyboard,
  nudgeSelectedTimelineNode,
  resizeSelectedTimelineNodeEnd,
  resizeSelectedTimelineNodeStart,
  splitSelectedTimelineNodeAtPlayhead,
} from './timelineKeyboardCommands.js';
import type { ProjectionEnvelope } from '../projectionTypes.js';
import type { TimelineRenderProjection } from '../timelineRenderTypes.js';

const timelineProjection: ProjectionEnvelope<TimelineRenderProjection> = {
  version: 7,
  change_event_id: 'event.timeline.keyboard',
  payload: {
    total_duration_ms: 120_000,
    tracks: [
      {
        track_id: 'track.scene',
        level: 'Scene',
        label: 'Scenes',
        sort_order: 30,
        collapsed: false,
      },
    ],
    clips: [
      {
        node_id: 'node.scene.beach',
        parent_id: 'node.sequence.opening',
        track_id: 'track.scene',
        level: 'Scene',
        name: 'Beach argument',
        start_ms: 10_000,
        end_ms: 20_000,
        sort_order: 10,
        locked: false,
        content_status: 'NotesOnly',
        beat_type: null,
        arc_ids: [],
      },
    ],
    relationships: [],
    gaps: [],
    affect_overlays: [],
  },
};

const emptyTimelineProjection: ProjectionEnvelope<TimelineRenderProjection> = {
  version: 8,
  change_event_id: 'event.timeline.keyboard.empty',
  payload: {
    ...timelineProjection.payload,
    clips: [],
  },
};

let invoke: ReturnType<typeof vi.fn>;

function stubDesktopInvoke() {
  invoke = vi.fn((command: string) => {
    if (command === 'projection_timeline_render') {
      return Promise.resolve(timelineProjection);
    }
    if (command === 'projection_selected_node') {
      return Promise.resolve({
        version: 9,
        change_event_id: 'event.selected.clear',
        payload: { selected_node: null },
      });
    }
    return Promise.resolve({
      outcome: 'recorded',
      projection:
        command === 'command_timeline_delete_node' ? emptyTimelineProjection : timelineProjection,
    });
  });
  vi.stubGlobal('window', {
    __TAURI__: {
      core: { invoke },
    },
  });
  return invoke;
}

beforeEach(async () => {
  resetEditorState();
  clearTimelineRenderProjection();
  clearSelectedNodeEditorProjection();
  timelineState.playheadMs = 0;
  stubDesktopInvoke();
  await refreshTimelineRenderProjection();
  invoke.mockClear();
});

afterEach(() => {
  vi.unstubAllGlobals();
  resetEditorState();
  clearTimelineRenderProjection();
  clearSelectedNodeEditorProjection();
});

describe('timeline keyboard commands', () => {
  it('deletes the selected node through the backend-confirmed command path', async () => {
    editorState.selectedNodeId = 'node.scene.beach';
    editorState.selectedLevel = 'Scene';

    await expect(deleteSelectedTimelineNodeFromKeyboard()).resolves.toBe('applied');

    expect(invoke).toHaveBeenCalledWith('command_timeline_delete_node', {
      command: {
        id: expect.any(String),
        payload: {
          node_id: 'node.scene.beach',
        },
      },
    });
    expect(editorState.selectedNodeId).toBeNull();
    expect(editorState.selectedLevel).toBeNull();
  });

  it('splits the selected node at the playhead only when the playhead is inside the clip', async () => {
    editorState.selectedNodeId = 'node.scene.beach';
    editorState.selectedLevel = 'Scene';
    timelineState.playheadMs = 15_000;

    await expect(splitSelectedTimelineNodeAtPlayhead()).resolves.toBe('applied');

    expect(invoke).toHaveBeenCalledWith('command_timeline_split_node', {
      command: {
        id: expect.any(String),
        payload: {
          node_id: 'node.scene.beach',
          at_ms: 15_000,
        },
      },
    });

    invoke.mockClear();
    editorState.selectedNodeId = 'node.scene.beach';
    editorState.selectedLevel = 'Scene';
    timelineState.playheadMs = 20_000;

    await expect(splitSelectedTimelineNodeAtPlayhead()).resolves.toBe('unavailable');

    expect(invoke).not.toHaveBeenCalledWith('command_timeline_split_node', expect.anything());
  });

  it('moves and resizes the selected node through range commands without optimistic projection edits', async () => {
    editorState.selectedNodeId = 'node.scene.beach';
    editorState.selectedLevel = 'Scene';

    await expect(nudgeSelectedTimelineNode(TIMELINE_KEYBOARD_STEP_MS)).resolves.toBe('applied');
    await expect(resizeSelectedTimelineNodeStart(TIMELINE_KEYBOARD_STEP_MS)).resolves.toBe(
      'applied',
    );
    await expect(resizeSelectedTimelineNodeEnd(-TIMELINE_KEYBOARD_STEP_MS)).resolves.toBe(
      'applied',
    );

    expect(invoke).toHaveBeenCalledWith('command_timeline_node_range', {
      command: {
        id: expect.any(String),
        payload: {
          node_id: 'node.scene.beach',
          start_ms: 11_000,
          end_ms: 21_000,
        },
      },
    });
    expect(invoke).toHaveBeenCalledWith('command_timeline_node_range', {
      command: {
        id: expect.any(String),
        payload: {
          node_id: 'node.scene.beach',
          start_ms: 11_000,
          end_ms: 20_000,
        },
      },
    });
    expect(invoke).toHaveBeenCalledWith('command_timeline_node_range', {
      command: {
        id: expect.any(String),
        payload: {
          node_id: 'node.scene.beach',
          start_ms: 10_000,
          end_ms: 19_000,
        },
      },
    });
  });

  it('does not emit backend commands when no selected projected clip exists', async () => {
    editorState.selectedNodeId = 'node.scene.missing';
    editorState.selectedLevel = 'Scene';

    await expect(deleteSelectedTimelineNodeFromKeyboard()).resolves.toBe('unavailable');
    await expect(nudgeSelectedTimelineNode(TIMELINE_KEYBOARD_STEP_MS)).resolves.toBe('unavailable');

    expect(invoke).not.toHaveBeenCalled();
  });
});
