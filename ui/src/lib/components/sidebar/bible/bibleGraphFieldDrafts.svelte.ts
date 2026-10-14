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
  verified: () => boolean;
  save: (field: BibleGraphField, text: string) => Promise<BibleGraphField | undefined>;
}) {
  // Opaque request identities must not participate in the observe effect.
  const requests: Record<string, { submitted: string } | undefined> = {};
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
      for (const id of Object.keys(requests)) delete requests[id];
    }
    for (const field of options.fields()) {
      const draft = state.drafts[field.id];
      if (
        draft &&
        draft.base !== baseOf(field) &&
        !matchesSubmitted(field, requests[field.id]?.submitted)
      )
        draft.conflict = true;
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
  function matchesSubmitted(field: BibleGraphField, submitted: string | undefined): boolean {
    return (
      submitted !== undefined &&
      JSON.stringify(field.value ?? null) ===
        JSON.stringify(submitted ? { type: 'text', value: submitted } : null)
    );
  }
  async function save(field: BibleGraphField): Promise<void> {
    observe();
    if (state.saving[field.id]) return;
    if (!options.verified()) {
      state.errors[field.id] = 'Verify saved facts before saving. Your draft is preserved.';
      return;
    }
    const currentField = options.fields().find((current) => current.id === field.id);
    if (!currentField || changed(currentField)) {
      state.errors[field.id] = 'Saved fact changed while editing. Your draft is preserved.';
      return;
    }
    const owner = options.owner(),
      submitted = value(currentField),
      draft = state.drafts[field.id],
      request = { submitted };
    requests[field.id] = request;
    const owned = () =>
      options.owner() === owner && state.owner === owner && requests[field.id] === request;
    state.saving[field.id] = true;
    state.errors[field.id] = undefined;
    try {
      const acknowledged = await options.save(currentField, request.submitted);
      if (!owned()) return;
      observe();
      const committed = options.fields().find((current) => current.id === field.id);
      if (
        acknowledged &&
        committed &&
        acknowledged.id === field.id &&
        baseOf(acknowledged) === baseOf(committed) &&
        matchesSubmitted(acknowledged, request.submitted) &&
        state.drafts[field.id] === draft &&
        !draft?.conflict
      )
        delete state.drafts[field.id];
    } catch (error) {
      if (owned())
        state.errors[field.id] = error instanceof Error ? error.message : 'Failed to save field';
    } finally {
      if (owned()) {
        delete requests[field.id];
        state.saving[field.id] = false;
        observe();
      }
    }
  }
  return { state, observe, value, changed, update, discard, save };
}
