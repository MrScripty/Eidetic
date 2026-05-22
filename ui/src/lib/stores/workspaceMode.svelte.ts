export type WorkspaceMode = 'script' | 'graph' | 'split';

export const workspaceModeState = $state<{
  mode: WorkspaceMode;
}>({
  mode: 'script',
});

export function setWorkspaceMode(mode: WorkspaceMode): void {
  workspaceModeState.mode = mode;
}
