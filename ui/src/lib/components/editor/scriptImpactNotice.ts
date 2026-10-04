import type { ScriptImpactCause } from '$lib/scriptTypes.js';

export function scriptImpactCauseLabel(cause: ScriptImpactCause): string {
  const source =
    cause.input.kind === 'script_block'
      ? 'Source screenplay text'
      : cause.input.kind === 'script_segment'
        ? 'Source screenplay segment'
        : 'Source input';
  return `${source} ${cause.reason === 'deleted' ? 'was removed.' : 'changed.'}`;
}
