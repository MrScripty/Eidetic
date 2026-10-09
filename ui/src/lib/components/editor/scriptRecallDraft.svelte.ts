import type { BibleRecallProjection } from '$lib/bibleRecallTypes.js';
import { buildScriptRecallSelection } from './scriptRecallSelection.js';

/** Transient intent for one preview target and one displayed recall packet. */
export function createScriptRecallDraft() {
  const state = $state({ fieldIds: [] as string[], error: '' });
  let owner = '';
  let evidence: BibleRecallProjection | null = null;

  function observe(scope: string, packet: BibleRecallProjection | null): void {
    if (scope === owner && packet === evidence) return;
    owner = scope;
    evidence = packet;
    state.fieldIds = [];
    state.error = '';
  }
  function select(
    scope: string,
    packet: BibleRecallProjection | null,
    id: string,
    checked: boolean,
  ): void {
    observe(scope, packet);
    const ids = checked ? [...state.fieldIds, id] : state.fieldIds.filter((field) => field !== id);
    try {
      buildScriptRecallSelection(packet, ids);
      state.fieldIds = ids;
      state.error = '';
    } catch (error) {
      state.error = error instanceof Error ? error.message : 'Fact selection refused';
    }
  }
  function request(scope: string, packet: BibleRecallProjection | null) {
    if (!state.fieldIds.length) return null;
    if (owner !== scope || evidence !== packet) {
      throw new Error('Recall evidence or preview target changed. Select facts again.');
    }
    return buildScriptRecallSelection(packet, [...state.fieldIds]);
  }
  return { state, observe, select, request };
}
