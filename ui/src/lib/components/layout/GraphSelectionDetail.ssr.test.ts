import { expect, it, vi } from 'vitest';
import { render } from 'svelte/server';
import GraphSelectionDetail from './GraphSelectionDetail.svelte';
import type { ContextStackProjection } from '$lib/contextInfluenceTypes.js';
import type { BibleRenderGraphProjection } from '$lib/bibleGraphTypes.js';

vi.mock('$lib/stores/bible.svelte.js', () => ({ clearBibleGraphSelection: vi.fn() }));

const projection: BibleRenderGraphProjection = {
  nodes: [],
  edges: [],
  influences: [],
  neighborhoods: [],
};
const context: ContextStackProjection = {
  target_node_id: 'scene.B',
  layers: [
    {
      node_id: 'scene.B',
      level: 'Scene',
      label: 'B',
      role: 'target',
      distilled_context: 'Old recorded RED umbrella summary.',
      sort_order: 0,
    },
  ],
  script_context: [
    {
      document_id: 'script.document.main',
      segment_id: 'segment.A',
      block_id: 'block.A',
      source_node_id: 'scene.A',
      revision_event_id: 'saved-A',
      segment_revision_event_id: 'placement-A',
      start_ms: 0,
      end_ms: 1000,
      text: '  Exact human BLUE umbrella — 雨\n\n',
    },
  ],
};

function body(value: ContextStackProjection, target = 'scene.B') {
  return render(GraphSelectionDetail, {
    props: {
      projection,
      selection: { kind: 'context_layer', timelineNodeId: target },
      contextStack: value,
    },
  }).body;
}

it('shows exact saved neighboring screenplay and its custody separately from recorded summary', () => {
  const html = body(context);
  expect(html).toContain('Recorded summary');
  expect(html).toContain('Old recorded RED umbrella summary.');
  expect(html).toContain('Saved screenplay context');
  expect(html).toContain('Nearby screenplay');
  expect(html).toContain('  Exact human BLUE umbrella — 雨\n\n');
  expect(html).toContain('data-block-revision="saved-A"');
  expect(html).toContain('data-segment-revision="placement-A"');
  expect(html).toMatch(/generated replacements require\s+review/);
  expect(html).not.toContain('Accept update');
  expect(context.script_context?.[0]?.text).toBe('  Exact human BLUE umbrella — 雨\n\n');
});

it('distinguishes known empty from legacy unavailable evidence without displaying another target context', () => {
  expect(body({ ...context, script_context: [] })).toContain('No saved screenplay');
  expect(body({ ...context, script_context: undefined })).toContain(
    'Saved screenplay evidence is unavailable',
  );
  const other = body(context, 'scene.other');
  expect(other).not.toContain('BLUE umbrella');
  expect(other).not.toContain('Old recorded RED');
});
