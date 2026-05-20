import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

import { describe, expect, it } from 'vitest';

const sourceRoot = join(process.cwd(), 'src');
const checkedExtensions = new Set(['.svelte', '.ts']);

interface BannedPattern {
  pattern: RegExp;
  reason: string;
}

const bannedPatterns: BannedPattern[] = [
  {
    pattern: /\btimelineState\s*\.\s*timeline\b/,
    reason: 'timeline structure must come from timeline render projections',
  },
  {
    pattern: /\bselectedNode\s*:\s*StoryNode\b/,
    reason: 'selected-node durable data must come from selected-node projections',
  },
  {
    pattern: /\bselectedNode\s*=\s*[^=]/,
    reason: 'selected-node durable objects must not be stored in frontend state',
  },
  {
    pattern: /\b(?:getTimeline|getTimelineChildren|getTimelineGaps)\s*\(/,
    reason: 'legacy broad timeline reads must not be reintroduced',
  },
  {
    pattern: /\/timeline\/nodes\/\{id\}\/children|\/timeline\/gaps|\/nodes\/\{id\}\/content/,
    reason: 'legacy broad timeline/content routes must remain deleted',
  },
  {
    pattern: /\b(?:updateNodeNotes|lockNode|unlockNode|updateNodeScript)\s*\(/,
    reason: 'legacy node mutation helpers must not bypass backend commands',
  },
  {
    pattern: /\.projection\s*\.\s*payload\s*=/,
    reason: 'projection payloads must be replaced by envelope, not patched in place',
  },
];

function sourceFiles(dir: string): string[] {
  return readdirSync(dir).flatMap((entry) => {
    const path = join(dir, entry);
    const stat = statSync(path);

    if (stat.isDirectory()) {
      return sourceFiles(path);
    }

    for (const extension of checkedExtensions) {
      if (path.endsWith(extension) && !path.endsWith('projectionOnlyGuards.test.ts')) {
        return [path];
      }
    }

    return [];
  });
}

describe('projection-only frontend guardrails', () => {
  it('keeps deleted broad durable ownership patterns out of UI source', () => {
    const violations = sourceFiles(sourceRoot).flatMap((path) => {
      const content = readFileSync(path, 'utf8');

      return bannedPatterns
        .filter(({ pattern }) => pattern.test(content))
        .map(({ reason }) => `${relative(process.cwd(), path)}: ${reason}`);
    });

    expect(violations).toEqual([]);
  });
});
