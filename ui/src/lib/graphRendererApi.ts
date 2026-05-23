import { invokeDesktop } from './desktopTransport.js';
import type { BibleRenderGraphProjectionRequest } from './bibleGraphTypes.js';
import type {
  GraphRendererCommand,
  GraphRendererStatus,
  GraphRendererVisualSnapshot,
  OpenGraphRendererRequest,
} from './graphRendererTypes.js';

export function openGraphRenderer(
  request: OpenGraphRendererRequest = {},
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_open', { request });
}

export function focusGraphRenderer(): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_focus');
}

export function closeGraphRenderer(): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_close');
}

export function getGraphRendererStatus(): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_status');
}

export function setGraphRendererProjection(
  request: BibleRenderGraphProjectionRequest,
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_set_projection', { request });
}

export function drainGraphRendererCommands(): Promise<GraphRendererCommand[]> {
  return invokeDesktop<GraphRendererCommand[]>('graph_renderer_drain_commands');
}

export function getGraphRendererVisualSnapshot(): Promise<GraphRendererVisualSnapshot> {
  return invokeDesktop<GraphRendererVisualSnapshot>('graph_renderer_visual_snapshot');
}
