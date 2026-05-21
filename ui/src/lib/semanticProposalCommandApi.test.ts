import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  acceptBibleReferenceProposal,
  createBibleReferenceProposal,
  rejectBibleReferenceProposal,
} from './commandApi.js';

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe('semantic proposal command api helpers', () => {
  it('uses desktop bible reference proposal commands when Tauri transport is available', async () => {
    const response = {
      outcome: 'recorded',
      projection: {
        version: 4,
        payload: {
          proposals: [],
        },
      },
    };
    const invoke = vi.fn().mockResolvedValue(response);
    vi.stubGlobal('window', {
      __TAURI__: {
        core: { invoke },
      },
    });
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      createBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          source_node_id: 'node.scene.one',
          child_name: 'First encounter',
          reference_kind: 'character',
          reference_text: 'Ada',
          rationale: 'Named in the generated child plan',
        },
        'command-proposal-create',
      ),
    ).resolves.toEqual(response);

    await expect(
      rejectBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          reason: 'Duplicate',
        },
        'command-proposal-reject',
      ),
    ).resolves.toEqual(response);

    await expect(
      acceptBibleReferenceProposal(
        {
          proposal_id: 'proposal-ada',
          node_id: 'bible.character.ada',
          parent_id: 'bible.root.characters',
          name: 'Ada',
          sort_order: 20,
        },
        'command-proposal-accept',
      ),
    ).resolves.toEqual(response);

    expect(invoke).toHaveBeenNthCalledWith(1, 'command_bible_reference_proposal_create', {
      command: {
        id: 'command-proposal-create',
        payload: {
          proposal_id: 'proposal-ada',
          source_node_id: 'node.scene.one',
          child_name: 'First encounter',
          reference_kind: 'character',
          reference_text: 'Ada',
          rationale: 'Named in the generated child plan',
        },
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(2, 'command_bible_reference_proposal_reject', {
      command: {
        id: 'command-proposal-reject',
        payload: {
          proposal_id: 'proposal-ada',
          reason: 'Duplicate',
        },
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(3, 'command_bible_reference_proposal_accept', {
      command: {
        id: 'command-proposal-accept',
        payload: {
          proposal_id: 'proposal-ada',
          node_id: 'bible.character.ada',
          parent_id: 'bible.root.characters',
          name: 'Ada',
          sort_order: 20,
        },
      },
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });
});
