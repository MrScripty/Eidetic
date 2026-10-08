"""No-display regression checks for the synthetic provider's exact fact custody."""
import ast
import json
import subprocess
from pathlib import Path
import unittest

source=Path(__file__).with_name('qualify-manual-fact-reconciliation.py').read_text()
module=ast.parse(source)
# Load only constants and the pure prompt predicate, without pyatspi/native imports.
selected=[n for n in module.body if isinstance(n,ast.Assign) and all(isinstance(t,ast.Name) and t.id in {'RED','BLUE','SAVED','DRAFT','BIBLE_DRAFT','MOTIVATION_DRAFT','NOTES','GENERATED_B','ANCESTOR_OLD','ANCESTOR_MANUAL','UNRELATED_ACT_OLD','ARC_NEW','ARC_PREVIEW','ARC_UNRELATED'} for t in n.targets) or isinstance(n,ast.FunctionDef) and n.name in ('fact_prompt','notes_prompt','context_prompt_matches','ancestor_notes_prompt','arc_description_prompt')]
namespace={'json':json};exec(compile(ast.Module(body=selected,type_ignores=[]),'<pure qualifier>','exec'),namespace)

class NativeInputTests(unittest.TestCase):
    def test_native_key_steps_preserve_exact_unicode_whitespace_and_newlines(self):
        helper=ast.parse(Path(__file__).with_name('manual-fact-native-helpers.py').read_text())
        function=next(n for n in helper.body if isinstance(n,ast.FunctionDef) and n.name=='native_input_steps')
        local={};exec(compile(ast.Module(body=[function],type_ignores=[]),'<native input>','exec'),local)
        text=namespace['SAVED'];steps=local['native_input_steps'](text)
        restored=''.join(value if kind=='type' else '\n' if kind=='newline' else chr(int(value,16)) for kind,value in steps)
        self.assertEqual(restored,text)
        self.assertIn(('unicode','2014'),steps)
        self.assertIn(('unicode','96e8'),steps)

class PromptTests(unittest.TestCase):
    def evidence(self):
        return {'text':namespace['SAVED'],'before_revision_event_id':'before','revision_event_id':'saved','facts':[{'field_id':'qualification.mara.tagline','text':namespace['RED'],'consumed_text':namespace['RED']}]}
    def test_exact_one_consumed_field(self):
        self.assertTrue(namespace['fact_prompt'](json.dumps(self.evidence()),0))
    def test_other_field_and_missing_changed_revision_refuse(self):
        e=self.evidence();e['facts'][0]['field_id']='qualification.eli.tagline'
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
        e=self.evidence();e['revision_event_id']='before'
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_unselected_field_or_draft_leak_refuses(self):
        for extra in [{'text':'UNCONSUMED'}, {'text':namespace['DRAFT']}]:
            e=self.evidence();e['facts'].append(extra)
            self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_unsaved_bible_drafts_cannot_leak_even_outside_selected_fact(self):
        for draft in (namespace['BIBLE_DRAFT'],namespace['MOTIVATION_DRAFT']):
            e=self.evidence();e['unexpected_local_draft']=draft
            self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))
    def test_later_proposal_uses_new_current_and_original_consumed_fact(self):
        e=self.evidence();e['facts'][0]['text']=namespace['BLUE']
        self.assertTrue(namespace['fact_prompt'](json.dumps(e),4))
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),3))
    def test_saved_whitespace_is_not_normalized(self):
        e=self.evidence();e['text']=e['text'].strip()
        self.assertFalse(namespace['fact_prompt'](json.dumps(e),0))

class NotesPromptTests(unittest.TestCase):
    def prompt(self):
        return 'CURRENT SELECTED CLIP NOTES:\n'+namespace['NOTES']+'\n'+namespace['GENERATED_B']+namespace['RED']
    def test_targeted_preview_receives_exact_saved_notes_and_original_target(self):
        self.assertTrue(namespace['notes_prompt']('Draft a targeted screenplay update',self.prompt()))
        self.assertFalse(namespace['notes_prompt']('ordinary generation',self.prompt()))
    def test_normalized_notes_missing_target_and_unsaved_draft_leaks_refuse(self):
        for text in [self.prompt().replace(namespace['NOTES'],namespace['NOTES'].strip()),self.prompt().replace(namespace['GENERATED_B'],''),self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['notes_prompt']('Draft a targeted screenplay update',text))

class ActualContextPromptTests(unittest.TestCase):
    def test_exact_authored_notes_with_unicode_and_whitespace(self):
        notes=namespace['NOTES'];user='SCENE: SCENE B\nSCENE NOTES:\n'+notes+'\n\nCONTEXT HIERARCHY:\n'
        self.assertTrue(namespace['context_prompt_matches'](user,notes))
        self.assertFalse(namespace['context_prompt_matches'](user,notes.strip()))
    def test_draft_leak_and_stale_notes_refuse(self):
        user='SCENE NOTES:\nold saved notes\n\n'
        self.assertFalse(namespace['context_prompt_matches'](user,namespace['NOTES']))
        self.assertFalse(namespace['context_prompt_matches'](user+namespace['DRAFT'],'old saved notes'))



