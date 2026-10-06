import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('uses browser rune proxies without sending a proxy or mutating an uncertain placement', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const source = fs.readFileSync('src/lib/components/editor/timelinePlacementDraft.svelte.ts', 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
let compiled = compileModule(js, { filename: 'placement.svelte.js', generate: 'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')));
const { createTimelinePlacementDraft } = await import('data:text/javascript;base64,' + Buffer.from(compiled).toString('base64'));
const calls = []; let reject = true;
const draft = createTimelinePlacementDraft({ nodeId: 'A', commandId: () => 'fixed', read: async () => null,
  apply: async (payload, id) => { calls.push(structuredClone({payload,id})); if (reject) { reject = false; throw new Error('Lost acknowledgement'); } }
});
draft.initialize({ start_ms: 1000, end_ms: 2000, node_revision_event_id: null });
draft.state.start = '6.125'; draft.state.end = '7';
await draft.apply(); assert.equal(draft.state.uncertain, true);
draft.state.start = '9'; await draft.apply();
assert.deepEqual(calls[0], calls[1]);
assert.equal(calls[1].payload.start_ms, 6125); assert.equal(calls[1].payload.expected.node_revision_event_id, null);
assert.equal(draft.state.saved, true); assert.equal(draft.state.base, null);
console.log('Browser placement custody passed');
`,
  });
  expect(output).toContain('Browser placement custody passed');
});
