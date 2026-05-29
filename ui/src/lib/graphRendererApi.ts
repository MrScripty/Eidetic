import { invokeDesktop } from './desktopTransport.js';
import type { BibleRenderGraphProjectionRequest } from './bibleGraphTypes.js';
import type {
  GraphRendererCameraCommand,
  GraphRendererStatus,
  GraphRendererTextEditorSettings,
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

export function updateGraphRendererProjectionRequest(
  request: BibleRenderGraphProjectionRequest,
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_update_projection_request', {
    request,
  });
}

export function applyGraphRendererCameraCommand(
  command: GraphRendererCameraCommand,
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_camera_command', {
    command,
  });
}

export function applyGraphRendererTextEditorSettings(
  settings: GraphRendererTextEditorSettings,
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_text_editor_settings', {
    settings,
  });
}

export function loadGraphRendererTextEditorSettings(): Promise<GraphRendererTextEditorSettings> {
  return invokeDesktop<GraphRendererTextEditorSettings>('graph_renderer_text_editor_settings_load');
}

export function saveGraphRendererTextEditorSettings(
  settings: GraphRendererTextEditorSettings,
): Promise<GraphRendererStatus> {
  return invokeDesktop<GraphRendererStatus>('graph_renderer_text_editor_settings_save', {
    settings,
  });
}

export function getGraphRendererVisualSnapshot(): Promise<GraphRendererVisualSnapshot> {
  return invokeDesktop<GraphRendererVisualSnapshot>('graph_renderer_visual_snapshot');
}
