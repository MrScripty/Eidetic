import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('retires old project refreshes with the actual client proposal store', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const url = source => 'data:text/javascript;base64,' + Buffer.from(source).toString('base64');
const commands = url('export async function acceptPropagationProposal() {} export async function requestScriptImpactProposal() {} export async function requestScriptFactProposal() {} export async function createPropagationProposal() {} export async function rejectPropagationProposal() {} export async function updatePropagationProposal() {}');
const reads = url('export function getPropagationProposalListProjection() { return globalThis.read(); }');
const source = ts.transpileModule(fs.readFileSync('src/lib/stores/propagationProposalProjection.svelte.ts', 'utf8'), { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
let compiled = compileModule(source, { filename:'proposals.svelte.js', generate:'client' }).js.code;
const guardSource = ts.transpileModule(fs.readFileSync('src/lib/stores/projectionCacheGuards.ts','utf8'), { compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext } }).outputText;
compiled = compiled.replace(/(['"])svelte\\/internal\\/client\\1/g, JSON.stringify(import.meta.resolve('svelte/internal/client'))).replace('$lib/commandApi.js',commands).replace('$lib/projectionApi.js',reads).replace('./projectionCacheGuards.js',url(guardSource));
const store = await import(url(compiled));
const projection = version => ({version,change_event_id:'event-'+version,payload:{proposals:[]}});
for (const outcome of ['success','failure']) {
  let resolveOld, rejectOld, resolveCurrent;
  globalThis.read = () => new Promise((resolve,reject) => { resolveOld=resolve; rejectOld=reject; });
  const oldRead = store.refreshPropagationProposalListProjection();
  const refusal = assert.rejects(oldRead, outcome==='success' ? /project changed/ : /old failure/);
  const admittedEpoch = store.getPropagationProposalSessionEpoch();
  store.clearPropagationProposalListProjection();
  assert.equal(store.getPropagationProposalSessionEpoch(),admittedEpoch+1);
  globalThis.read = async () => projection(1);
  await store.refreshPropagationProposalListProjection();
  globalThis.read = () => new Promise(resolve => resolveCurrent=resolve);
  const currentRead = store.refreshPropagationProposalListProjection();
  if(outcome==='success')resolveOld(projection(99)); else rejectOld(Error('old failure'));
  await refusal;
  assert.equal(store.getCachedPropagationProposalListProjection().version,1);
  assert.equal(store.propagationProposalProjectionState.pending,true);
  assert.equal(store.propagationProposalProjectionState.error,undefined);
  resolveCurrent(projection(2)); await currentRead;
  assert.equal(store.getCachedPropagationProposalListProjection().version,2);
  assert.equal(store.propagationProposalProjectionState.pending,false);
}
globalThis.read = async () => { throw Error('current failure'); };
await assert.rejects(store.refreshPropagationProposalListProjection(),/current failure/);
assert.equal(store.propagationProposalProjectionState.error,'current failure');
console.log('client proposal refresh lifetime passed');
`,
  });
  expect(output).toContain('client proposal refresh lifetime passed');
});
