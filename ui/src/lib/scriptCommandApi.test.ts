import { afterEach, describe, expect, it, vi } from 'vitest';

import { setScriptBlock } from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('script command api helpers', () => {
  it('sends script block commands and returns versioned script projections', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 5,
        change_event_id: 'event-script-1',
        payload: {
          document: {
            id: 'script.document.main',
            title: 'Pilot',
            sort_order: 0,
          },
          segments: [
            {
              segment: {
                id: 'script.segment.beat-1',
                document_id: 'script.document.main',
                source_node_id: 'node.beat.opening',
                start_ms: 1000,
                end_ms: 5000,
                status: 'current',
                sort_order: 1,
              },
              blocks: [
                {
                  block: {
                    id: 'script.block.action-1',
                    segment_id: 'script.segment.beat-1',
                    block_kind: 'action',
                    text: 'Ada enters with a wet umbrella.',
                    sort_order: 2,
                  },
                  spans: [],
                  locks: [],
                },
              ],
            },
          ],
        },
      },
    };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(response), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      setScriptBlock(
        {
          document_id: 'script.document.main',
          document_title: 'Pilot',
          document_sort_order: 0,
          segment_id: 'script.segment.beat-1',
          source_node_id: 'node.beat.opening',
          segment_start_ms: 1000,
          segment_end_ms: 5000,
          segment_status: 'current',
          segment_sort_order: 1,
          block_id: 'script.block.action-1',
          block_kind: 'action',
          text: 'Ada enters with a wet umbrella.',
          sort_order: 2,
        },
        'command-script-1',
      ),
    ).resolves.toEqual(response);

    expect(fetchMock).toHaveBeenCalledWith(
      '/api/commands/script/block',
      expect.objectContaining({
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: 'command-script-1',
          payload: {
            document_id: 'script.document.main',
            document_title: 'Pilot',
            document_sort_order: 0,
            segment_id: 'script.segment.beat-1',
            source_node_id: 'node.beat.opening',
            segment_start_ms: 1000,
            segment_end_ms: 5000,
            segment_status: 'current',
            segment_sort_order: 1,
            block_id: 'script.block.action-1',
            block_kind: 'action',
            text: 'Ada enters with a wet umbrella.',
            sort_order: 2,
          },
        }),
      }),
    );
  });

  it('throws backend errors without local fallback state', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(JSON.stringify({ error: 'segment range invalid' }), {
          status: 400,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    );

    await expect(
      setScriptBlock(
        {
          document_id: 'script.document.main',
          document_title: 'Pilot',
          segment_id: 'script.segment.beat-1',
          segment_start_ms: 5000,
          segment_end_ms: 1000,
          segment_status: 'current',
          block_id: 'script.block.action-1',
          block_kind: 'action',
          text: 'Ada enters with a wet umbrella.',
        },
        'command-script-1',
      ),
    ).rejects.toThrow('segment range invalid');
  });
});
