import { beforeEach, expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptImpactReview from './ScriptImpactReview.svelte';
import type { PropagationProposal } from '$lib/propagationProposalTypes.js';
import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
import {
  clearPropagationProposalListProjection,
  propagationProposalProjectionState,
} from '$lib/stores/propagationProposalProjection.svelte.js';
import {
  getSessionScriptBlockEditDraft,
  resetSessionScriptBlockEditDrafts,
} from '$lib/stores/scriptBlockEditSession.svelte.js';

beforeEach(() => {
  clearPropagationProposalListProjection();
  resetSessionScriptBlockEditDrafts();
});

it('shows Notes evidence beside saved/proposed screenplay and explicit acceptance while retaining authored drafts and unrelated proposals', () => {
  const cause = {
    dependency_id: 'generation.B.timeline_notes',
    input: { kind: 'timeline_node' as const, node_id: 'scene.B' },
    consumed_revision_event_id: 'notes.original',
    current_revision_event_id: 'notes.current',
    reason: 'changed' as const,
    input_excerpt: 'Original Notes',
  };
  const segment: ScriptSegmentProjection = {
    segment: {
      id: 'segment.B',
      document_id: 'main',
      source_node_id: 'scene.B',
      start_ms: 120000,
      end_ms: 180000,
      status: 'current',
      sort_order: 1,
    },
    impact: {
      generation_event_id: 'generation.B',
      output_block_id: 'block.B',
      lineage_available: true,
      needs_review: true,
      causes: [cause],
    },
    blocks: [
      {
        block: {
          id: 'block.B',
          segment_id: 'segment.B',
          block_kind: 'action',
          text: '  Saved screenplay stays exact — 雨.\n\n  ',
          sort_order: 0,
        },
        revision_event_id: 'saved.B',
        spans: [],
        locks: [],
      },
    ],
  };
  const own = getSessionScriptBlockEditDraft('main', 'block.B');
  const unrelated = getSessionScriptBlockEditDraft('main', 'block.F');
  own.begin(segment.blocks[0]!);
  unrelated.begin({ ...segment.blocks[0]!, block: { ...segment.blocks[0]!.block, id: 'block.F' } });
  own.state.text = '  Own exact unsaved draft — 雨.\n\n  ';
  unrelated.state.text = '  Unrelated exact draft — whistle.\n\n  ';
  const proposal: PropagationProposal = {
    id: 'notes.preview.B',
    action: 'patch_script_block',
    target: { kind: 'script_block', block_id: 'block.B' },
    status: 'pending',
    summary: 'Synthetic Notes preview',
    proposed_text: '  Synthetic proposed scene — 雨.\n\n  ',
    created_at_ms: 1,
    script_review_binding: {
      request: {
        proposal_id: 'notes.preview.B',
        document_id: 'main',
        segment_id: 'segment.B',
        block_id: 'block.B',
        expected_block_revision_event_id: 'saved.B',
        generation_event_id: 'generation.B',
        dependency_id: cause.dependency_id,
      },
      cause,
      target_segment_revision_event_id: 'segment.B.current',
      script_inputs: [],
      bible_context: { version: 1, payload: {} },
      timeline_notes_previous: {
        node_id: 'scene.B',
        notes: '  Original exact Notes — 雨.\n\n  ',
        revision_event_id: 'notes.original',
      },
      timeline_notes_current: {
        node_id: 'scene.B',
        notes: '  Current exact Notes — 雨.\n\n  ',
        revision_event_id: 'notes.current',
      },
      ancestor_notes_previous: [
        {
          node_id: 'act.A',
          notes: '  Exact original Act Notes — 雨.\n\n  ',
          revision_event_id: 'act.original',
        },
      ],
      ancestor_notes_current: [
        {
          node_id: 'act.A',
          notes: '  Exact current Act Notes — 雨.\n\n  ',
          revision_event_id: 'act.current',
        },
      ],
    },
  };
  const other = {
    ...proposal,
    id: 'notes.preview.F',
    proposed_text: 'Unrelated proposed screenplay',
    script_review_binding: {
      ...proposal.script_review_binding!,
      request: { ...proposal.script_review_binding!.request, segment_id: 'segment.F' },
    },
  };
  propagationProposalProjectionState.projection = {
    version: 1,
    payload: { proposals: [proposal, other] },
  };
  const body = render(ScriptImpactReview, { props: { documentId: 'main', segment } }).body;
  expect(body).toContain('Timeline Notes changed.');
  expect(body).toContain(segment.blocks[0]!.block.text);
  expect(body).toContain(proposal.proposed_text!);
  expect(body).toContain(proposal.script_review_binding!.timeline_notes_previous!.notes);
  expect(body).toContain(proposal.script_review_binding!.timeline_notes_current!.notes);
  expect(body).toContain('Ancestor Notes used for this update');
  expect(body).toContain(proposal.script_review_binding!.ancestor_notes_previous![0]!.notes);
  expect(body).toContain(proposal.script_review_binding!.ancestor_notes_current![0]!.notes);
  expect(body).toContain('Accept update');
  expect(body).toContain('Reject');
  expect(body).not.toContain('Unrelated proposed screenplay');
  expect(own.state.text).toBe('  Own exact unsaved draft — 雨.\n\n  ');
  expect(unrelated.state.text).toBe('  Unrelated exact draft — whistle.\n\n  ');
  expect(propagationProposalProjectionState.projection!.payload.proposals).toEqual([
    proposal,
    other,
  ]);
});
