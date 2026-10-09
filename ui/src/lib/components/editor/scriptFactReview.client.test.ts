import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('retains exact retry and retires late results with actual client rune state', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const url = source => 'data:text/javascript;base64,' + Buffer.from(source).toString('base64');
const stub = url('export function getPropagationProposalSessionEpoch() { return 0; } export async function applyRequestScriptFactProposalCommand(...args) { return globalThis.provider(...args); } export async function applyScriptFactDecisionCommand(...args) { return globalThis.decision(...args); }');
const source = ts.transpileModule(fs.readFileSync('src/lib/components/editor/scriptFactReview.svelte.ts', 'utf8'), { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
let compiled = compileModule(source, { filename:'review.svelte.js', generate:'client' }).js.code;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client'))).replace('svelte/reactivity', import.meta.resolve('svelte/reactivity')).replace('$lib/stores/propagationProposalProjection.svelte.js',stub);
const { createScriptFactReview } = await import(url(compiled));
let owner = 'saved.B';
const review = createScriptFactReview(() => owner);
const payload = { document_id:'main',segment_id:'B',block_id:'B',expected_block_revision_event_id:'saved.B',expected_field_revision_event_id:'fact.rev',generation_event_id:'gen.B',dependency_id:'Mara.consumed' };
const calls=[];
globalThis.provider = async (...args) => { calls.push(args); if(calls.length===1)throw Error('Lost acknowledgement'); };
await review.analyze(payload); assert.equal(review.state.retry,true);
await review.analyze(payload); assert.deepEqual(calls[1],calls[0]);
let finish;
globalThis.provider = () => new Promise(resolve => finish=resolve);
const pending = review.analyze(payload); assert.equal(review.state.pending,true);
await review.analyze(payload); owner='new.saved.B'; finish(); await pending;
assert.equal(review.state.retry,false); assert.match(review.state.error,/evidence changed/);
assert.equal(review.state.pending,false);
console.log('client rune fact review passed');
`,
  });
  expect(output).toContain('client rune fact review passed');
});
