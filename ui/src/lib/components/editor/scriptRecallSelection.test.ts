import { expect, it } from 'vitest';
import { packet } from './scriptRecallSelection.fixture.js';
import { buildScriptRecallSelection } from './scriptRecallSelection.js';
import { createScriptRecallDraft } from './scriptRecallDraft.svelte.js';

it('carries only chosen identities and exact name/path receipts without prompt values or mutable packet references', () => {
  const original = packet();
  const selected = buildScriptRecallSelection(original, ['house.0'])!;
  expect(selected.facts).toEqual([
    {
      node_id: 'House',
      part_key: 'profile',
      field_key: 'fact0',
      field_id: 'house.0',
      revision_event_id: 'revision.0',
    },
  ]);
  expect(selected.facts[0]).not.toHaveProperty('value');
  expect(selected.names.map((name) => name.node_id)).toEqual(['House', 'Mara']);
  expect(selected.paths[0]!.relationship.revision_event_id).toBe('path.revision');
  original.paths[0]!.relationship.edge.label = 'Changed';
  expect(selected.paths[0]!.relationship.edge.label).toBe('untimed home');
  expect(buildScriptRecallSelection(null, [])).toBeNull();
});

it('copies custom and builtin directed/undirected path kinds and query kinds without shared nested references', () => {
  const original = packet();
  original.request.edge_kinds = [{ custom: 'Authored association' }, 'located_in'];
  original.paths[0]!.relationship.edge.edge_kind = { custom: 'Authored association' };
  original.paths[0]!.relationship.edge.directed = false;
  const selected = buildScriptRecallSelection(original, ['house.0'])!;
  const queryKind = original.request.edge_kinds[0]!;
  const pathKind = original.paths[0]!.relationship.edge.edge_kind;
  if (typeof queryKind === 'object') queryKind.custom = 'Changed query';
  if (typeof pathKind === 'object') pathKind.custom = 'Changed path';
  expect(selected.query.edge_kinds).toEqual([{ custom: 'Authored association' }, 'located_in']);
  expect(selected.paths[0]!.relationship.edge.edge_kind).toEqual({
    custom: 'Authored association',
  });
  expect(selected.paths[0]!.relationship.edge.directed).toBe(false);
});

it('refuses timed, unresolved, omitted, unknown-name, duplicate and oversized selection instead of inferring truth', () => {
  for (const id of ['snapshot.field', 'unresolved', 'omitted']) {
    expect(() => buildScriptRecallSelection(packet(), [id])).toThrow(/cannot be selected/);
  }
  const timed = packet();
  timed.request.story_time_ms = 1000;
  expect(() => buildScriptRecallSelection(timed, ['house.0'])).toThrow(/unspecified/);
  const unknown = packet();
  unknown.nodes[1]!.name_revision_event_id = null;
  expect(() => buildScriptRecallSelection(unknown, ['house.0'])).toThrow(/unknown/);
  expect(() => buildScriptRecallSelection(packet(), ['house.0', 'house.0'])).toThrow(/Duplicate/);
  expect(
    buildScriptRecallSelection(
      packet(),
      Array.from({ length: 8 }, (_, index) => `house.${index}`),
    )!.facts,
  ).toHaveLength(8);
  expect(() =>
    buildScriptRecallSelection(
      packet(),
      Array.from({ length: 9 }, (_, index) => `house.${index}`),
    ),
  ).toThrow(/eight/);
});

it('retires intent on packet or target/session changes and refuses a submit before reactive cleanup', () => {
  const draft = createScriptRecallDraft();
  const evidence = packet();
  draft.select('B/session1/revision1', evidence, 'house.0', true);
  expect(draft.request('B/session1/revision1', evidence)!.facts).toHaveLength(1);
  expect(() => draft.request('B/session2/revision1', evidence)).toThrow(/changed/);
  expect(() => draft.request('B/session1/revision1', packet())).toThrow(/changed/);
  draft.observe('B/session1/revision1', null);
  expect(draft.state.fieldIds).toEqual([]);
  draft.select('B/session1/revision1', evidence, 'house.0', true);
  draft.observe('C/session1/revision1', evidence);
  expect(draft.request('C/session1/revision1', evidence)).toBeNull();
});
