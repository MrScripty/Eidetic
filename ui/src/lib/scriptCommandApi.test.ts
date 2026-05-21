import { afterEach, describe, expect, it, vi } from 'vitest';

import { setScriptBlock, setScriptLock } from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('script command api helpers', () => {
  it('uses desktop script commands when Tauri transport is available', async () => {
    const blockResponse = {
      outcome: 'recorded',
      projection: {
        version: 5,
        payload: {
          document: {
            id: 'script.document.main',
            title: 'Pilot',
            sort_order: 0,
          },
          segments: [],
        },
      },
    };
    const lockResponse = {
      outcome: 'recorded',
      projection: {
        version: 6,
        payload: {
          document: {
            id: 'script.document.main',
            title: 'Pilot',
            sort_order: 0,
          },
          segments: [],
        },
      },
    };
    const invoke = vi.fn().mockResolvedValueOnce(blockResponse).mockResolvedValueOnce(lockResponse);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await setScriptBlock(
      {
        document_id: 'script.document.main',
        document_title: 'Pilot',
        segment_id: 'script.segment.beat-1',
        segment_start_ms: 1000,
        segment_end_ms: 5000,
        segment_status: 'current',
        block_id: 'script.block.action-1',
        block_kind: 'action',
        text: 'Ada enters with a wet umbrella.',
      },
      'command-script-1',
    );
    await setScriptLock(
      {
        lock_id: 'script.lock.action-1',
        span_id: 'script.block.action-1.span.main',
        reason: 'User approved wording.',
      },
      'command-lock-1',
    );

    expect(invoke).toHaveBeenNthCalledWith(1, 'command_script_block', {
      command: expect.objectContaining({ id: 'command-script-1' }),
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'command_script_lock', {
      command: expect.objectContaining({ id: 'command-lock-1' }),
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
