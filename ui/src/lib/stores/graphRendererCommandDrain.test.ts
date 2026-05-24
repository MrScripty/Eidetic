import { describe, expect, it, vi } from 'vitest';

import type { GraphRendererCommand } from '$lib/graphRendererTypes.js';
import { startGraphRendererCommandDrain } from './graphRendererCommandDrain.js';

describe('graph renderer command drain lifecycle', () => {
  it('drains immediately and applies available renderer commands', async () => {
    const commands: GraphRendererCommand[] = [{ type: 'select_node', node_id: 'node.ada' }];
    const drain = vi.fn().mockResolvedValue(commands);
    const apply = vi.fn();
    const intervalHandle = {};
    const setIntervalFn = vi.fn().mockReturnValue(intervalHandle);
    const clearIntervalFn = vi.fn();

    const stop = startGraphRendererCommandDrain({
      drain,
      apply,
      setIntervalFn,
      clearIntervalFn,
    });
    await Promise.resolve();

    expect(drain).toHaveBeenCalledTimes(1);
    expect(apply).toHaveBeenCalledWith(commands);

    stop();
    expect(clearIntervalFn).toHaveBeenCalledWith(intervalHandle);
  });

  it('does not overlap slow drain requests', async () => {
    let resolveDrain: (commands: GraphRendererCommand[]) => void = () => {};
    const drain = vi.fn(
      () =>
        new Promise<GraphRendererCommand[]>((resolve) => {
          resolveDrain = resolve;
        }),
    );
    const apply = vi.fn();
    let intervalCallback = () => {};

    const stop = startGraphRendererCommandDrain({
      drain,
      apply,
      setIntervalFn: (callback) => {
        intervalCallback = callback;
        return 1;
      },
      clearIntervalFn: vi.fn(),
    });

    intervalCallback?.();
    expect(drain).toHaveBeenCalledTimes(1);

    resolveDrain([{ type: 'select_edge', edge_id: 'edge.ada.beach' }]);
    await Promise.resolve();

    expect(apply).toHaveBeenCalledTimes(1);
    stop();
  });

  it('does not drain while the renderer command bridge is disabled', async () => {
    const drain = vi.fn().mockResolvedValue([]);
    let intervalCallback = () => {};

    const stop = startGraphRendererCommandDrain({
      drain,
      shouldDrain: () => false,
      setIntervalFn: (callback) => {
        intervalCallback = callback;
        return 1;
      },
      clearIntervalFn: vi.fn(),
    });
    await Promise.resolve();
    intervalCallback();
    await Promise.resolve();

    expect(drain).not.toHaveBeenCalled();
    stop();
  });

  it('drops commands that resolve after the drain is stopped', async () => {
    let resolveDrain: (commands: GraphRendererCommand[]) => void = () => {};
    const drain = vi.fn(
      () =>
        new Promise<GraphRendererCommand[]>((resolve) => {
          resolveDrain = resolve;
        }),
    );
    const apply = vi.fn();

    const stop = startGraphRendererCommandDrain({
      drain,
      apply,
      setIntervalFn: vi.fn().mockReturnValue(1),
      clearIntervalFn: vi.fn(),
    });

    stop();
    resolveDrain([{ type: 'inspect_node', node_id: 'node.ada' }]);
    await Promise.resolve();

    expect(apply).not.toHaveBeenCalled();
  });

  it('does not surface drain errors after the drain is stopped', async () => {
    let rejectDrain: (error: unknown) => void = () => {};
    const drain = vi.fn(
      () =>
        new Promise<GraphRendererCommand[]>((_resolve, reject) => {
          rejectDrain = reject;
        }),
    );
    const onError = vi.fn();

    const stop = startGraphRendererCommandDrain({
      drain,
      onError,
      setIntervalFn: vi.fn().mockReturnValue(1),
      clearIntervalFn: vi.fn(),
    });

    stop();
    rejectDrain(new Error('closed renderer'));
    await Promise.resolve();

    expect(onError).not.toHaveBeenCalled();
  });

  it('clears the polling interval only once when stopped repeatedly', () => {
    const intervalHandle = {};
    const clearIntervalFn = vi.fn();

    const stop = startGraphRendererCommandDrain({
      drain: vi.fn().mockResolvedValue([]),
      setIntervalFn: vi.fn().mockReturnValue(intervalHandle),
      clearIntervalFn,
    });

    stop();
    stop();

    expect(clearIntervalFn).toHaveBeenCalledTimes(1);
    expect(clearIntervalFn).toHaveBeenCalledWith(intervalHandle);
  });

  it('normalizes invalid polling intervals to the default cadence', () => {
    const setIntervalFn = vi.fn().mockReturnValue(1);

    const stop = startGraphRendererCommandDrain({
      intervalMs: 0,
      drain: vi.fn().mockResolvedValue([]),
      apply: vi.fn(),
      setIntervalFn,
      clearIntervalFn: vi.fn(),
    });

    expect(setIntervalFn).toHaveBeenCalledWith(expect.any(Function), 100);
    stop();
  });
});
