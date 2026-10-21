import { execFileSync } from 'node:child_process';
import { expect, it } from 'vitest';

it('executes real browser rune proxies with immutable text and type retries', () => {
  const output = execFileSync(process.execPath, ['--input-type=module'], {
    encoding: 'utf8',
    input: `
import assert from 'node:assert/strict';import fs from 'node:fs';import ts from 'typescript';import {compileModule} from 'svelte/compiler';
let source=fs.readFileSync('src/lib/components/editor/scriptBlockEditDraft.svelte.ts','utf8').replace("import { createCommandId } from '$lib/commandTransport.js';","const createCommandId = () => 'fixed-type-id';");
const js=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ESNext,module:ts.ModuleKind.ESNext}}).outputText;
let compiled=compileModule(js,{filename:'type.svelte.js',generate:'client'}).js.code;
compiled=compiled.replace(/(['"])svelte\\/internal\\/client\\1/g,JSON.stringify(import.meta.resolve('svelte/internal/client')));
const {createScriptBlockEditDraft}=await import('data:text/javascript;base64,'+Buffer.from(compiled).toString('base64'));
const calls=[];let fail=true;const draft=createScriptBlockEditDraft({documentId:'main',blockId:'A',readCurrent:async()=>null,reload:async()=>{},save:async(payload,id)=>{calls.push(structuredClone({payload,id}));if(fail){fail=false;throw new Error('Lost acknowledgement');}}});
draft.begin({block:{id:'A',segment_id:'s',block_kind:'action',text:'  Exact — 雨.\\n\\n  ',sort_order:0},revision_event_id:'event.A',spans:[],locks:[]});draft.state.kind='dialogue';await draft.save();assert.equal(draft.state.uncertain,true);
draft.state.kind='note';draft.state.text='Later mutation';await draft.save();assert.deepEqual(calls[0],calls[1]);assert.equal(calls[1].payload.block_kind,'dialogue');assert.equal(draft.state.editing,false);console.log('Browser typed edit custody passed');
`,
  });
  expect(output).toContain('Browser typed edit custody passed');
});
