import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

const bootstrap = `
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { compileModule } from 'svelte/compiler';
const url = source => 'data:text/javascript;base64,' + Buffer.from(source).toString('base64');
const transpile = file => ts.transpileModule(fs.readFileSync(file,'utf8'), {compilerOptions:{target:ts.ScriptTarget.ESNext,module:ts.ModuleKind.ESNext}}).outputText;
const compile = file => compileModule(transpile(file),{filename:file.replace(/\\.ts$/,'.js'),generate:'client'}).js.code.replace(/(['"])svelte\\/internal\\/client\\1/g,JSON.stringify(import.meta.resolve('svelte/internal/client')));
const reads = url('export function getBibleGraphNodeProjection(key) { return globalThis.read(key); }');
const session = url('export function getEditorSessionGeneration() { return globalThis.session; }');
globalThis.session = 0;
let source = compile('src/lib/stores/bibleGraphNodeDetailProjection.svelte.ts').replace('$lib/projectionApi.js',reads).replace('./editor.svelte.js',session).replace('./projectionCacheGuards.js',url(transpile('src/lib/stores/projectionCacheGuards.ts'))).replace('./bibleGraphNodeReadOwners.js',url(transpile('src/lib/stores/bibleGraphNodeReadOwners.ts')));
const store = await import(url(source));
const { createBibleGraphFieldDrafts } = await import(url(compile('src/lib/components/sidebar/bible/bibleGraphFieldDrafts.svelte.ts')));
const projection = (node,value,version=1) => ({version,change_event_id:'event-'+version,payload:{node:{id:node,name:node},parts:[{part:{id:node+'.profile'},fields:[{id:node+'.tagline',part_id:node+'.profile',field_key:'tagline',value:{type:'text',value}},{id:node+'.motivation',part_id:node+'.profile',field_key:'motivation',value:{type:'text',value:'Unchanged motivation'}}]}]}});
const key = node => ({node_id:node});
const fields = node => store.getCachedBibleGraphNodeProjection(key(node)).payload.parts[0].fields;
const settle = async () => { for(let n=0;n<6;n++)await Promise.resolve(); };
`;

function run(body: string) {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: bootstrap + body + "\nconsole.log('owned Bible detail regression passed');\n",
  });
  expect(output).toContain('owned Bible detail regression passed');
}

it('refreshes clean committed fields while dirty same-field and unrelated drafts retain their bases', () =>
  run(`
globalThis.read = async () => projection('Mara','Red');
const left = store.retainBibleGraphNodeDetail(key('Mara')); const right = store.retainBibleGraphNodeDetail(key('Mara')); await settle();
let calls=0;
const dirty = createBibleGraphFieldDrafts({owner:()=> 'Mara:'+globalThis.session, fields:()=>fields('Mara'), save:async()=>calls++});
const clean = createBibleGraphFieldDrafts({owner:()=> 'Mara:'+globalThis.session, fields:()=>fields('Mara'), save:async()=>calls++});
dirty.update(fields('Mara')[0],'  Unsaved blue draft — 雨.  '); dirty.update(fields('Mara')[1],'Independent motivation draft');
const base = dirty.state.drafts['Mara.tagline'].base;
globalThis.read = async () => projection('Mara','Blue',2); await store.refreshOwnedBibleGraphNodeProjections(); dirty.observe();
assert.equal(clean.value(fields('Mara')[0]),'Blue'); assert.equal(dirty.value(fields('Mara')[0]),'  Unsaved blue draft — 雨.  ');
assert.equal(dirty.state.drafts['Mara.tagline'].base,base); assert.equal(dirty.state.drafts['Mara.tagline'].baseText,'Red');
assert.equal(dirty.changed(fields('Mara')[0]),true); assert.equal(dirty.changed(fields('Mara')[1]),false); await dirty.save(fields('Mara')[0]); assert.equal(calls,0);
assert.match(dirty.state.errors['Mara.tagline'],/draft is preserved/);
globalThis.read = async () => projection('Mara','Red',3); await store.refreshOwnedBibleGraphNodeProjections(); dirty.observe(); assert.equal(dirty.changed(fields('Mara')[0]),true);
dirty.discard(fields('Mara')[0]); assert.equal(dirty.value(fields('Mara')[0]),'Red'); assert.equal(dirty.value(fields('Mara')[1]),'Independent motivation draft');
left(); assert.equal(store.getCachedBibleGraphNodeProjection(key('Mara')).version,3); right(); assert.equal(store.getCachedBibleGraphNodeProjection(key('Mara')),undefined);
`));

