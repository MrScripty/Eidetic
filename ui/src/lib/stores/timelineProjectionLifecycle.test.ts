import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as commands from '$lib/commandApi.js';
import { getTimelineRenderProjection } from '$lib/projectionApi.js';
import type { ProjectionEnvelope } from '$lib/projectionTypes.js';
import type { TimelineRenderProjection } from '$lib/timelineRenderTypes.js';
import type { TimelineCommandResponse } from '$lib/timelineCommandTypes.js';
import * as store from './timelineRenderProjection.svelte.js';

vi.mock('$lib/commandApi.js', () => ({
  applyTimelineChildren: vi.fn(),
  createTimelineNode: vi.fn(),
  createTimelineRelationship: vi.fn(),
  deleteTimelineNode: vi.fn(),
  deleteTimelineRelationship: vi.fn(),
  setTimelineNodeLock: vi.fn(),
  setTimelineNodeNotes: vi.fn(),
  setTimelineNodeRange: vi.fn(),
  splitTimelineNode: vi.fn(),
}));
vi.mock('$lib/projectionApi.js', () => ({ getTimelineRenderProjection: vi.fn() }));

type Projection = ProjectionEnvelope<TimelineRenderProjection>;
function projection(version: number, project: string): Projection {
  return {
    version,
    change_event_id: project,
    payload: { total_duration_ms: 120_000, tracks: [], clips: [], relationships: [] },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

const oldProject = projection(100, 'project-A');
const newProject = projection(1, 'project-B');
const range = { node_id: 'node-A', start_ms: 1_000, end_ms: 2_000 };
const read = vi.mocked(getTimelineRenderProjection);
const write = vi.mocked(commands.setTimelineNodeRange);

beforeEach(() => {
  vi.resetAllMocks();
  store.clearTimelineRenderProjection();
});

describe('timeline projection cache lifetime', () => {
  it('keeps a cleared cache empty when an old refresh succeeds', async () => {
    const old = deferred<Projection>();
    read.mockReturnValueOnce(old.promise);
    const request = store.refreshTimelineRenderProjection();
    store.clearTimelineRenderProjection();
    old.resolve(oldProject);
    await expect(request).resolves.toEqual(oldProject);
    expect(store.getCachedTimelineRenderProjection()).toBeNull();
    expect(store.timelineRenderProjectionState.pending).toBe(false);
  });

  it('does not replace a new session with a higher-version response from the old session', async () => {
    const old = deferred<Projection>();
    read.mockReturnValueOnce(old.promise).mockResolvedValueOnce(newProject);
    const request = store.refreshTimelineRenderProjection();
    store.clearTimelineRenderProjection();
    await store.refreshTimelineRenderProjection();
    old.resolve(oldProject);
    await request;
    expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
  });

  it('ignores old refresh failure and finally while a new session is loading', async () => {
    const old = deferred<Projection>();
    const current = deferred<Projection>();
    read.mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise);
    const oldRequest = store.refreshTimelineRenderProjection();
    const rejection = expect(oldRequest).rejects.toThrow('old failure');
    store.clearTimelineRenderProjection();
    const currentRequest = store.refreshTimelineRenderProjection();
    old.reject(new Error('old failure'));
    await rejection;
    expect(store.timelineRenderProjectionState.error).toBeUndefined();
    expect(store.timelineRenderProjectionState.pending).toBe(true);
    current.resolve(newProject);
    await currentRequest;
    expect(store.timelineRenderProjectionState.pending).toBe(false);
    expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
  });

  const commandCases = [
    {
      name: 'range',
      api: commands.setTimelineNodeRange,
      run: () => store.applyTimelineNodeRangeCommand(range),
    },
    {
      name: 'create',
      api: commands.createTimelineNode,
      run: () =>
        store.applyCreateTimelineNodeCommand({
          parent_id: null,
          level: 'Scene',
          name: 'Scene',
          start_ms: 0,
          end_ms: 1000,
          beat_type: null,
        }),
    },
    {
      name: 'children',
      api: commands.applyTimelineChildren,
      run: () => store.applyTimelineChildrenCommand({ parent_id: 'parent', children: [] }),
    },
    {
      name: 'relationship create',
      api: commands.createTimelineRelationship,
      run: () =>
        store.applyCreateTimelineRelationshipCommand({
          from_node_id: 'a',
          to_node_id: 'b',
          relationship_type: 'Thematic',
        }),
    },
    {
      name: 'relationship delete',
      api: commands.deleteTimelineRelationship,
      run: () => store.applyDeleteTimelineRelationshipCommand({ relationship_id: 'relationship' }),
    },
    {
      name: 'lock',
      api: commands.setTimelineNodeLock,
      run: () => store.applyTimelineNodeLockCommand({ node_id: 'node', locked: true }),
    },
    {
      name: 'notes',
      api: commands.setTimelineNodeNotes,
      run: () => store.applyTimelineNodeNotesCommand({ node_id: 'node', notes: 'notes' }),
    },
    {
      name: 'split',
      api: commands.splitTimelineNode,
      run: () => store.applySplitTimelineNodeCommand({ node_id: 'node', at_ms: 500 }),
    },
    {
      name: 'delete',
      api: commands.deleteTimelineNode,
      run: () => store.applyDeleteTimelineNodeCommand({ node_id: 'node' }),
    },
  ];

  it.each(commandCases)(
    'ignores obsolete $name success without ending current pending work',
    async ({ api, run }) => {
      const old = deferred<TimelineCommandResponse>();
      const current = deferred<Projection>();
      vi.mocked(api).mockReturnValueOnce(old.promise);
      const oldRequest = run();
      store.clearTimelineRenderProjection();
      read.mockResolvedValueOnce(newProject).mockReturnValueOnce(current.promise);
      await store.refreshTimelineRenderProjection();
      const currentRequest = store.refreshTimelineRenderProjection();
      const response: TimelineCommandResponse = { outcome: 'recorded', projection: oldProject };
      old.resolve(response);
      await expect(oldRequest).resolves.toEqual(response);
      expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
      expect(store.timelineRenderProjectionState.pending).toBe(true);
      current.resolve(newProject);
      await currentRequest;
      expect(store.timelineRenderProjectionState.pending).toBe(false);
    },
  );

  it.each(commandCases)(
    'ignores obsolete $name failure without erasing the new session error',
    async ({ api, run }) => {
      const old = deferred<TimelineCommandResponse>();
      vi.mocked(api).mockReturnValueOnce(old.promise);
      const oldRequest = run();
      const rejection = expect(oldRequest).rejects.toThrow('old command failure');
      store.clearTimelineRenderProjection();
      read
        .mockResolvedValueOnce(newProject)
        .mockRejectedValueOnce(new Error('new session failure'));
      await store.refreshTimelineRenderProjection();
      await expect(store.refreshTimelineRenderProjection()).rejects.toThrow('new session failure');
      old.reject(new Error('old command failure'));
      await rejection;
      expect(store.timelineRenderProjectionState.error).toBe('new session failure');
      expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
      expect(store.timelineRenderProjectionState.pending).toBe(false);
    },
  );
});

describe('overlapping requests within one timeline cache lifetime', () => {
  it('keeps concurrent commands pending and accepts the highest version regardless of request order', async () => {
    const earlier = deferred<TimelineCommandResponse>();
    const later = deferred<TimelineCommandResponse>();
    write.mockReturnValueOnce(earlier.promise).mockReturnValueOnce(later.promise);
    const earlierRequest = store.applyTimelineNodeRangeCommand(range);
    const laterRequest = store.applyTimelineNodeRangeCommand({ ...range, end_ms: 3_000 });
    later.resolve({ outcome: 'recorded', projection: projection(2, 'later') });
    await laterRequest;
    expect(store.timelineRenderProjectionState.pending).toBe(true);
    earlier.resolve({ outcome: 'recorded', projection: projection(3, 'earlier') });
    await earlierRequest;
    expect(store.getCachedTimelineRenderProjection()).toEqual(projection(3, 'earlier'));
    expect(store.timelineRenderProjectionState.pending).toBe(false);
  });

  it.each(['refresh', 'command'] as const)(
    'keeps pending until both requests finish when %s finishes first',
    async (first) => {
      const refresh = deferred<Projection>();
      const command = deferred<TimelineCommandResponse>();
      read.mockReturnValueOnce(refresh.promise);
      write.mockReturnValueOnce(command.promise);
      const refreshRequest = store.refreshTimelineRenderProjection();
      const commandRequest = store.applyTimelineNodeRangeCommand(range);
      if (first === 'refresh') {
        refresh.resolve(projection(8, 'refresh'));
        await refreshRequest;
      } else {
        command.resolve({ outcome: 'recorded', projection: projection(7, 'command') });
        await commandRequest;
      }
      expect(store.timelineRenderProjectionState.pending).toBe(true);
      refresh.resolve(projection(8, 'refresh'));
      command.resolve({ outcome: 'recorded', projection: projection(7, 'command') });
      await Promise.all([refreshRequest, commandRequest]);
      expect(store.timelineRenderProjectionState.pending).toBe(false);
      expect(store.getCachedTimelineRenderProjection()).toEqual(projection(8, 'refresh'));
    },
  );

  it('does not let an earlier command failure overwrite the latest refresh success', async () => {
    const command = deferred<TimelineCommandResponse>();
    write.mockReturnValueOnce(command.promise);
    const commandRequest = store.applyTimelineNodeRangeCommand(range);
    const rejection = expect(commandRequest).rejects.toThrow('late failure');
    read.mockResolvedValueOnce(newProject);
    await store.refreshTimelineRenderProjection();
    command.reject(new Error('late failure'));
    await rejection;
    expect(store.timelineRenderProjectionState.error).toBeUndefined();
    expect(store.timelineRenderProjectionState.pending).toBe(false);
    expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
  });

  it('preserves the latest request error when an earlier request succeeds later', async () => {
    const refresh = deferred<Projection>();
    read.mockReturnValueOnce(refresh.promise);
    const refreshRequest = store.refreshTimelineRenderProjection();
    write.mockRejectedValueOnce(new Error('latest command failed'));
    await expect(store.applyTimelineNodeRangeCommand(range)).rejects.toThrow(
      'latest command failed',
    );
    expect(store.timelineRenderProjectionState.pending).toBe(true);
    refresh.resolve(newProject);
    await refreshRequest;
    expect(store.timelineRenderProjectionState.error).toBe('latest command failed');
    expect(store.timelineRenderProjectionState.pending).toBe(false);
    expect(store.getCachedTimelineRenderProjection()).toEqual(newProject);
  });
});
