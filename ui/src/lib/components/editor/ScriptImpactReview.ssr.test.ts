import { beforeEach, expect, it } from 'vitest';
import { render } from 'svelte/server';
import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
import {
  clearPropagationProposalListProjection,
  propagationProposalProjectionState,
} from '$lib/stores/propagationProposalProjection.svelte.js';
import ScriptImpactReview from './ScriptImpactReview.svelte';

const segment: ScriptSegmentProjection = {
  segment: {
    id: 'segment.B',
    document_id: 'main',
    source_node_id: 'node.B',
    start_ms: 1000,
    end_ms: 2000,
    status: 'current',
    sort_order: 0,
  },
  blocks: [
    {
      block: {
        id: 'block.B',
        segment_id: 'segment.B',
        block_kind: 'action',
        text: '  Canonical B — 雨\n\n',
        sort_order: 0,
      },
      revision_event_id: 'B-current',
      spans: [],
      locks: [],
    },
  ],
  impact: {
    generation_event_id: 'generation.B',
    output_block_id: 'block.B',
    lineage_available: true,
    needs_review: true,
    causes: [
      {
        dependency_id: 'B.input-A',
        input: { kind: 'script_block', block_id: 'block.A' },
        consumed_revision_event_id: 'A-old',
        current_revision_event_id: 'A-current',
        reason: 'changed',
        input_excerpt: 'Original A',
      },
    ],
  },
};

beforeEach(clearPropagationProposalListProjection);

it('renders exact proposed text alongside current canon and explicit review actions', () => {
  propagationProposalProjectionState.projection = {
    version: 2,
    payload: {
      proposals: [
        {
          id: 'review.B',
          action: 'patch_script_block',
          target: { kind: 'script_block', block_id: 'block.B' },
          status: 'pending',
          summary: 'Targeted update',
          proposed_text: '  Proposed <B> — 雨\n\n',
          created_at_ms: 10,
          script_review_binding: {
            request: {
              proposal_id: 'review.B',
              document_id: 'main',
              segment_id: 'segment.B',
              block_id: 'block.B',
              expected_block_revision_event_id: 'B-current',
              generation_event_id: 'generation.B',
              dependency_id: 'B.input-A',
              story_time_ms: null,
            },
            cause: segment.impact!.causes[0]!,
            target_segment_revision_event_id: 'B-placement',
            script_inputs: [],
            bible_context: { version: 1, payload: {} },
          },
        },
      ],
    },
  };
  const { body } = render(ScriptImpactReview, { props: { documentId: 'main', segment } });
  expect(body).toContain('  Canonical B — 雨\n\n');
  expect(body).toContain('  Proposed &lt;B> — 雨\n\n');
  expect(body).toContain('Accept update');
  expect(body).toContain('Reject');
  expect(segment.blocks[0]?.block.text).toBe('  Canonical B — 雨\n\n');
});

it('disables preview when the impact does not identify its generated output block', () => {
  const older = structuredClone(segment);
  delete older.impact!.output_block_id;
  const { body } = render(ScriptImpactReview, { props: { documentId: 'main', segment: older } });
  expect(body).toMatch(/<button[^>]*disabled[^>]*>Preview update<\/button>/);
});

it('identifies an authored Bible fact as a cause through the existing review surface', () => {
  const fact = structuredClone(segment);
  fact.impact!.causes[0]!.input = {
    kind: 'bible_field',
    node_id: 'Mara',
    part_key: 'profile',
    field_key: 'tagline',
    field_id: 'Mara.tagline',
  };
  const { body } = render(ScriptImpactReview, { props: { documentId: 'main', segment: fact } });
  expect(body).toContain('Bible fact profile.tagline changed.');
  expect(body).toContain('Preview update');
  expect(body).not.toMatch(/<button[^>]*disabled[^>]*>Preview update/);
  expect(fact.blocks[0]?.block.text).toBe('  Canonical B — 雨\n\n');
});
