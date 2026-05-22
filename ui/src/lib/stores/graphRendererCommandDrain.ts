import { drainGraphRendererCommands } from '$lib/graphRendererApi.js';
import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import { applyGraphRendererCommands } from './graphRendererCommands.js';

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

export function startGraphRendererCommandDrain({
  intervalMs = 100,
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
  }, intervalMs);
  void tick();

  return () => {
    stopped = true;
    clearIntervalFn(interval);
  };
}
