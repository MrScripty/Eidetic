import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';

const repo = process.argv[2];
assert(repo, 'Pass the QA worktree root');
const ui = path.join(repo, 'ui');
const require = createRequire(path.join(ui, 'package.json'));
const ts = (await import(pathToFileURL(require.resolve('typescript')).href)).default;
const compiler = await import(pathToFileURL(require.resolve('svelte/compiler')).href);
const compileModule = compiler.compileModule ?? compiler.default.compileModule;
const svelteURL = pathToFileURL(require.resolve('svelte')).href;
const clientURL = pathToFileURL(require.resolve('svelte/internal/client')).href;
const { flushSync } = await import(svelteURL);
const { effect_root, render_effect } = await import(clientURL);
const oldRef = 'ff8f822f299745479b3f7d514824a54b2922c75d';
const newRef = 'dabf0206c10c6552e7061fc18f8deb9c69c99062';
const wrapperPath = 'ui/src/qualification/manualFacts.svelte.ts';
const key = { node_id: 'qualification.mara' };
const url = text => 'data:text/javascript;base64,' + Buffer.from(text).toString('base64');
const transpile = source => ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
}).outputText;
const compile = (source, filename) => compileModule(transpile(source), {
  filename, generate: 'client',
}).js.code.replace(/(['"])svelte\/internal\/client\1/g, JSON.stringify(clientURL));
const sourceAt = (ref, file) => execFileSync('git', ['show', `${ref}:${file}`], {
  cwd: repo, encoding: 'utf8',
});
const sha = text => createHash('sha256').update(text).digest('hex');
let serial = 0;

async function instance(ref) {
  const id = ++serial;
  const reads = url(`
    export async function getBibleGraphNodeProjection(key) {
      globalThis.backendReads++;
      return {version:1,change_event_id:'event-1',payload:{node:{id:key.node_id,name:'Mara'},parts:[]}};
    }
    export async function getScriptDocumentProjection() {return {payload:{segments:[]}};}
  `);
  const commands = url('export async function setBibleGraphField(){ return {}; }');
  // These controls are not invoked by this read-lifetime regression. The wrapper
  // getter and production store under test are real, client-compiled source.
  const unusedRefresh = url('export async function refreshBibleGraphNodeProjection(){}');
  const wrapperSource = sourceAt(ref, wrapperPath);
  const wrapperCode = compile(wrapperSource, 'manualFacts.svelte.js')
    .replaceAll('../lib/commandApi.js', commands)
    .replaceAll('../lib/projectionApi.js', reads)
    .replaceAll('../lib/stores/bibleGraphNodeDetailProjection.svelte.js', unusedRefresh)
    .replace(/(['"])svelte\1/g, JSON.stringify(svelteURL)) + `\n// instance ${id}\n`;
  const wrapperURL = url(wrapperCode);
  const qa = await import(wrapperURL);
  const session = url('export function getEditorSessionGeneration(){ return 0; }');
  const guards = url(transpile(sourceAt(ref, 'ui/src/lib/stores/projectionCacheGuards.ts')));
  const owners = url(transpile(sourceAt(ref, 'ui/src/lib/stores/bibleGraphNodeReadOwners.ts')));
  const storeCode = compile(sourceAt(ref, 'ui/src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts'), 'bibleGraphNodeDetailProjection.svelte.js')
    .replaceAll('$lib/projectionApi.js', wrapperURL)
    .replaceAll('./editor.svelte.js', session)
    .replaceAll('./projectionCacheGuards.js', guards)
    .replaceAll('./bibleGraphNodeReadOwners.js', owners) + `\n// instance ${id}\n`;
  return { qa, store: await import(url(storeCode)), wrapperSHA: sha(wrapperSource) };
}

async function settle() {
  for (let n = 0; n < 8; n++) {
    await Promise.resolve();
    flushSync();
  }
}

function mount(store, limit = Infinity) {
  let runs = 0, releases = 0;
  let stop;
  flushSync(() => {
    stop = effect_root(() => render_effect(() => {
      runs++;
      // Bound the intentionally broken old effect loop so this regression cannot
      // monopolize the process. The corrected case never reaches this branch.
      if (runs >= limit) return;
      const release = store.retainBibleGraphNodeDetail(key);
      return () => { releases++; release(); };
    }));
  });
  return { stop, get runs() { return runs; }, get releases() { return releases; } };
}

console.log(JSON.stringify({
  scope: 'Actual Svelte client-compiled QA wrapper and production detail store; real effect/retain/read/release lifetimes; transport backend response is synthetic; no native or model execution',
  oldRef, newRef, product: 'e5a09ede159a6511694896fdb5d4382e51f4418d',
}));

globalThis.backendReads = 0;
const old = await instance(oldRef);
const oldOwner = mount(old.store, 6);
const oldOtherOwner = mount(old.store, 6);
await settle();
assert.equal(oldOwner.runs, 1);
assert.equal(oldOtherOwner.runs, 1);
flushSync(() => { old.qa.factQA.detailMode = 'fail'; });
await settle();
assert(oldOwner.runs > 2, 'Old fault state must re-enter the owner effect');
assert(oldOwner.releases > 0, 'Old owner must be released during fault-state re-entry');
assert(old.qa.factQA.detailFailures > 0);
console.log(JSON.stringify({ oldFailureReproduced: true, ownerRuns: oldOwner.runs,
  ownerReleases: oldOwner.releases, detailFailures: old.qa.factQA.detailFailures,
  otherOwnerRuns: oldOtherOwner.runs, otherOwnerReleases: oldOtherOwner.releases,
  admittedError: old.store.getBibleGraphNodeProjectionError(key) ?? null,
  pending: old.store.isBibleGraphNodeProjectionPending(key), wrapperSHA: old.wrapperSHA }));
oldOwner.stop();
oldOtherOwner.stop();

for (const mode of ['normal', 'fail', 'hold']) {
  globalThis.backendReads = 0;
  const current = await instance(newRef);
  current.qa.factQA.detailMode = mode;
  const owner = mount(current.store);
  const otherOwner = mount(current.store);
  await settle();
  assert.equal(owner.runs, 1, `${mode}: initial read must not re-enter owner`);
  assert.equal(otherOwner.runs, 1, `${mode}: second inspector must not re-enter owner`);
  assert.equal(owner.releases, 0);
  if (mode === 'fail') {
    assert.equal(current.qa.factQA.detailFailures, 2);
    assert.equal(current.store.getBibleGraphNodeProjectionError(key), 'SYNTHETIC QA transport: Bible detail unavailable.');
    assert.equal(current.store.isBibleGraphNodeProjectionPending(key), false);
    assert.equal(current.store.isBibleGraphNodeProjectionVerified(key), false);
    assert.equal(globalThis.backendReads, 0);
  } else if (mode === 'hold') {
    assert.equal(current.qa.factQA.detailReadsHeld, 2);
    assert.equal(current.store.isBibleGraphNodeProjectionPending(key), true);
    assert.equal(globalThis.backendReads, 0);
    current.qa.releaseDetailRecovery();
    await settle();
    assert.equal(current.store.isBibleGraphNodeProjectionVerified(key), true);
    assert.equal(globalThis.backendReads, 2);
  } else {
    assert.equal(current.store.isBibleGraphNodeProjectionVerified(key), true);
    assert.equal(globalThis.backendReads, 2);
    current.qa.factQA.detailMode = 'fail';
    flushSync();
    assert.equal(owner.runs, 1, 'Changing QA mode alone must not re-enter retain');
    await assert.rejects(current.store.refreshOwnedBibleGraphNodeProjections(), /Bible detail unavailable/);
    await settle();
    assert.equal(current.qa.factQA.detailFailures, 1);
    assert.equal(current.store.isBibleGraphNodeProjectionPending(key), false);
    assert.equal(current.store.isBibleGraphNodeProjectionVerified(key), false);
    current.qa.holdDetailRecovery();
    const held = current.store.refreshOwnedBibleGraphNodeProjections();
    await settle();
    assert.equal(current.qa.factQA.detailReadsHeld, 1);
    assert.equal(current.store.isBibleGraphNodeProjectionPending(key), true);
    current.qa.releaseDetailRecovery();
    await held;
    await settle();
    assert.equal(current.store.isBibleGraphNodeProjectionVerified(key), true);
    assert.equal(globalThis.backendReads, 3);
  }
  assert.equal(owner.runs, 1, `${mode}: QA mode/counter changes must not re-enter owner`);
  assert.equal(owner.releases, 0);
  assert.equal(otherOwner.runs, 1);
  assert.equal(otherOwner.releases, 0);
  console.log(JSON.stringify({ correctedMode: mode, ownerRuns: owner.runs,
    otherOwnerRuns: otherOwner.runs, otherOwnerReleases: otherOwner.releases,
    ownerReleases: owner.releases, detailFailures: current.qa.factQA.detailFailures,
    detailReadsHeld: current.qa.factQA.detailReadsHeld, backendReads: globalThis.backendReads,
    verified: current.store.isBibleGraphNodeProjectionVerified(key), wrapperSHA: current.wrapperSHA }));
  owner.stop();
  assert.equal(owner.releases, 1);
  otherOwner.stop();
  assert.equal(otherOwner.releases, 1);
  assert.equal(current.store.getCachedBibleGraphNodeProjection(key), undefined);
}
console.log('PASS: old fault owner re-entry reproduced; corrected normal/fail/hold reads retain their owner and admit failure/recovery correctly.');
