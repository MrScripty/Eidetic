import type { GraphRendererStatus } from '$lib/graphRendererTypes.js';

export const graphRendererWindowState = $state<{
  status: GraphRendererStatus | null;
}>({
  status: null,
});

export function setGraphRendererWindowStatus(status: GraphRendererStatus): void {
  graphRendererWindowState.status = status;
}

export function clearGraphRendererWindowStatus(): void {
  graphRendererWindowState.status = null;
}

export function shouldDrainGraphRendererCommands(): boolean {
  const status = graphRendererWindowState.status;
  return Boolean(
    status?.renderer_window_open &&
    status.renderer_scene_ready &&
    status.renderer_window_visible_supported &&
    status.renderer_window_capability_reason === 'verified_support',
  );
}
