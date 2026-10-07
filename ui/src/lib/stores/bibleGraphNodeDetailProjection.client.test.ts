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
const dirty = createBibleGraphFieldDrafts({verified:()=>true,owner:()=> 'Mara:'+globalThis.session, fields:()=>fields('Mara'), save:async()=>calls++});
const clean = createBibleGraphFieldDrafts({verified:()=>true,owner:()=> 'Mara:'+globalThis.session, fields:()=>fields('Mara'), save:async()=>calls++});
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
const editor=createBibleGraphFieldDrafts({verified:()=>true,owner:()=>owner,fields:()=>current,save:()=>new Promise(resolve=>finish=resolve)});
editor.update(current[0],'Original manual draft'); const saving=editor.save(current[0]); assert.equal(editor.state.saving.shared,true);
owner='Eli'; current=[{...current[0],value:{type:'text',value:'Eli saved'}}]; editor.observe(); editor.update(current[0],'Eli manual draft'); const eliBase=editor.state.drafts.shared.base;
finish(); await saving; assert.equal(editor.value(current[0]),'Eli manual draft'); assert.equal(editor.state.drafts.shared.base,eliBase); assert.equal(editor.state.saving.shared,undefined);
const interrupted=createBibleGraphFieldDrafts({verified:()=>true,owner:()=>owner,fields:()=>current,save:async()=>{throw Error('Lost acknowledgement');}}); interrupted.update(current[0],'Retained exact draft'); await interrupted.save(current[0]); assert.equal(interrupted.value(current[0]),'Retained exact draft'); assert.equal(interrupted.state.drafts.shared.baseText,'Eli saved'); assert.equal(interrupted.state.errors.shared,'Lost acknowledgement');
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

it('retains exact submitted text and base when newer facts precede or follow a delayed acknowledgement', () =>
  run(`
for (const ordering of ['refresh-before-ack','ack-before-refresh']) {
  let current=[{id:'fact',part_id:'profile',field_key:'tagline',value:{type:'text',value:'Red'}}], finish;
  const editor=createBibleGraphFieldDrafts({verified:()=>true,owner:()=> 'Mara',fields:()=>current,save:()=>new Promise(resolve=>finish=resolve)});
  const exact='  Blue draft — 雨.\\n\\n  ';
  editor.update(current[0],exact); const base=editor.state.drafts.fact.base;
  const saving=editor.save(current[0]); const ack={...current[0],value:{type:'text',value:exact.trim()}};
  if(ordering==='refresh-before-ack') { current=[{...current[0],value:{type:'text',value:'Green'}}]; editor.observe(); }
  finish(ack); await saving;
  if(ordering==='ack-before-refresh') { current=[{...current[0],value:{type:'text',value:'Green'}}]; editor.observe(); }
  assert.equal(editor.value(current[0]),exact); assert.equal(editor.state.drafts.fact.base,base); assert.equal(editor.state.drafts.fact.baseText,'Red'); assert.equal(editor.changed(current[0]),true);
  assert.equal(editor.state.saving.fact,false); editor.discard(current[0]); assert.equal(editor.value(current[0]),'Green');
}
`));

it('clears only an owned matching acknowledgement and retains an already observed conflict', () =>
  run(`
for(const conflict of [false,true]) {
  let current=[{id:'fact',part_id:'profile',field_key:'tagline',value:{type:'text',value:'Red'}}],finish;
  const editor=createBibleGraphFieldDrafts({verified:()=>true,owner:()=> 'Mara',fields:()=>current,save:()=>new Promise(resolve=>finish=resolve)});
  editor.update(current[0],'  Blue — 雨.  '); const saving=editor.save(current[0]);
  if(conflict) { current=[{...current[0],value:{type:'text',value:'Green'}}]; editor.observe(); }
  current=[{...current[0],value:{type:'text',value:'Blue — 雨.'}}]; editor.observe(); finish(current[0]); await saving;
  if(conflict) { assert.equal(editor.value(current[0]),'  Blue — 雨.  '); assert.equal(editor.changed(current[0]),true); }
  else { assert.equal(editor.state.drafts.fact,undefined); assert.equal(editor.value(current[0]),'Blue — 雨.'); }
}
`));

