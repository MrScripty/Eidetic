import type { ScriptImpactCause } from '$lib/scriptTypes.js';

export function scriptImpactCauseLabel(cause: ScriptImpactCause): string {
  if (
    cause.input.kind === 'timeline_node' &&
    cause.dependency_id.endsWith(`.ancestor_notes.${cause.input.node_id}`)
  ) {
    return `Ancestor timeline Notes ${cause.reason === 'deleted' ? 'were removed.' : 'changed.'}`;
  }
  if (cause.input.kind === 'timeline_node' && cause.dependency_id.endsWith('.timeline_notes')) {
    return `Timeline Notes ${cause.reason === 'deleted' ? 'were removed.' : 'changed.'}`;
  }
  if (
    cause.input.kind === 'story_arc_field' &&
    cause.dependency_id.endsWith(`.arc_description_applicability.${cause.input.arc_id}`)
  ) {
    return 'Story arc description became available.';
  }
  if (cause.reason === 'context_changed') {
    return cause.dependency_id.endsWith('.bible_context')
      ? 'Bible context membership changed.'
      : 'Screenplay context changed.';
  }
  if (cause.input.kind === 'story_arc_field') {
    return `Story arc ${cause.input.field.replaceAll('_', ' ')} ${cause.reason === 'deleted' ? 'was removed.' : 'changed.'}`;
  }
  const source =
    cause.input.kind === 'script_block'
      ? 'Source screenplay text'
      : cause.input.kind === 'script_segment'
        ? 'Source screenplay segment'
        : cause.input.kind === 'bible_field'
          ? `Bible fact ${cause.input.part_key}.${cause.input.field_key}`
          : cause.input.kind === 'bible_node'
            ? 'Bible name'
            : cause.input.kind === 'bible_edge'
              ? 'Bible relationship'
              : 'Source input';
  return `${source} ${cause.reason === 'deleted' ? 'was removed.' : 'changed.'}`;
}
