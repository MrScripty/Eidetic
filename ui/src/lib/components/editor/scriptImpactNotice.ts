import type { ScriptImpactCause } from '$lib/scriptTypes.js';

export function scriptImpactCauseLabel(cause: ScriptImpactCause): string {
  if (cause.reason === 'context_changed') {
    return cause.dependency_id.endsWith('.bible_context')
      ? 'Bible context membership changed.'
      : 'Screenplay context changed.';
  }
  const source =
    cause.input.kind === 'script_block'
      ? 'Source screenplay text'
      : cause.input.kind === 'script_segment'
        ? 'Source screenplay segment'
        : cause.input.kind === 'bible_field'
          ? `Bible fact ${cause.input.part_key}.${cause.input.field_key}`
          : cause.input.kind === 'bible_edge'
            ? 'Bible relationship'
            : 'Source input';
  return `${source} ${cause.reason === 'deleted' ? 'was removed.' : 'changed.'}`;
}