it('keeps a failed cached read and pending retry unsaveable until admitted recovery', () =>
  run(`
globalThis.read=async()=>projection('Mara','Red',1); const release=store.retainBibleGraphNodeDetail(key('Mara')); await settle();
assert.equal(store.isBibleGraphNodeProjectionVerified(key('Mara')),true);
let calls=0;
const editor=createBibleGraphFieldDrafts({owner:()=> 'Mara',fields:()=>fields('Mara'),verified:()=>store.isBibleGraphNodeProjectionVerified(key('Mara')),save:async()=>calls++});
editor.update(fields('Mara')[0],'  Local draft — 雨.\\n\\n  '); const base=editor.state.drafts['Mara.tagline'].base;
globalThis.read=async()=>{throw Error('Current detail refresh failed');}; await assert.rejects(store.refreshOwnedBibleGraphNodeProjections(),/refresh failed/);
assert.equal(fields('Mara')[0].value.value,'Red'); assert.equal(store.getBibleGraphNodeProjectionError(key('Mara')),'Current detail refresh failed');
await editor.save(fields('Mara')[0]); assert.equal(calls,0); assert.equal(store.isBibleGraphNodeProjectionVerified(key('Mara')),false);
let finish;globalThis.read=()=>new Promise(resolve=>finish=resolve);const retry=store.refreshOwnedBibleGraphNodeProjections();
assert.equal(store.getBibleGraphNodeProjectionError(key('Mara')),undefined); await editor.save(fields('Mara')[0]); assert.equal(calls,0);
finish(projection('Mara','Green',2)); await retry; editor.observe(); assert.equal(store.isBibleGraphNodeProjectionVerified(key('Mara')),true);
assert.equal(editor.value(fields('Mara')[0]),'  Local draft — 雨.\\n\\n  '); assert.equal(editor.state.drafts['Mara.tagline'].base,base); assert.equal(editor.changed(fields('Mara')[0]),true); await editor.save(fields('Mara')[0]);assert.equal(calls,0);
editor.discard(fields('Mara')[0]); await editor.save(fields('Mara')[0]); assert.equal(calls,1); release();
`));

it('reconciles a matching acknowledgement during delayed refresh while keeping Save gated until recovery', () =>
  run(`
globalThis.read=async()=>projection('Mara','Red',1);const release=store.retainBibleGraphNodeDetail(key('Mara'));await settle();
let acknowledge,refresh;const editor=createBibleGraphFieldDrafts({owner:()=> 'Mara',fields:()=>fields('Mara'),verified:()=>store.isBibleGraphNodeProjectionVerified(key('Mara')),save:()=>new Promise(resolve=>acknowledge=resolve)});
editor.update(fields('Mara')[0],'  Blue draft — 雨.  ');const saving=editor.save(fields('Mara')[0]);
globalThis.read=()=>new Promise(resolve=>refresh=resolve);const reading=store.refreshOwnedBibleGraphNodeProjections();
const ack=projection('Mara','Blue draft — 雨.',2);store.cacheNodeProjection(store.cacheKey(key('Mara')),ack);acknowledge(ack.payload.parts[0].fields[0]);await saving;
assert.equal(editor.state.drafts['Mara.tagline'],undefined);assert.equal(editor.value(fields('Mara')[0]),'Blue draft — 雨.');assert.equal(store.isBibleGraphNodeProjectionVerified(key('Mara')),false);
await editor.save(fields('Mara')[0]);assert.match(editor.state.errors['Mara.tagline'],/Verify saved facts/);
refresh(projection('Mara','Red',1));await assert.rejects(reading,/older revision/);assert.equal(store.isBibleGraphNodeProjectionVerified(key('Mara')),false);assert.equal(fields('Mara')[0].value.value,'Blue draft — 雨.');
globalThis.read=async()=>projection('Mara','Green',3);await store.refreshOwnedBibleGraphNodeProjections();editor.observe();assert.equal(editor.changed(fields('Mara')[0]),false);assert.equal(editor.value(fields('Mara')[0]),'Green');release();
`));

it('does not let an old save complete a new same-name owner after selection ABA', () =>
  run(`
let owner='Mara', current=[{id:'fact',part_id:'profile',field_key:'tagline',value:{type:'text',value:'Red'}}];const completions=[];
const editor=createBibleGraphFieldDrafts({verified:()=>true,owner:()=>owner,fields:()=>current,save:()=>new Promise(resolve=>completions.push(resolve))});
editor.update(current[0],'Old blue');const old=editor.save(current[0]);owner='Eli';editor.observe();owner='Mara';editor.observe();editor.update(current[0],'New exact draft — 雨.');const fresh=editor.save(current[0]);
completions[0]({...current[0],value:{type:'text',value:'Old blue'}});await old;assert.equal(editor.value(current[0]),'New exact draft — 雨.');assert.equal(editor.state.saving.fact,true);
current=[{...current[0],value:{type:'text',value:'New exact draft — 雨.'}}];completions[1](current[0]);await fresh;assert.equal(editor.state.drafts.fact,undefined);assert.equal(editor.state.saving.fact,false);
`));
