import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('keeps bounded selections and exact receipts with actual browser rune proxies', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const transpile = path => ts.transpileModule(fs.readFileSync(path, 'utf8'), { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
const url = source => 'data:text/javascript;base64,' + Buffer.from(source).toString('base64');
const builder = url(transpile('src/lib/components/editor/scriptRecallSelection.ts'));
let compiled = compileModule(transpile('src/lib/components/editor/scriptRecallDraft.svelte.ts'), { filename: 'recall.svelte.js', generate: 'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')));
compiled = compiled.replace('./scriptRecallSelection.js', builder);
const { createScriptRecallDraft } = await import(url(compiled));
const draft = createScriptRecallDraft();
let proxied = compileModule('export function proxy(value) { let packet = $state(value); return packet; }', { filename:'packet.svelte.js', generate:'client' }).js.code;
proxied = proxied.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')));
const { proxy } = await import(url(proxied));
const packet = proxy({ request: { anchor_node_id: 'Mara', story_time_ms: null, direction: 'both', edge_kinds: [{custom:'Home'}], neighbor_limit: 8 },
 nodes: [{ node_id: 'Mara', name: 'Mara', name_revision_event_id: 'name1', fields: [] },
 { node_id: 'House', name: 'House', name_revision_event_id: 'name2', fields: Array.from({length:9}, (_,i) => ({
 part_key: 'profile', field_key: 'f'+i, source: { kind:'baseline', field_id:'f'+i, revision_event_id:'r'+i }, value:{type:'text',value:'Fact '+i} })) }],
 paths: [{neighbor_node_id:'House',relationship:{revision_event_id:'path1',edge:{edge_id:'home',from_node_id:'Mara',to_node_id:'House',edge_kind:{custom:'Home'},directed:false,label:'Home'}}}] });
for (let i=0; i<8; i++) draft.select('B/session1', packet, 'f'+i, true);
draft.select('B/session1', packet, 'f8', true);
assert.equal(draft.state.fieldIds.length, 8);
assert.match(draft.state.error, /eight/);
const receipt = draft.request('B/session1', packet);
assert.equal(structuredClone(receipt).facts[0].revision_event_id, 'r0');
packet.request.edge_kinds[0].custom = 'Changed';
packet.paths[0].relationship.edge.edge_kind.custom = 'Changed';
assert.equal(receipt.query.edge_kinds[0].custom, 'Home');
assert.equal(receipt.paths[0].relationship.edge.edge_kind.custom, 'Home');
draft.select('B/session1', packet, 'f0', false);
assert.equal(draft.state.fieldIds.length, 7);
assert.throws(() => draft.request('B/session2', packet), /changed/);
draft.observe('B/session2', packet);
assert.equal(draft.request('B/session2', packet), null);
console.log('Browser recall selection custody passed');
`,
  });
  expect(output).toContain('Browser recall selection custody passed');
});