class AncestorNotesPromptTests(unittest.TestCase):
    def prompt(self):
        return ('CURRENT CONSUMED ANCESTOR NOTES (owned.act):\n'+namespace['NOTES']+'\n'
            +'TARGET BLOCK TO UPDATE:\n'+namespace['ANCESTOR_MANUAL']+namespace['RED'])
    def test_exact_ancestor_and_saved_manual_target_are_admitted(self):
        self.assertTrue(namespace['ancestor_notes_prompt']('targeted screenplay update',self.prompt()))
    def test_trimmed_notes_missing_manual_target_and_unrelated_or_draft_leak_are_refused(self):
        for text in [self.prompt().replace(namespace['NOTES'],namespace['NOTES'].strip()),
            self.prompt().replace(namespace['ANCESTOR_MANUAL'],namespace['GENERATED_B']),
            self.prompt()+namespace['UNRELATED_ACT_OLD'],self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['ancestor_notes_prompt']('targeted screenplay update',text))

class CanonicalImpactReceiptTests(unittest.TestCase):
    def test_actual_read_seam_retains_exact_ancestor_and_existing_review_causes(self):
        # Execute the actual QA public-read wrapper with a mixed projection. Only
        # the transport/state primitives are replaced; its filtering is unchanged.
        root=Path(__file__).resolve().parents[1]
        script=r'''
const fs=require('node:fs'),vm=require('node:vm');
const ts=require('./ui/node_modules/typescript');
const causes=[
  {dependency_id:'generation.event.ancestor_notes.owned-act',input:{kind:'timeline_node',node_id:'owned-act'},input_excerpt:'  exact — 雨.\n\n  '},
  {dependency_id:'generation.event.timeline_notes',input:{kind:'timeline_node',node_id:'scene'}},
  {dependency_id:'generation.event.field',input:{kind:'bible_field'}},
  {dependency_id:'generation.event.edge',input:{kind:'bible_edge'}},
  {dependency_id:'generation.event.arc',input:{kind:'story_arc_field'}},
  {dependency_id:'generation.event.node',input:{kind:'timeline_node',node_id:'other'}}
];
const projection={payload:{segments:[{segment:{source_node_id:'scene',id:'segment'},impact:{needs_review:true,causes}}]}};
const exports={};
const context={exports,$state:x=>x,require:name=>name==='svelte'?{untrack:f=>f()}:name.endsWith('projectionApi.js')?{getScriptDocumentProjection:async()=>projection}:{}};
const source=fs.readFileSync('ui/src/qualification/manualFacts.svelte.ts','utf8');
vm.runInNewContext(ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText,context);
exports.readImpacts().then(()=>process.stdout.write(exports.receipt())).catch(error=>{console.error(error);process.exitCode=1;});
'''
        result=subprocess.run(['node','-e',script],cwd=root,text=True,capture_output=True,check=True)
        receipt=json.loads(result.stdout)
        self.assertEqual(receipt['error'],'')
        row=json.loads(receipt['impacts'])[0]
        self.assertEqual(row['source'],'scene')
        self.assertTrue(row['needsReview'])
        self.assertEqual([c['dependency_id'] for c in row['causes']],[
            'generation.event.ancestor_notes.owned-act','generation.event.timeline_notes',
            'generation.event.field','generation.event.edge','generation.event.arc'])
        self.assertEqual(row['causes'][0]['input'],{'kind':'timeline_node','node_id':'owned-act'})
        self.assertEqual(row['causes'][0]['input_excerpt'],'  exact — 雨.\n\n  ')


class ArcDescriptionPromptTests(unittest.TestCase):
    def prompt(self):
        return ('ORIGINAL KNOWN-EMPTY ARC DESCRIPTION APPLICABILITY (arc): description prose was not supplied; field revision owned\n'
            +namespace['ARC_NEW']+'TARGET BLOCK TO UPDATE:\n'+namespace['ANCESTOR_MANUAL']+namespace['RED'])
    def test_exact_new_direction_original_omission_and_manual_target(self):
        self.assertTrue(namespace['arc_description_prompt']('targeted screenplay update',self.prompt()))
    def test_missing_omission_trimmed_description_or_draft_or_unrelated_leaks_refuse(self):
        for text in [self.prompt().replace('description prose was not supplied; field revision ','unknown '),
            self.prompt().replace(namespace['ARC_NEW'],namespace['ARC_NEW'].strip()),
            self.prompt().replace(namespace['ANCESTOR_MANUAL'],namespace['GENERATED_B']),
            self.prompt()+namespace['ARC_UNRELATED'],self.prompt()+namespace['DRAFT']]:
            self.assertFalse(namespace['arc_description_prompt']('targeted screenplay update',text))

if __name__=='__main__':unittest.main()
