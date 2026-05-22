import { beforeEach, describe, expect, it } from 'vitest';

import { setWorkspaceMode, workspaceModeState } from './workspaceMode.svelte.js';

beforeEach(() => {
  workspaceModeState.mode = 'script';
});

describe('workspace mode store', () => {
  it('stores transient workspace layout mode', () => {
    setWorkspaceMode('graph');

    expect(workspaceModeState.mode).toBe('graph');

    setWorkspaceMode('split');

    expect(workspaceModeState.mode).toBe('split');
  });
});
