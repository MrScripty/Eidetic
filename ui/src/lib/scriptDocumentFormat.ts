import type { ScriptBlockKind, ScriptDocumentProjection } from './scriptTypes.js';

export function scriptDocumentText(projection: ScriptDocumentProjection): string {
  return projection.segments
    .flatMap((segment) => segment.blocks)
    .map(({ block }) => formatBlockText(block.block_kind, block.text))
    .filter((text) => text.length > 0)
    .join('\n');
}

export function scriptDocumentBlockCount(projection: ScriptDocumentProjection): number {
  return projection.segments.reduce((count, segment) => count + segment.blocks.length, 0);
}

function formatBlockText(kind: ScriptBlockKind, text: string): string {
  const trimmed = text.trim();
  if (!trimmed) return '';

  if (kind === 'character') {
    return trimmed.toUpperCase();
  }

  if (kind === 'parenthetical' && !trimmed.startsWith('(') && !trimmed.endsWith(')')) {
    return `(${trimmed})`;
  }

  return trimmed;
}
