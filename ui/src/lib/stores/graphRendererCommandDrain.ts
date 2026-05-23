import { drainGraphRendererCommands } from '$lib/graphRendererApi.js';
import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import { applyGraphRendererCommands } from './graphRendererCommands.js';

const DEFAULT_GRAPH_RENDERER_COMMAND_DRAIN_INTERVAL_MS = 100;

export interface GraphRendererCommandDrainOptions {
  intervalMs?: number;
  drain?: () => Promise<GraphRendererCommand[]>;
  apply?: (commands: GraphRendererCommand[]) => number;
  onError?: (error: unknown) => void;
  setIntervalFn?: (callback: () => void, intervalMs: number) => unknown;
  clearIntervalFn?: (handle: unknown) => void;
}

function startInterval(callback: () => void, intervalMs: number): unknown {
  return setInterval(callback, intervalMs);
}

function stopInterval(handle: unknown): void {
  clearInterval(handle as ReturnType<typeof setInterval>);
}

function normalizeIntervalMs(intervalMs: number): number {
  if (!Number.isFinite(intervalMs) || intervalMs <= 0) {
    return DEFAULT_GRAPH_RENDERER_COMMAND_DRAIN_INTERVAL_MS;
  }
  return intervalMs;
}

export function startGraphRendererCommandDrain({
  intervalMs = DEFAULT_GRAPH_RENDERER_COMMAND_DRAIN_INTERVAL_MS,
  drain = drainGraphRendererCommands,
  apply = applyGraphRendererCommands,
  onError,
  setIntervalFn = startInterval,
  clearIntervalFn = stopInterval,
}: GraphRendererCommandDrainOptions = {}): () => void {
  let stopped = false;
  let draining = false;

  async function tick(): Promise<void> {
    if (stopped || draining) {
      return;
    }

    draining = true;
    try {
      const commands = await drain();
      if (!stopped && commands.length > 0) {
        apply(commands);
      }
    } catch (error) {
      onError?.(error);
    } finally {
      draining = false;
    }
  }

  const interval = setIntervalFn(() => {
    void tick();
  }, normalizeIntervalMs(intervalMs));
  void tick();

  return () => {
    stopped = true;
    clearIntervalFn(interval);
  };
}
