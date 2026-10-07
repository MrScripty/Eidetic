import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptArcEvidence from './ScriptArcEvidence.svelte';
import { scriptImpactCauseLabel } from './scriptImpactNotice.js';
import type { StoryArcFieldInput } from '$lib/storyArcTypes.js';

const before: StoryArcFieldInput = {
  arc_id: 'arc.witness',
  field: 'description',
  value: "Mara conceals the witness's identity.\nExact authored continuation.",
  revision_event_id: 'original-revision',
};

describe('recorded screenplay arc evidence', () => {
  it('shows exact original and changed description with their separate source revisions', () => {
    const current = {
      ...before,
      value: "Mara reveals the witness's identity.",
      revision_event_id: 'changed-revision',
    };
    const { body } = render(ScriptArcEvidence, {
      props: { previous: [before], current: [current] },
    });
    expect(body).toContain('Arc input change');
    expect(body).toContain('Exact authored continuation.');
    expect(body).toContain('conceals');
    expect(body).toContain('reveals');
    expect(body).toContain('original-revision');
    expect(body).toContain('changed-revision');
  });

  it('distinguishes cleared fields from removed arcs and records deletion authority', () => {
    const cleared = render(ScriptArcEvidence, {
      props: {
        previous: [before],
        current: [{ ...before, value: '', revision_event_id: 'cleared' }],
      },
    }).body;
    expect(cleared).toContain('(cleared)');
    expect(cleared).not.toContain('(removed)');
    const deleted = render(ScriptArcEvidence, {
      props: { previous: [before], current: [], absent: [['arc.witness', 'deleted-revision']] },
    }).body;
    expect(deleted).toContain('(removed)');
    expect(deleted).toContain('Deletion revision deleted-revision');
  });

  it('labels missing original receipts and unbound template history without inventing authority', () => {
    const { body } = render(ScriptArcEvidence, {
      props: { previous: null, current: [{ ...before, revision_event_id: null }] },
    });
    expect(body).toContain('Original arc consumption is unknown.');
    expect(body).toContain('unknown; unbound history');
    expect(body).not.toContain('Arc input change');
  });

  it('labels description change and removal as arc causes in the existing review chooser', () => {
    const cause = {
      dependency_id: 'generation.arc.description',
      input: {
        kind: 'story_arc_field' as const,
        arc_id: 'arc.witness',
        field: 'description' as const,
      },
      consumed_revision_event_id: 'old',
      current_revision_event_id: 'new',
      reason: 'changed' as const,
      input_excerpt: before.value,
    };
    expect(scriptImpactCauseLabel(cause)).toBe('Story arc description changed.');
    expect(
      scriptImpactCauseLabel({ ...cause, reason: 'deleted', current_revision_event_id: null }),
    ).toBe('Story arc description was removed.');
  });
});
