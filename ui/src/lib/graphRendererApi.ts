import { invokeDesktop } from './desktopTransport.js';
import type {
  GraphRendererCommand,
  GraphRendererStatus,
  GraphRendererVisualSnapshot,
} from './graphRendererTypes.js';

export function getGraphRendererStatus(): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_status');
}

export function drainGraphRendererCommands(): Promise<GraphRendererCommand[]> {
  return invokeDesktop<GraphRendererCommand[]>('graph_renderer_drain_commands');
}

export function getGraphRendererVisualSnapshot(): Promise<GraphRendererVisualSnapshot> {
  return invokeDesktop<GraphRendererVisualSnapshot>('graph_renderer_visual_snapshot');
}