it('retires delayed success and failure across rapid selection ABA without changing the new owner', () =>
  run(`
for(const outcome of ['success','failure']) {
  store.clearBibleGraphNodeDetailProjections(); let finishOld,failOld,finishNew;
  globalThis.read = () => new Promise((resolve,reject)=>{finishOld=resolve;failOld=reject;}); const releaseOld=store.retainBibleGraphNodeDetail(key('Mara'));
  releaseOld(); globalThis.read = async()=>projection('Eli','Eli saved'); const releaseEli=store.retainBibleGraphNodeDetail(key('Eli')); await settle();
  globalThis.read = () => new Promise(resolve=>finishNew=resolve); const releaseNew=store.retainBibleGraphNodeDetail(key('Mara'));
  if(outcome==='success')finishOld(projection('Mara','Old retired',99)); else failOld(Error('Old retired failure')); await settle(); releaseOld();
  assert.equal(store.getCachedBibleGraphNodeProjection(key('Mara')),undefined); assert.equal(store.isBibleGraphNodeProjectionPending(key('Mara')),true); assert.equal(store.getBibleGraphNodeProjectionError(key('Mara')),undefined);
  assert.equal(fields('Eli')[0].value.value,'Eli saved'); finishNew(projection('Mara','Current blue',1)); await settle(); assert.equal(fields('Mara')[0].value.value,'Current blue'); releaseEli(); releaseNew();
}
`));

it('rejects old project and superseded reads and refuses a mismatched node payload', () =>
  run(`
let finishOld,finishCurrent; globalThis.read=()=>new Promise(resolve=>finishOld=resolve); const oldRead=store.refreshBibleGraphNodeProjection(key('Mara')); const oldRefusal=assert.rejects(oldRead,/owner changed/);
globalThis.session++; store.clearBibleGraphNodeDetailProjections(); globalThis.read=()=>new Promise(resolve=>finishCurrent=resolve); const currentRead=store.refreshBibleGraphNodeProjection(key('Mara'));
finishOld(projection('Mara','Old project',99)); await oldRefusal; assert.equal(store.isBibleGraphNodeProjectionPending(key('Mara')),true); finishCurrent(projection('Mara','New project',1)); await currentRead;
let finishFirst; globalThis.read=()=>new Promise(resolve=>finishFirst=resolve); const first=store.refreshBibleGraphNodeProjection(key('Mara')); const firstRefusal=assert.rejects(first,/owner changed/);
globalThis.read=async()=>projection('Mara','Newest',2); await store.refreshBibleGraphNodeProjection(key('Mara')); finishFirst(projection('Mara','Superseded',100)); await firstRefusal; assert.equal(fields('Mara')[0].value.value,'Newest');
globalThis.read=async()=>projection('Eli','Wrong node',99); await assert.rejects(store.refreshBibleGraphNodeProjection(key('Mara')),/another node/); assert.equal(fields('Mara')[0].value.value,'Newest');
`));

it('keeps interrupted field drafts and prevents late saves from clearing a different selection', () =>
  run(`
let owner='Mara', current=[{id:'shared',part_id:'profile',field_key:'tagline',value:{type:'text',value:'Mara saved'}}], finish;
const editor=createBibleGraphFieldDrafts({owner:()=>owner,fields:()=>current,save:()=>new Promise(resolve=>finish=resolve)});
editor.update(current[0],'Original manual draft'); const saving=editor.save(current[0]); assert.equal(editor.state.saving.shared,true);
owner='Eli'; current=[{...current[0],value:{type:'text',value:'Eli saved'}}]; editor.observe(); editor.update(current[0],'Eli manual draft'); const eliBase=editor.state.drafts.shared.base;
finish(); await saving; assert.equal(editor.value(current[0]),'Eli manual draft'); assert.equal(editor.state.drafts.shared.base,eliBase); assert.equal(editor.state.saving.shared,undefined);
const interrupted=createBibleGraphFieldDrafts({owner:()=>owner,fields:()=>current,save:async()=>{throw Error('Lost acknowledgement');}}); interrupted.update(current[0],'Retained exact draft'); await interrupted.save(current[0]); assert.equal(interrupted.value(current[0]),'Retained exact draft'); assert.equal(interrupted.state.drafts.shared.baseText,'Eli saved'); assert.equal(interrupted.state.errors.shared,'Lost acknowledgement');
`));

it('keeps the shared latest read owned when one of two Bible inspectors closes', () =>
  run(`
for(const outcome of ['success','failure']) {
  store.clearBibleGraphNodeDetailProjections(); const reads=[];
  globalThis.read=()=>new Promise((resolve,reject)=>reads.push({resolve,reject}));
  const left=store.retainBibleGraphNodeDetail(key('Mara')); const right=store.retainBibleGraphNodeDetail(key('Mara')); right();
  reads[0].resolve(projection('Mara','Superseded initial',99));
  if(outcome==='success')reads[1].resolve(projection('Mara','Shared blue',1)); else reads[1].reject(Error('Current shared read failed'));
  await settle();
  if(outcome==='success')assert.equal(fields('Mara')[0].value.value,'Shared blue');
  else { assert.equal(store.getBibleGraphNodeProjectionError(key('Mara')),'Current shared read failed'); globalThis.read=async()=>projection('Mara','Retried shared blue',2); await store.refreshOwnedBibleGraphNodeProjections(); assert.equal(fields('Mara')[0].value.value,'Retried shared blue'); }
  assert.equal(store.isBibleGraphNodeProjectionPending(key('Mara')),false); left(); assert.equal(store.getCachedBibleGraphNodeProjection(key('Mara')),undefined);
}
`));
