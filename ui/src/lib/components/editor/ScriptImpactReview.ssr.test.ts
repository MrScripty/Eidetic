import { beforeEach, expect, it } from 'vitest';
import { render } from 'svelte/server';
import type { ScriptSegmentProjection } from '$lib/scriptTypes.js';
import {
  clearPropagationProposalListProjection,
  propagationProposalProjectionState,
} from '$lib/stores/propagationProposalProjection.svelte.js';
import ScriptImpactReview from './ScriptImpactReview.svelte';
import ScriptImpactNotice from './ScriptImpactNotice.svelte';

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

it('explains changed and removed consumed relationships without altering saved text', () => {
  const relationship = structuredClone(segment);
  relationship.impact!.causes[0]!.input = { kind: 'bible_edge', edge_id: 'Mara.Eli' };
  relationship.impact!.causes[0]!.input_excerpt = 'Mara trusts Eli';
  const notice = render(ScriptImpactNotice, { props: { impact: relationship.impact! } });
  expect(notice.body).toContain('Bible relationship changed.');
  expect(notice.body).toContain('Mara trusts Eli');
  relationship.impact!.causes[0]!.reason = 'deleted';
  const review = render(ScriptImpactReview, {
    props: { documentId: 'main', segment: relationship },
  });
  expect(review.body).toContain('Bible relationship was removed.');
  expect(review.body).toContain('Preview update');
  expect(relationship.blocks[0]!.block.text).toBe(segment.blocks[0]!.block.text);
});

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
            bible_node_name_inputs: [
              {
                node_id: 'Mara',
                name: 'Captured <Marisol>',
                revision_event_id: 'name-original-revision',
              },
            ],
            bible_relationship_inputs: [
              {
                edge: {
                  edge_id: 'Mara.Eli',
                  from_node_id: 'Mara',
                  to_node_id: 'Eli',
                  edge_kind: 'references',
                  label: 'Original captured <relationship>',
                  directed: true,
                },
                revision_event_id: 'edge-original-revision',
              },
            ],
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
  expect(body).toContain('Names used for this preview');
  expect(body).toContain('Captured &lt;Marisol>');
  expect(body).toContain('name-original-revision');
  expect(body).toContain('Relationships used for this preview');
  expect(body).toContain('Original captured &lt;relationship>');
  expect(body).toContain('edge-original-revision');
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

it('explains scenes entering and leaving the consumed screenplay window', () => {
  const moved = structuredClone(segment);
  moved.impact!.causes[0]!.input = { kind: 'timeline_node', node_id: 'node.B' };
  moved.impact!.causes[0]!.reason = 'context_changed';
  moved.impact!.causes[0]!.input_excerpt = 'Entered: E. Left: A.';
  const { body } = render(ScriptImpactReview, { props: { documentId: 'main', segment: moved } });
  const notice = render(ScriptImpactNotice, { props: { impact: moved.impact! } });
  expect(body).toContain('Screenplay context changed.');
  expect(notice.body).toContain('Entered: E. Left: A.');
  expect(body).toContain('Preview update');
  expect(body).not.toMatch(/<button[^>]*disabled[^>]*>Preview update/);
  expect(moved.blocks[0]?.block.text).toBe('  Canonical B — 雨\n\n');
});

it('renders Bible membership changes with exact field evidence and preserves canon until review', () => {
  const target = structuredClone(segment);
  target.impact!.causes = [
    {
      dependency_id: 'generation.B.bible_context',
      input: { kind: 'timeline_node', node_id: 'node.B' },
      consumed_revision_event_id: 'old-fields',
      current_revision_event_id: 'new-fields',
      reason: 'context_changed',
      input_excerpt: 'Untimed Bible fields entered: Mara.profile.motivation; removed: ',
    },
  ];
  const notice = render(ScriptImpactNotice, { props: { impact: target.impact! } });
  expect(notice.body).toContain('Bible context membership changed.');
  expect(notice.body).toContain('Mara.profile.motivation');
  expect(target.blocks[0]!.block.text).toBe('  Canonical B — 雨\n\n');
  target.impact!.causes[0]!.dependency_id = 'generation.B.context';
  expect(render(ScriptImpactNotice, { props: { impact: target.impact! } }).body).toContain(
    'Screenplay context changed.',
  );
});

it('explains a consumed Bible name edit and removal through explicit screenplay review', () => {
  const renamed = structuredClone(segment);
  const cause = renamed.impact!.causes[0]!;
  cause.input = { kind: 'bible_node', node_id: 'Mara' };
  cause.input_excerpt = 'Mara';
  const changed = render(ScriptImpactReview, { props: { documentId: 'main', segment: renamed } });
  expect(changed.body).toContain('Bible name changed.');
  expect(changed.body).toContain('Preview update');
  cause.reason = 'deleted';
  const removed = render(ScriptImpactNotice, { props: { impact: renamed.impact! } });
  expect(removed.body).toContain('Bible name was removed.');
  expect(removed.body).toContain('Mara');
  expect(renamed.blocks[0]!.block.text).toBe(segment.blocks[0]!.block.text);
});
