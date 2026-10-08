import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('uses browser rune proxies without sending a proxy or mutating an uncertain title', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const source = fs.readFileSync('src/lib/components/editor/timelineTitleDraft.svelte.ts', 'utf8');
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
let compiled = compileModule(js, { filename: 'title.svelte.js', generate: 'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client')));
const { createTimelineTitleDraft } = await import('data:text/javascript;base64,' + Buffer.from(compiled).toString('base64'));
const calls = []; let reject = true;
const draft = createTimelineTitleDraft({ nodeId: 'A', commandId: () => 'fixed', read: async () => null,
  apply: async (payload, id) => { calls.push(structuredClone({payload,id})); if (reject) { reject = false; throw new Error('Lost acknowledgement'); } }
});
draft.initialize({ name: 'SCENE A', revision_event_id: null });
draft.state.name = '  Station departure — 雨.  ';
await draft.apply(); assert.equal(draft.state.uncertain, true);
draft.state.name = 'Later draft'; await draft.apply();
assert.deepEqual(calls[0], calls[1]);
assert.equal(calls[1].payload.name, '  Station departure — 雨.  '); assert.equal(calls[1].payload.expected.revision_event_id, null);
assert.equal(draft.state.saved, true); assert.equal(draft.state.base, null);
console.log('Browser title custody passed');
`,
  });
  expect(output).toContain('Browser title custody passed');
});
