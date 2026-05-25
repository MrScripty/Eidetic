import { describe, expect, it, vi } from 'vitest';

import { ensureGraphWorkspaceScaffoldProjection } from './graphWorkspaceBootstrap.js';

describe('graph workspace bootstrap', () => {
  it('ensures backend canonical roots before loading graph render projections', async () => {
    const calls: string[] = [];
    const request = {
      selected_node_id: 'node.character.ada',
      max_nodes: 120,
    };

    await ensureGraphWorkspaceScaffoldProjection(request, {
      ensureCanonicalRoots: vi.fn(async () => {
        calls.push('ensure-roots');
      }),
      refreshRenderGraph: vi.fn(async (receivedRequest) => {
        calls.push(`refresh:${receivedRequest.selected_node_id ?? 'none'}`);
      }),
    });

    expect(calls).toEqual(['ensure-roots', 'refresh:node.character.ada']);
  });

  it('does not refresh render graph projections when canonical root creation fails', async () => {
    const refreshRenderGraph = vi.fn();

    await expect(
      ensureGraphWorkspaceScaffoldProjection(
        {},
        {
          ensureCanonicalRoots: vi.fn(async () => {
            throw new Error('roots unavailable');
          }),
          refreshRenderGraph,
        },
      ),
    ).rejects.toThrow('roots unavailable');

    expect(refreshRenderGraph).not.toHaveBeenCalled();
  });
});
