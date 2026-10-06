import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('keeps interrupted relationship drafts and serializes label/delete actions with actual client proxies', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const js = ts.transpileModule(fs.readFileSync('src/lib/components/sidebar/bible/bibleGraphEdgeLabelEditor.svelte.ts', 'utf8'), {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
let compiled = compileModule(js, { filename: 'bibleGraphEdgeLabelEditor.svelte.js', generate: 'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')));
const { createBibleGraphEdgeLabelEditor } = await import('data:text/javascript;base64,' + Buffer.from(compiled).toString('base64'));
const original = { id: 'Mara.Eli', from_node_id: 'Mara', to_node_id: 'Eli', edge_kind: 'references', label: 'Trust', directed: true, sort_order: 7 };
let edges = [original], revisions = { 'Mara.Eli': 'original-revision' };
let saveCalls = [], deleteCalls = [], rejectSave, resolveDelete;
let pendingSave = () => new Promise((_, reject) => { rejectSave = reject; });
let pendingDelete = () => new Promise(resolve => { resolveDelete = resolve; });
const editor = createBibleGraphEdgeLabelEditor({ edges: () => edges, revisions: () => revisions,
  save: async (edge, payload) => { saveCalls.push({ edge, payload }); return pendingSave(); },
  remove: async edge => { deleteCalls.push(edge); return pendingDelete(); },
  refresh: async () => {}, confirm: () => true,
});
editor.edit(original);
editor.state.label = '  Exact retained manual label  ';
const saving = editor.save();
assert.equal(editor.state.saving, true);
assert.equal(editor.busy(), true);
await editor.remove(original); await editor.save(); editor.cancel(); editor.edit({ ...original, label: 'Other' });
assert.equal(saveCalls.length, 1); assert.equal(deleteCalls.length, 0);
assert.deepEqual(saveCalls[0].payload, { edge_id: 'Mara.Eli', label: 'Exact retained manual label', expected_revision_event_id: 'original-revision' });
assert.equal(editor.state.label, '  Exact retained manual label  ');
rejectSave(new Error('Relationship changed while editing; reopen its label editor.'));
await saving;
assert.equal(editor.state.saving, false); assert.equal(editor.state.edge.id, original.id);
assert.equal(editor.state.label, '  Exact retained manual label  ');
assert.match(editor.state.error, /Relationship changed/);
const deleting = editor.remove(original);
assert.equal(editor.state.deleting, original.id); assert.equal(editor.busy(), true);
await editor.save(); await editor.remove(original); editor.cancel();
assert.equal(saveCalls.length, 1); assert.equal(deleteCalls.length, 1);
edges = []; revisions = {};
resolveDelete(); await deleting;
assert.equal(editor.busy(), false); assert.equal(editor.state.edge.id, original.id);
assert.equal(editor.state.label, '  Exact retained manual label  ');
await editor.save(); assert.equal(saveCalls.length, 1); assert.match(editor.state.error, /Relationship changed/);
edges = [{ ...original, directed: false, edge_kind: 'conflicts_with' }]; revisions = { 'Mara.Eli': 'recreated-revision' };
await editor.save(); assert.equal(saveCalls.length, 1);
editor.edit(edges[0]); editor.state.label = 'Fresh manual label';
pendingSave = async () => {};
await editor.save(); assert.equal(saveCalls.length, 2);
assert.equal(saveCalls[1].payload.expected_revision_event_id, 'recreated-revision');
assert.equal(editor.state.edge, null);
// An interrupted delete also preserves its editable draft and clears the lock.
editor.edit(edges[0]); editor.state.label = 'Draft during failed delete';
pendingDelete = async () => { throw new Error('Interrupted delete'); };
await editor.remove(edges[0]);
assert.equal(editor.busy(), false); assert.equal(editor.state.label, 'Draft during failed delete');
assert.equal(editor.state.error, 'Interrupted delete');
console.log('Client label interruption and conflict controls passed');
`,
  });
  expect(output).toContain('Client label interruption and conflict controls passed');
});
