import type { BibleGraphField } from '$lib/bibleGraphTypes.js';
import type { FieldValue } from '$lib/projectionTypes.js';

export function formatBibleFieldValue(value: FieldValue | null | undefined): string {
  if (!value) return '';
  if (value.type === 'object_ref') return `${value.value.kind}: ${value.value.id}`;
  if (value.type === 'bool') return value.value ? 'True' : 'False';
  return value.value.toString();
}

const baseOf = (field: BibleGraphField) =>
  JSON.stringify([field.part_id, field.field_key, field.value ?? null]);

/** Local draft bases survive canonical refresh; they are never silently rebased. */
export function createBibleGraphFieldDrafts(options: {
  owner: () => string;
  fields: () => BibleGraphField[];
  save: (field: BibleGraphField, text: string) => Promise<unknown>;
}) {
  const state = $state({
    owner: options.owner(),
    drafts: {} as Record<
      string,
      { text: string; base: string; baseText: string; conflict: boolean }
    >,
    saving: {} as Record<string, boolean>,
    errors: {} as Record<string, string | undefined>,
  });
  function observe() {
    const owner = options.owner();
    if (state.owner !== owner) {
      state.owner = owner;
      state.drafts = {};
      state.saving = {};
      state.errors = {};
    }
    for (const field of options.fields()) {
      const draft = state.drafts[field.id];
      if (draft && draft.base !== baseOf(field)) draft.conflict = true;
    }
  }
  function value(field: BibleGraphField): string {
    return state.drafts[field.id]?.text ?? formatBibleFieldValue(field.value);
  }
  function changed(field: BibleGraphField): boolean {
    const draft = state.drafts[field.id];
    return !!draft && (draft.conflict || draft.base !== baseOf(field));
  }
  function update(field: BibleGraphField, text: string): void {
    observe();
    if (state.saving[field.id] || !options.fields().includes(field)) return;
    const draft = state.drafts[field.id];
    state.drafts[field.id] = draft
      ? { ...draft, text }
      : {
          text,
          base: baseOf(field),
          baseText: formatBibleFieldValue(field.value),
          conflict: false,
        };
    state.errors[field.id] = undefined;
  }
  function discard(field: BibleGraphField): void {
    if (state.saving[field.id]) return;
    delete state.drafts[field.id];
    delete state.errors[field.id];
  }
  async function save(field: BibleGraphField): Promise<void> {
    observe();
    if (state.saving[field.id]) return;
    const currentField = options.fields().find((current) => current.id === field.id);
    if (!currentField || changed(currentField)) {
      state.errors[field.id] = 'Saved fact changed while editing. Your draft is preserved.';
      return;
    }
    const owner = options.owner(),
      submitted = value(currentField);
    state.saving[field.id] = true;
    state.errors[field.id] = undefined;
    try {
      await options.save(currentField, submitted.trim());
      if (options.owner() === owner && state.owner === owner) delete state.drafts[field.id];
    } catch (error) {
      if (options.owner() === owner && state.owner === owner)
        state.errors[field.id] = error instanceof Error ? error.message : 'Failed to save field';
    } finally {
      if (options.owner() === owner && state.owner === owner) state.saving[field.id] = false;
    }
  }
  return { state, observe, value, changed, update, discard, save };
}
