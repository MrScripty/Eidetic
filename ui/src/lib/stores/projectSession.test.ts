import { describe, expect, it, vi } from 'vitest';

import type { Project } from '$lib/projectTypes.js';
import { activateProjectSession, type ProjectSessionLifecycle } from './projectSession.js';

function project(name = 'Pilot'): Project {
  return {
    name,
    premise: '',
    timeline: {
      total_duration_ms: 1,
      tracks: [],
      nodes: [],
      node_arcs: [],
      relationships: [],
      structure: {
        template_name: 'Test',
        segments: [],
      },
    },
    references: [],
  };
}

function lifecycle(): ProjectSessionLifecycle & {
  calls: string[];
} {
  const calls: string[] = [];

  return {
    calls,
    clearProjectionRefreshQueue: vi.fn(() => calls.push('clear-queue')),
    resetEditorState: vi.fn(() => calls.push('reset-editor')),
    clearBibleSelection: vi.fn(() => calls.push('clear-bible-selection')),
    clearProjectionCaches: vi.fn(() => calls.push('clear-projection-caches')),
    setActiveProject: vi.fn(() => calls.push('set-project')),
    refreshProjections: vi.fn(async () => {
      calls.push('refresh-projections');
    }),
  };
}

describe('project session activation', () => {
  it('clears lifecycle state before setting and refreshing the active project', async () => {
    const deps = lifecycle();
    const nextProject = project('Episode Two');

    await activateProjectSession(nextProject, deps);

    expect(deps.clearProjectionRefreshQueue).toHaveBeenCalledWith();
    expect(deps.resetEditorState).toHaveBeenCalledWith();
    expect(deps.clearBibleSelection).toHaveBeenCalledWith();
    expect(deps.clearProjectionCaches).toHaveBeenCalledWith();
    expect(deps.setActiveProject).toHaveBeenCalledWith(nextProject);
    expect(deps.refreshProjections).toHaveBeenCalledWith();
    expect(deps.calls).toEqual([
      'clear-queue',
      'reset-editor',
      'clear-bible-selection',
      'clear-projection-caches',
      'set-project',
      'refresh-projections',
    ]);
  });
});
