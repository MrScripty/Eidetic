import { describe, expect, it } from 'vitest';
import { render } from 'svelte/server';
import ScriptTimelineNotesEvidence from './ScriptTimelineNotesEvidence.svelte';
import { scriptImpactCauseLabel } from './scriptImpactNotice.js';
import type { TimelineNotesInput } from '$lib/propagationProposalTypes.js';

const previous: TimelineNotesInput = {
  node_id: 'scene.B',
  notes: '  Conceal the witness — 雨.\n\n  ',
  revision_event_id: 'notes.original',
};

describe('timeline Notes review evidence', () => {
  it('shows exact original/current text and separate owned revisions in the existing review', () => {
    const current = {
      ...previous,
      notes: '  Reveal the witness — 雨.\n\n  ',
      revision_event_id: 'notes.current',
    };
    const body = render(ScriptTimelineNotesEvidence, { props: { previous, current } }).body;
    expect(body).toContain(previous.notes);
    expect(body).toContain(current.notes);
    expect(body).toContain('notes.original');
    expect(body).toContain('notes.current');
    expect(body).toMatch(/Accept update replaces only the\s+selected block\./);
  });

  it('distinguishes cleared Notes and field ABA from missing original receipts', () => {
    const cleared = render(ScriptTimelineNotesEvidence, {
      props: { previous, current: { ...previous, notes: '', revision_event_id: 'notes.cleared' } },
    }).body;
    expect(cleared).toContain('(cleared)');
    expect(cleared).toContain('notes.cleared');
    expect(cleared).not.toContain('Original Notes consumption is unknown.');
    const restored = render(ScriptTimelineNotesEvidence, {
      props: { previous, current: { ...previous, revision_event_id: 'notes.restored' } },
    }).body;
    expect(restored).toContain('notes.original');
    expect(restored).toContain('notes.restored');
  });

  it('keeps absent legacy consumption and unbound history explicit without inventing text', () => {
    const body = render(ScriptTimelineNotesEvidence, {
      props: { previous: null, current: { ...previous, revision_event_id: null } },
    }).body;
    expect(body).toContain('Original Notes consumption is unknown.');
    expect(body).toContain('unknown; unbound history');
    expect(body).not.toContain('Originally consumed Notes');
    expect(render(ScriptTimelineNotesEvidence, { props: {} }).body).not.toContain(
      'Recorded timeline Notes evidence',
    );
  });

  it('labels Notes change/removal separately from screenplay window changes', () => {
    const cause = {
      dependency_id: 'generation.B.timeline_notes',
      input: { kind: 'timeline_node' as const, node_id: 'scene.B' },
      consumed_revision_event_id: 'old',
      current_revision_event_id: 'new',
      reason: 'changed' as const,
      input_excerpt: previous.notes,
    };
    expect(scriptImpactCauseLabel(cause)).toBe('Timeline Notes changed.');
    expect(scriptImpactCauseLabel({ ...cause, reason: 'deleted' })).toBe(
      'Timeline Notes were removed.',
    );
    expect(
      scriptImpactCauseLabel({
        ...cause,
        dependency_id: 'generation.B.context',
        reason: 'context_changed',
      }),
    ).toBe('Screenplay context changed.');
  });
});
