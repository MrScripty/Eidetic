import { describe, expect, it } from 'vitest';

import { scriptDocumentBlockCount, scriptDocumentText } from './scriptDocumentFormat.js';
import type { ScriptDocumentProjection } from './scriptTypes.js';

describe('script document formatting', () => {
  it('formats ordered projection blocks as screenplay text', () => {
    const projection = scriptProjection([
      ['scene_heading', 'int. kitchen - morning'],
      ['action', 'Ada enters with a wet umbrella.'],
      ['character', 'ada'],
      ['parenthetical', 'quietly'],
      ['dialogue', 'It followed me home.'],
    ]);

    expect(scriptDocumentText(projection)).toBe(
      [
        'int. kitchen - morning',
        'Ada enters with a wet umbrella.',
        'ADA',
        '(quietly)',
        'It followed me home.',
      ].join('\n'),
    );
    expect(scriptDocumentBlockCount(projection)).toBe(5);
  });

  it('drops empty blocks while preserving parenthetical text that is already wrapped', () => {
    const projection = scriptProjection([
      ['action', '   '],
      ['parenthetical', '(under her breath)'],
      ['dialogue', 'Not again.'],
    ]);

    expect(scriptDocumentText(projection)).toBe('(under her breath)\nNot again.');
    expect(scriptDocumentBlockCount(projection)).toBe(3);
  });
});

function scriptProjection(
  blocks: Array<
    [ScriptDocumentProjection['segments'][number]['blocks'][number]['block']['block_kind'], string]
  >,
): ScriptDocumentProjection {
  return {
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
        blocks: blocks.map(([block_kind, text], index) => ({
          block: {
            id: `script.block.${index}`,
            segment_id: 'script.segment.beat-1',
            block_kind,
            text,
            sort_order: index,
          },
          spans: [],
          locks: [],
        })),
      },
    ],
  };
}
